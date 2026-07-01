use crate::audio::state::{AudioBuffer, AudioCommand, AudioState, BusRouting, PlaybackEvent};
use crate::audio::effects::{effect_metadata, normalize, EffectChain, EffectSlot};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};
use rtrb::{Consumer, Producer, RingBuffer};
use assert_no_alloc::assert_no_alloc;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;

const RESAMPLE_BUFFER_SIZE: usize = 48000 * 2 * 60 * 5; // 5 minutes stereo 48k

/// Capacity of the retire ring. A user cannot swap/remove effects faster than a
/// few per audio block, so this is comfortably oversized; if it ever fills, the
/// audio thread falls back to dropping inline (still correct, just not RT-ideal).
const RETIRE_RING_CAPACITY: usize = 64;

/// Build an `EffectSlot` from persisted config. Runs OFF the audio thread (engine
/// init), so it may allocate and call `normalize`. Persisted params are stored
/// NORMALIZED (0..1); they are mapped to real values here before being applied.
/// `sample_rate` is the real device rate so time/LFO-based DSP runs at the
/// correct rate from the first frame.
fn init_effect_slot(cfg: &crate::audio::state::EffectSlotConfig, sample_rate: u32, tempo: f32) -> Option<EffectSlot> {
    let mut fx = crate::audio::effects::create_effect(cfg.effect_type)?;
    fx.set_sample_rate(sample_rate);
    fx.set_tempo(tempo);
    let specs = effect_metadata(cfg.effect_type);
    for (pid, &val) in cfg.params.iter().enumerate() {
        let real = match specs.get(pid) {
            Some(spec) => normalize(spec, val),
            None => val,
        };
        fx.set_parameter(pid as u8, real);
    }
    Some(EffectSlot { effect: fx, mix: cfg.mix })
}

/// Hand a retired effect slot off the audio thread to be dropped elsewhere. The
/// `EffectSlot` is MOVED into the ring by value (a fat-pointer + `f32` memcpy — no
/// allocation, and crucially no `Box::new`/`box_free` on the audio thread); the
/// heap teardown of the effect happens on the drain thread. If the ring is full
/// the slot is dropped inline as a bounded fallback (the only path on which any
/// deallocation can touch the audio thread — see `RETIRE_RING_CAPACITY`).
#[inline]
fn retire_slot(retire_tx: &mut Option<Producer<EffectSlot>>, slot: EffectSlot) {
    match retire_tx {
        Some(tx) => {
            // On overflow, `push` returns the value back; dropping it here is the
            // documented fallback (rare, bounded by RETIRE_RING_CAPACITY).
            let _ = tx.push(slot);
        }
        None => drop(slot),
    }
}

struct AudioEngineThreadState {
    buffers: HashMap<usize, Arc<AudioBuffer>>,
    active_events: Vec<PlaybackEvent>,
    pre_listen_event: Option<(Arc<AudioBuffer>, f32)>, // buffer, position
    command_rx: Consumer<AudioCommand>,
    resampling_buffer: Vec<f32>,
    resampling_index: usize,
    resampling_armed: Arc<std::sync::atomic::AtomicBool>,
    bus1_fx: EffectChain,
    bus2_fx: EffectChain,
    master_fx: EffectChain,
    tempo: f32,
    /// Producer end of the retire ring. Retired effect slots are pushed here so
    /// their heap teardown runs OFF the audio thread. `None` in unit tests that
    /// don't exercise retirement.
    retire_tx: Option<Producer<EffectSlot>>,
}

pub fn start_audio_engine(state: AudioState, consumer: Consumer<AudioCommand>) -> Result<Stream, String> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("No output device available")?;
    let config = device.default_output_config().map_err(|e| e.to_string())?;

    let stream_config: StreamConfig = config.clone().into();
    let channels = stream_config.channels as usize;
    let sample_rate = stream_config.sample_rate;

    // Publish the real device rate so OFF-thread effect builds (effect swaps via
    // AudioState::set_bus_effect) use it instead of fundsp's 44.1 kHz default.
    state.sample_rate.store(sample_rate, Ordering::Relaxed);
    let init_tempo = state.tempo.lock().map(|t| *t).unwrap_or(120.0);

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let fx_config = state.fx_config.lock().unwrap().clone();
    let mut bus1_fx = EffectChain::new();
    let mut bus2_fx = EffectChain::new();

    for (i, slot) in fx_config.bus1.slots.iter().enumerate() {
        if let Some(cfg) = slot {
            if let Some(fx) = init_effect_slot(cfg, sample_rate, init_tempo) {
                bus1_fx.slots[i] = Some(fx);
            }
        }
    }

    for (i, slot) in fx_config.bus2.slots.iter().enumerate() {
        if let Some(cfg) = slot {
            if let Some(fx) = init_effect_slot(cfg, sample_rate, init_tempo) {
                bus2_fx.slots[i] = Some(fx);
            }
        }
    }

    // Retire ring: retired effect slots travel from the audio thread to a
    // dedicated drain thread, so heap teardown (Box/ring-buffer dealloc) never
    // runs in the cpal callback.
    let (retire_tx, mut retire_rx) = RingBuffer::<EffectSlot>::new(RETIRE_RING_CAPACITY);
    std::thread::Builder::new()
        .name("fx-retire".into())
        .spawn(move || loop {
            while let Ok(slot) = retire_rx.pop() {
                drop(slot); // heap teardown OFF the audio thread
            }
            if retire_rx.is_abandoned() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        })
        .expect("failed to spawn fx-retire thread");

    let mut thread_state = AudioEngineThreadState {
        buffers: HashMap::new(),
        active_events: Vec::new(),
        pre_listen_event: None,
        command_rx: consumer,
        resampling_buffer: vec![0.0; RESAMPLE_BUFFER_SIZE],
        resampling_index: 0,
        resampling_armed: state.resampling_armed,
        bus1_fx,
        bus2_fx,
        master_fx: EffectChain::new(),
        tempo: init_tempo,
        retire_tx: Some(retire_tx),
    };

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_output_stream(
            stream_config.clone(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                write_data(data, channels, sample_rate, &mut thread_state)
            },
            err_fn,
            None,
        ),
        _ => return Err("Unsupported sample format".to_string()),
    }
    .map_err(|e| e.to_string())?;

    stream.play().map_err(|e| e.to_string())?;

    Ok(stream)
}

fn write_data(
    output: &mut [f32],
    channels: usize,
    target_sample_rate: u32,
    thread_state: &mut AudioEngineThreadState,
) {
    // Process commands from UI thread (lock-free)
    while let Ok(command) = thread_state.command_rx.pop() {
        match command {
            AudioCommand::AddBuffer(pad_id, buffer) => {
                thread_state.buffers.insert(pad_id, buffer);
            }
            AudioCommand::PreListen { buffer } => {
                thread_state.pre_listen_event = Some((buffer, 0.0));
            }
            AudioCommand::TriggerPad { pad_id, mute_group, routing } => {
                if thread_state.buffers.contains_key(&pad_id) {
                    // Mute group choking logic
                    if let Some(mg) = mute_group {
                        thread_state
                            .active_events
                            .retain(|event| event.mute_group != Some(mg));
                    }

                    thread_state.active_events.push(PlaybackEvent {
                        pad_id,
                        position: 0.0,
                        volume: 1.0,
                        mute_group,
                        routing,
                    });
                }
            }
            AudioCommand::SetBusEffect { bus, slot, slot_fx } => {
                // The effect arrives FULLY BUILT off the audio thread (alloc,
                // sample-rate, tempo, default params already applied). The audio
                // thread only moves it in and retires the previous slot off-thread,
                // so no allocation or deallocation happens in this callback.
                let retire_tx = &mut thread_state.retire_tx;
                let chain = match bus {
                    BusRouting::Bus1 => Some(&mut thread_state.bus1_fx),
                    BusRouting::Bus2 => Some(&mut thread_state.bus2_fx),
                    BusRouting::Dry => None,
                };
                if let Some(chain) = chain {
                    if slot < chain.slots.len() {
                        // `replace` returns the previous slot BY VALUE (it is not
                        // dropped here); we hand it to the retire ring. Both the
                        // install and the retire are pure moves — no alloc/dealloc.
                        if let Some(old) = chain.slots[slot].replace(slot_fx) {
                            retire_slot(retire_tx, old);
                        }
                    } else {
                        // Out-of-range slot: retire the unused build off-thread.
                        retire_slot(retire_tx, slot_fx);
                    }
                } else {
                    // Dry bus has no chain; retire the unused build off-thread.
                    retire_slot(retire_tx, slot_fx);
                }
            }
            AudioCommand::SetEffectParam { bus, slot, param_id, value } => {
                // `value` is already the REAL (normalized-off-thread) value.
                let chain = match bus {
                    BusRouting::Bus1 => Some(&mut thread_state.bus1_fx),
                    BusRouting::Bus2 => Some(&mut thread_state.bus2_fx),
                    BusRouting::Dry => None,
                };
                if let Some(chain) = chain {
                    if slot < chain.slots.len() {
                        if let Some(ref mut active) = chain.slots[slot] {
                            active.effect.set_parameter(param_id, value);
                        }
                    }
                }
            }
            AudioCommand::SetEffectMix { bus, slot, mix } => {
                let chain = match bus {
                    BusRouting::Bus1 => Some(&mut thread_state.bus1_fx),
                    BusRouting::Bus2 => Some(&mut thread_state.bus2_fx),
                    BusRouting::Dry => None,
                };
                if let Some(chain) = chain {
                    if slot < chain.slots.len() {
                        if let Some(ref mut active) = chain.slots[slot] {
                            active.mix = mix;
                        }
                    }
                }
            }
            AudioCommand::RemoveBusEffect { bus, slot } => {
                let retire_tx = &mut thread_state.retire_tx;
                let chain = match bus {
                    BusRouting::Bus1 => Some(&mut thread_state.bus1_fx),
                    BusRouting::Bus2 => Some(&mut thread_state.bus2_fx),
                    BusRouting::Dry => None,
                };
                if let Some(chain) = chain {
                    if slot < chain.slots.len() {
                        // Retire the removed effect off-thread (its heap teardown
                        // must not run in the audio callback). `take` moves the
                        // slot out by value — no alloc/dealloc here.
                        if let Some(old) = chain.slots[slot].take() {
                            retire_slot(retire_tx, old);
                        }
                    }
                }
            }
            AudioCommand::SetTempo { bpm } => {
                thread_state.tempo = bpm;
                thread_state.bus1_fx.set_tempo(bpm);
                thread_state.bus2_fx.set_tempo(bpm);
                thread_state.master_fx.set_tempo(bpm);
            }
        }
    }

    // Clear the output buffer first
    for sample in output.iter_mut() {
        *sample = 0.0;
    }

    let mut finished_events = Vec::new();

    // The output buffer is interleaved: [L, R, L, R, ...]
    for frame in output.chunks_mut(channels) {
        assert_no_alloc(|| {
            let mut bus1_mix = [0.0_f32; 8];
            let mut bus2_mix = [0.0_f32; 8];
            let mut dry_mix = [0.0_f32; 8];

            for (i, event) in thread_state.active_events.iter_mut().enumerate() {
                if let Some(buffer) = thread_state.buffers.get(&event.pad_id) {
                    let ratio = buffer.sample_rate as f32 / target_sample_rate as f32;
                    let index_f = event.position;
                    let index = index_f as usize;

                    if index * (buffer.channels as usize) >= buffer.samples.len() {
                        finished_events.push(i);
                        continue;
                    }

                    // Basic resampling with nearest neighbor interpolation and mixing
                    for c in 0..channels {
                        let source_c = if c < buffer.channels as usize { c } else { 0 }; // Mono to stereo fallback
                        let sample_idx = index * (buffer.channels as usize) + source_c;

                        if sample_idx < buffer.samples.len() {
                            let sample_val = buffer.samples[sample_idx] * event.volume;
                            match event.routing {
                                BusRouting::Bus1 => bus1_mix[c] += sample_val,
                                BusRouting::Bus2 => bus2_mix[c] += sample_val,
                                BusRouting::Dry => dry_mix[c] += sample_val,
                            }
                        }
                    }

                    event.position += ratio;
                } else {
                    finished_events.push(i);
                }
            }

            // Convert to stereo frames for FX processing
            let mut b1_frame = [bus1_mix[0], if channels > 1 { bus1_mix[1] } else { bus1_mix[0] }];
            let mut b2_frame = [bus2_mix[0], if channels > 1 { bus2_mix[1] } else { bus2_mix[0] }];
            let dry_frame = [dry_mix[0], if channels > 1 { dry_mix[1] } else { dry_mix[0] }];

            thread_state.bus1_fx.process_frame(&mut b1_frame);
            thread_state.bus2_fx.process_frame(&mut b2_frame);

            let mut master_frame = [
                b1_frame[0] + b2_frame[0] + dry_frame[0],
                b1_frame[1] + b2_frame[1] + dry_frame[1],
            ];

            thread_state.master_fx.process_frame(&mut master_frame);

            let mut frame_mix = [0.0_f32; 8];
            for c in 0..channels {
                frame_mix[c] = master_frame[c.min(1)];
            }

            let mut pre_listen_finished = false;
            if let Some((buffer, position)) = &mut thread_state.pre_listen_event {
                let ratio = buffer.sample_rate as f32 / target_sample_rate as f32;
                let index_f = *position;
                let index = index_f as usize;

                if index * (buffer.channels as usize) >= buffer.samples.len() {
                    pre_listen_finished = true;
                } else {
                    for c in 0..channels {
                        let source_c = if c < buffer.channels as usize { c } else { 0 };
                        let sample_idx = index * (buffer.channels as usize) + source_c;
                        if sample_idx < buffer.samples.len() {
                            frame_mix[c] += buffer.samples[sample_idx];
                        }
                    }
                    *position += ratio;
                }
            }
            
            if pre_listen_finished {
                thread_state.pre_listen_event = None;
            }

            // Apply mix to frame with clipping protection
            let is_armed = thread_state.resampling_armed.load(Ordering::Relaxed);
            for (c, sample) in frame.iter_mut().enumerate() {
                let out_sample = frame_mix[c].clamp(-1.0_f32, 1.0_f32);
                *sample = out_sample;

                if is_armed && thread_state.resampling_index < thread_state.resampling_buffer.len() {
                    thread_state.resampling_buffer[thread_state.resampling_index] = out_sample;
                    thread_state.resampling_index += 1;
                }
            }
        });
    }

    // Remove finished events in reverse order to preserve indices
    finished_events.sort_unstable_by(|a, b| b.cmp(a));
    finished_events.dedup();
    for i in finished_events {
        if i < thread_state.active_events.len() {
            thread_state.active_events.remove(i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::state::{AudioBuffer, AudioState};

    #[test]
    fn test_write_data_mixing() {
        let (state, consumer) = AudioState::new(1024);
        let mut thread_state = AudioEngineThreadState {
            buffers: HashMap::new(),
            active_events: Vec::new(),
            pre_listen_event: None,
            command_rx: consumer,
            resampling_buffer: vec![0.0; 1024],
            resampling_index: 0,
            resampling_armed: state.resampling_armed.clone(),
            bus1_fx: EffectChain::new(),
            bus2_fx: EffectChain::new(),
            master_fx: EffectChain::new(),
            tempo: 120.0,
            retire_tx: None,
        };

        state.add_buffer(
            0,
            AudioBuffer {
                samples: vec![1.0, 1.0, 1.0, 1.0],
                channels: 1,
                sample_rate: 44100,
            },
        );
        state.add_buffer(
            1,
            AudioBuffer {
                samples: vec![-0.5, -0.5, -0.5, -0.5],
                channels: 1,
                sample_rate: 44100,
            },
        );

        state.trigger_pad(0, None, BusRouting::Dry);
        state.trigger_pad(1, None, BusRouting::Dry);

        let mut output = vec![0.0; 4]; // 2 frames of stereo
        write_data(&mut output, 2, 44100, &mut thread_state);

        assert_eq!(output, vec![0.5, 0.5, 0.5, 0.5]);
    }

    #[test]
    fn test_write_data_resampling() {
        let (state, consumer) = AudioState::new(1024);
        let mut thread_state = AudioEngineThreadState {
            buffers: HashMap::new(),
            active_events: Vec::new(),
            pre_listen_event: None,
            command_rx: consumer,
            resampling_buffer: vec![0.0; 1024],
            resampling_index: 0,
            resampling_armed: state.resampling_armed.clone(),
            bus1_fx: EffectChain::new(),
            bus2_fx: EffectChain::new(),
            master_fx: EffectChain::new(),
            tempo: 120.0,
            retire_tx: None,
        };

        state.add_buffer(
            0,
            AudioBuffer {
                samples: vec![0.1, 0.2, 0.3, 0.4],
                channels: 1,
                sample_rate: 22050, // Half the target rate
            },
        );

        state.trigger_pad(0, None, BusRouting::Dry);

        let mut output = vec![0.0; 8]; // 4 frames of stereo
        write_data(&mut output, 2, 44100, &mut thread_state);

        assert_eq!(output, vec![0.1, 0.1, 0.1, 0.1, 0.2, 0.2, 0.2, 0.2]);
    }

    #[test]
    fn test_mute_group_choking() {
        let (state, consumer) = AudioState::new(1024);
        let mut thread_state = AudioEngineThreadState {
            buffers: HashMap::new(),
            active_events: Vec::new(),
            pre_listen_event: None,
            command_rx: consumer,
            resampling_buffer: vec![0.0; 1024],
            resampling_index: 0,
            resampling_armed: state.resampling_armed.clone(),
            bus1_fx: EffectChain::new(),
            bus2_fx: EffectChain::new(),
            master_fx: EffectChain::new(),
            tempo: 120.0,
            retire_tx: None,
        };

        state.add_buffer(
            0,
            AudioBuffer {
                samples: vec![1.0, 1.0, 1.0, 1.0],
                channels: 1,
                sample_rate: 44100,
            },
        );
        state.add_buffer(
            1,
            AudioBuffer {
                samples: vec![0.5, 0.5, 0.5, 0.5],
                channels: 1,
                sample_rate: 44100,
            },
        );

        // Trigger pad 0 with mute group 1
        state.trigger_pad(0, Some(1), BusRouting::Dry);
        
        let mut output = vec![0.0; 2]; // 1 frame
        write_data(&mut output, 2, 44100, &mut thread_state);
        // It plays pad 0 (1.0)
        assert_eq!(output, vec![1.0, 1.0]);

        // Trigger pad 1 with same mute group 1
        state.trigger_pad(1, Some(1), BusRouting::Dry);

        let mut output2 = vec![0.0; 2]; // next frame
        write_data(&mut output2, 2, 44100, &mut thread_state);
        
        // Pad 0 should be choked, only Pad 1 plays (0.5)
        assert_eq!(output2, vec![0.5, 0.5]);
    }

    #[test]
    fn test_ring_buffer_commands() {
        let (state, consumer) = AudioState::new(1024);
        let mut thread_state = AudioEngineThreadState {
            buffers: HashMap::new(),
            active_events: Vec::new(),
            pre_listen_event: None,
            command_rx: consumer,
            resampling_buffer: vec![0.0; 1024],
            resampling_index: 0,
            resampling_armed: state.resampling_armed.clone(),
            bus1_fx: EffectChain::new(),
            bus2_fx: EffectChain::new(),
            master_fx: EffectChain::new(),
            tempo: 120.0,
            retire_tx: None,
        };

        // Enqueue effect commands. SetEffectParam carries a normalized 0..1 knob
        // value that is normalized to the real range off the audio thread.
        state.set_bus_effect(crate::audio::state::BusRouting::Bus1, 0, crate::audio::effects::EffectType::Delay);
        state.set_effect_param(crate::audio::state::BusRouting::Bus1, 0, 0, 0.75);
        state.set_effect_mix(crate::audio::state::BusRouting::Bus1, 0, 0.25);

        // Run one frame to process commands
        let mut output = vec![0.0; 2];
        write_data(&mut output, 2, 44100, &mut thread_state);

        // SetBusEffect created a live slot; SetEffectMix mutated its mix field.
        let active = thread_state.bus1_fx.slots[0].as_ref().expect("slot must be populated");
        assert_eq!(active.mix, 0.25);
    }

    #[test]
    fn test_unimplemented_effect_swap_is_noop() {
        let (state, consumer) = AudioState::new(1024);
        let mut thread_state = AudioEngineThreadState {
            buffers: HashMap::new(),
            active_events: Vec::new(),
            pre_listen_event: None,
            command_rx: consumer,
            resampling_buffer: vec![0.0; 1024],
            resampling_index: 0,
            resampling_armed: state.resampling_armed.clone(),
            bus1_fx: EffectChain::new(),
            bus2_fx: EffectChain::new(),
            master_fx: EffectChain::new(),
            tempo: 120.0,
            retire_tx: None,
        };

        // Install a live, implemented effect first (set its mix to a recognizable
        // value so we can prove the original slot survives the no-op swap).
        state.set_bus_effect(crate::audio::state::BusRouting::Bus1, 0, crate::audio::effects::EffectType::Delay);
        state.set_effect_mix(crate::audio::state::BusRouting::Bus1, 0, 0.42);
        let mut output = vec![0.0; 2];
        write_data(&mut output, 2, 44100, &mut thread_state);
        assert!(thread_state.bus1_fx.slots[0].is_some());

        // Swapping to an UNIMPLEMENTED variant must be a graceful no-op: it never
        // enqueues a command, so the live slot is untouched. `RingMod` is a real
        // deferred variant (`create_effect` returns `None`); do NOT use a variant
        // that later PRs implement, or this test silently stops testing the no-op.
        state.set_bus_effect(crate::audio::state::BusRouting::Bus1, 0, crate::audio::effects::EffectType::RingMod);
        let mut output = vec![0.0; 2];
        write_data(&mut output, 2, 44100, &mut thread_state);
        let active = thread_state.bus1_fx.slots[0]
            .as_ref()
            .expect("unimplemented swap must not clear the slot");
        assert_eq!(active.mix, 0.42, "the original effect slot must survive a no-op swap");
    }

    /// Helper: a thread_state with a live retire ring whose drain end is returned
    /// so a test can assert how many slots were retired off the audio thread.
    fn thread_state_with_retire(
        state: &AudioState,
        consumer: Consumer<AudioCommand>,
    ) -> (AudioEngineThreadState, Consumer<EffectSlot>) {
        let (retire_tx, retire_rx) = RingBuffer::<EffectSlot>::new(RETIRE_RING_CAPACITY);
        let ts = AudioEngineThreadState {
            buffers: HashMap::new(),
            active_events: Vec::new(),
            pre_listen_event: None,
            command_rx: consumer,
            resampling_buffer: vec![0.0; 1024],
            resampling_index: 0,
            resampling_armed: state.resampling_armed.clone(),
            bus1_fx: EffectChain::new(),
            bus2_fx: EffectChain::new(),
            master_fx: EffectChain::new(),
            tempo: 120.0,
            retire_tx: Some(retire_tx),
        };
        (ts, retire_rx)
    }

    #[test]
    fn test_effect_swap_installs_new_and_retires_old_off_thread() {
        let (state, consumer) = AudioState::new(1024);
        let (mut thread_state, mut retire_rx) = thread_state_with_retire(&state, consumer);

        // Install Delay, then swap to a different implemented effect (Reverb).
        state.set_bus_effect(BusRouting::Bus1, 0, crate::audio::effects::EffectType::Delay);
        let mut output = vec![0.0; 2];
        write_data(&mut output, 2, 44100, &mut thread_state);
        assert!(thread_state.bus1_fx.slots[0].is_some(), "first effect must install");
        assert!(retire_rx.pop().is_err(), "installing into an empty slot retires nothing");

        // Swap installs the new build and hands the OLD slot to the retire ring.
        // The whole command-drain + swap MUST be allocation- and deallocation-free
        // on the audio thread: the ready effect is moved in, the old one moved out
        // to the ring — no `Box::new`, no inline drop. `assert_no_alloc` proves it
        // (this guard is exactly what a naive `Box<EffectSlot>` retire path fails).
        state.set_bus_effect(BusRouting::Bus1, 0, crate::audio::effects::EffectType::Reverb);
        let mut output = vec![0.0; 2];
        assert_no_alloc(|| {
            write_data(&mut output, 2, 44100, &mut thread_state);
        });
        assert!(thread_state.bus1_fx.slots[0].is_some(), "swapped-in effect must be present");
        assert!(retire_rx.pop().is_ok(), "the swapped-out effect must be retired off-thread");
    }

    #[test]
    fn test_remove_effect_retires_off_thread() {
        let (state, consumer) = AudioState::new(1024);
        let (mut thread_state, mut retire_rx) = thread_state_with_retire(&state, consumer);

        state.set_bus_effect(BusRouting::Bus1, 0, crate::audio::effects::EffectType::Delay);
        let mut output = vec![0.0; 2];
        write_data(&mut output, 2, 44100, &mut thread_state);

        // Removal moves the slot out to the retire ring — also alloc/dealloc-free
        // on the audio thread.
        state.remove_bus_effect(BusRouting::Bus1, 0);
        let mut output = vec![0.0; 2];
        assert_no_alloc(|| {
            write_data(&mut output, 2, 44100, &mut thread_state);
        });
        assert!(thread_state.bus1_fx.slots[0].is_none(), "removed slot must be empty");
        assert!(retire_rx.pop().is_ok(), "the removed effect must be retired off-thread");
    }
}
