use crate::audio::state::{AudioBuffer, AudioCommand, AudioState, BusRouting, PlaybackEvent};
use crate::audio::effects::EffectChain;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};
use rtrb::Consumer;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;

const RESAMPLE_BUFFER_SIZE: usize = 48000 * 2 * 60 * 5; // 5 minutes stereo 48k

struct AudioEngineThreadState {
    buffers: HashMap<usize, Arc<AudioBuffer>>,
    active_events: Vec<PlaybackEvent>,
    command_rx: Consumer<AudioCommand>,
    resampling_buffer: Vec<f32>,
    resampling_index: usize,
    resampling_armed: Arc<std::sync::atomic::AtomicBool>,
    bus1_fx: EffectChain,
    bus2_fx: EffectChain,
    master_fx: EffectChain,
    tempo: f32,
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

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let mut thread_state = AudioEngineThreadState {
        buffers: HashMap::new(),
        active_events: Vec::new(),
        command_rx: consumer,
        resampling_buffer: vec![0.0; RESAMPLE_BUFFER_SIZE],
        resampling_index: 0,
        resampling_armed: state.resampling_armed,
        bus1_fx: EffectChain::new(),
        bus2_fx: EffectChain::new(),
        master_fx: EffectChain::new(),
        tempo: 120.0,
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
            AudioCommand::SetBusEffect { bus, slot, effect } => {
                // Instantiation will happen in Phase 2
            }
            AudioCommand::SetEffectParam { bus, slot, param_id, value } => {
                let chain = match bus {
                    BusRouting::Bus1 => Some(&mut thread_state.bus1_fx),
                    BusRouting::Bus2 => Some(&mut thread_state.bus2_fx),
                    BusRouting::Dry => None,
                };
                if let Some(chain) = chain {
                    if slot < chain.slots.len() {
                        if let Some(ref mut fx) = chain.slots[slot] {
                            fx.set_parameter(param_id, value);
                        }
                    }
                }
            }
            AudioCommand::RemoveBusEffect { bus, slot } => {
                let chain = match bus {
                    BusRouting::Bus1 => Some(&mut thread_state.bus1_fx),
                    BusRouting::Bus2 => Some(&mut thread_state.bus2_fx),
                    BusRouting::Dry => None,
                };
                if let Some(chain) = chain {
                    if slot < chain.slots.len() {
                        chain.slots[slot] = None;
                    }
                }
            }
            AudioCommand::SetTempo { bpm } => {
                thread_state.tempo = bpm;
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
        let mut bus1_mix = vec![0.0; channels];
        let mut bus2_mix = vec![0.0; channels];
        let mut dry_mix = vec![0.0; channels];

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
        let mut dry_frame = [dry_mix[0], if channels > 1 { dry_mix[1] } else { dry_mix[0] }];

        thread_state.bus1_fx.process_frame(&mut b1_frame);
        thread_state.bus2_fx.process_frame(&mut b2_frame);

        let mut master_frame = [
            b1_frame[0] + b2_frame[0] + dry_frame[0],
            b1_frame[1] + b2_frame[1] + dry_frame[1],
        ];

        thread_state.master_fx.process_frame(&mut master_frame);

        let mut frame_mix = vec![0.0; channels];
        for c in 0..channels {
            frame_mix[c] = master_frame[c.min(1)];
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
            command_rx: consumer,
            resampling_buffer: vec![0.0; 1024],
            resampling_index: 0,
            resampling_armed: state.resampling_armed.clone(),
            bus1_fx: EffectChain::new(),
            bus2_fx: EffectChain::new(),
            master_fx: EffectChain::new(),
            tempo: 120.0,
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
            command_rx: consumer,
            resampling_buffer: vec![0.0; 1024],
            resampling_index: 0,
            resampling_armed: state.resampling_armed.clone(),
            bus1_fx: EffectChain::new(),
            bus2_fx: EffectChain::new(),
            master_fx: EffectChain::new(),
            tempo: 120.0,
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
            command_rx: consumer,
            resampling_buffer: vec![0.0; 1024],
            resampling_index: 0,
            resampling_armed: state.resampling_armed.clone(),
            bus1_fx: EffectChain::new(),
            bus2_fx: EffectChain::new(),
            master_fx: EffectChain::new(),
            tempo: 120.0,
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
}
