use rtrb::{Producer, RingBuffer};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use crate::audio::effects::{create_effect, effect_metadata, normalize, EffectSlot, EffectType};
use serde::{Serialize, Deserialize};
use std::fs;

/// Serde default for `EffectSlotConfig::mix` (fully wet → zero regression vs the
/// pre-mix behavior).
fn one() -> f32 {
    1.0
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EffectSlotConfig {
    pub effect_type: EffectType,
    /// Per-parameter NORMALIZED control values (0..1), one per metadata entry.
    /// `#[serde(default)]` keeps existing `fx_config.json` files deserializing.
    #[serde(default)]
    pub params: Vec<f32>,
    /// Wet/dry mix (0..1). `#[serde(default = "one")]` so legacy configs load as
    /// fully wet.
    #[serde(default = "one")]
    pub mix: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FxChainConfig {
    pub slots: [Option<EffectSlotConfig>; 4],
}

impl Default for FxChainConfig {
    fn default() -> Self {
        Self { slots: [None, None, None, None] }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AppFxConfig {
    pub bus1: FxChainConfig,
    pub bus2: FxChainConfig,
}

impl AppFxConfig {
    pub fn load() -> Self {
        if let Ok(data) = fs::read_to_string("fx_config.json") {
            if let Ok(config) = serde_json::from_str(&data) {
                return config;
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Ok(data) = serde_json::to_string(self) {
            let _ = fs::write("fx_config.json", data);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BusRouting {
    Bus1,
    Bus2,
    Dry,
}

#[derive(Clone)]
pub struct AudioBuffer {
    pub samples: Vec<f32>,
    pub channels: u16,
    pub sample_rate: u32,
}

#[derive(Clone)]
pub struct PlaybackEvent {
    pub pad_id: usize,
    pub position: f32,
    pub volume: f32,
    pub mute_group: Option<u8>,
    pub routing: BusRouting,
}

pub enum AudioCommand {
    TriggerPad {
        pad_id: usize,
        mute_group: Option<u8>,
        routing: BusRouting,
    },
    AddBuffer(usize, Arc<AudioBuffer>),
    PreListen { buffer: Arc<AudioBuffer> },
    /// The effect is FULLY BUILT off the audio thread (allocation, sample-rate,
    /// tempo, and default params already applied) and handed over ready to install.
    /// Carried BY VALUE so the audio thread only *moves* it into the slot — moving
    /// an `EffectSlot` (a `Box<dyn Effect>` fat pointer + an `f32`) is pure memory
    /// movement with no heap allocation. An unimplemented variant produces NO
    /// command (see `set_bus_effect`).
    SetBusEffect { bus: BusRouting, slot: usize, slot_fx: EffectSlot },
    /// `value` carries the REAL (already-normalized) parameter value. The audio
    /// thread writes it straight to the DSP node with no curve math.
    SetEffectParam { bus: BusRouting, slot: usize, param_id: u8, value: f32 },
    SetEffectMix { bus: BusRouting, slot: usize, mix: f32 },
    RemoveBusEffect { bus: BusRouting, slot: usize },
    SetTempo { bpm: f32 },
}

/// Fallback sample rate used for off-thread effect builds until the audio engine
/// reports the real device rate (matches fundsp's default).
pub const DEFAULT_SAMPLE_RATE: u32 = 44_100;

#[derive(Clone)]
pub struct AudioState {
    pub command_tx: Arc<std::sync::Mutex<Producer<AudioCommand>>>,
    pub resampling_armed: Arc<AtomicBool>,
    pub fx_config: Arc<std::sync::Mutex<AppFxConfig>>,
    /// Real device sample rate, published by the audio engine on startup. Read
    /// when building effects OFF the audio thread so time/LFO-based DSP runs at
    /// the correct rate instead of fundsp's 44.1 kHz default.
    pub sample_rate: Arc<AtomicU32>,
    /// Current tempo (BPM), mirrored here so off-thread effect builds seed
    /// tempo-driven nodes with the live value rather than a hardcoded default.
    pub tempo: Arc<Mutex<f32>>,
}

impl AudioState {
    pub fn new(capacity: usize) -> (Self, rtrb::Consumer<AudioCommand>) {
        let (producer, consumer) = RingBuffer::new(capacity);
        let fx_config = AppFxConfig::load();
        (
            Self {
                command_tx: Arc::new(std::sync::Mutex::new(producer)),
                resampling_armed: Arc::new(AtomicBool::new(false)),
                fx_config: Arc::new(std::sync::Mutex::new(fx_config)),
                sample_rate: Arc::new(AtomicU32::new(DEFAULT_SAMPLE_RATE)),
                tempo: Arc::new(Mutex::new(120.0)),
            },
            consumer,
        )
    }

    /// Build a fully-initialized effect slot OFF the audio thread: allocation,
    /// sample-rate, tempo, and normalized default parameters are all applied here
    /// so the audio thread only has to move the ready slot into place. Returns
    /// `None` for unimplemented variants (graceful no-op — never enqueued).
    fn build_effect_slot(&self, effect: EffectType) -> Option<EffectSlot> {
        let mut fx = create_effect(effect)?;
        fx.set_sample_rate(self.sample_rate.load(Ordering::Relaxed));
        if let Ok(tempo) = self.tempo.lock() {
            fx.set_tempo(*tempo);
        }
        // Seed metadata defaults so the live audio matches the UI knobs and the
        // persisted config (which `set_bus_effect` also writes at mix = 1.0).
        for (pid, spec) in effect_metadata(effect).iter().enumerate() {
            fx.set_parameter(pid as u8, normalize(spec, spec.default));
        }
        Some(EffectSlot { effect: fx, mix: 1.0 })
    }

    pub fn add_buffer(&self, pad_id: usize, buffer: AudioBuffer) {
        if let Ok(mut tx) = self.command_tx.lock() {
            let _ = tx.push(AudioCommand::AddBuffer(pad_id, Arc::new(buffer)));
        }
    }

    pub fn pre_listen(&self, buffer: AudioBuffer) {
        if let Ok(mut tx) = self.command_tx.lock() {
            let _ = tx.push(AudioCommand::PreListen { buffer: Arc::new(buffer) });
        }
    }

    pub fn trigger_pad(&self, pad_id: usize, mute_group: Option<u8>, routing: BusRouting) {
        if let Ok(mut tx) = self.command_tx.lock() {
            let _ = tx.push(AudioCommand::TriggerPad { pad_id, mute_group, routing });
        }
    }

    pub fn set_bus_effect(&self, bus: BusRouting, slot: usize, effect: EffectType) {
        // Build the effect OFF the audio thread (this is where allocation is
        // allowed, per the engine's zero-alloc contract). An unimplemented variant
        // yields `None` → graceful no-op: nothing is persisted or enqueued, so a
        // live slot is never disturbed.
        let slot_fx = match self.build_effect_slot(effect) {
            Some(s) => s,
            None => return,
        };

        if let Ok(mut config) = self.fx_config.lock() {
            let chain = match bus {
                BusRouting::Bus1 => Some(&mut config.bus1),
                BusRouting::Bus2 => Some(&mut config.bus2),
                BusRouting::Dry => None,
            };
            if let Some(chain) = chain {
                if slot < chain.slots.len() {
                    // Seed persisted params from the effect's metadata defaults
                    // (normalized 0..1), so the slot has one entry per parameter.
                    let params = effect_metadata(effect).iter().map(|s| s.default).collect();
                    chain.slots[slot] = Some(EffectSlotConfig {
                        effect_type: effect,
                        params,
                        mix: 1.0,
                    });
                    config.save();
                }
            } else {
                // Dry bus has no effect chain; nothing to install.
                return;
            }
        }
        if let Ok(mut tx) = self.command_tx.lock() {
            let _ = tx.push(AudioCommand::SetBusEffect { bus, slot, slot_fx });
        }
    }

    /// `value` is the NORMALIZED 0..1 knob value. Normalization to the real-unit
    /// range happens HERE (off the audio thread) via the parameter's metadata
    /// curve; the audio thread only writes the resulting real value.
    pub fn set_effect_param(&self, bus: BusRouting, slot: usize, param_id: u8, value: f32) {
        let mut real_value = value;
        if let Ok(mut config) = self.fx_config.lock() {
            let chain = match bus {
                BusRouting::Bus1 => Some(&mut config.bus1),
                BusRouting::Bus2 => Some(&mut config.bus2),
                BusRouting::Dry => None,
            };
            if let Some(chain) = chain {
                if slot < chain.slots.len() {
                    if let Some(ref mut slot_cfg) = chain.slots[slot] {
                        let specs = effect_metadata(slot_cfg.effect_type);
                        if let Some(spec) = specs.get(param_id as usize) {
                            real_value = normalize(spec, value);
                        }
                        // Persist the NORMALIZED control value (round-trippable
                        // through metadata on reload).
                        if (param_id as usize) < slot_cfg.params.len() {
                            slot_cfg.params[param_id as usize] = value;
                            config.save();
                        }
                    }
                }
            }
        }
        if let Ok(mut tx) = self.command_tx.lock() {
            let _ = tx.push(AudioCommand::SetEffectParam { bus, slot, param_id, value: real_value });
        }
    }

    pub fn set_effect_mix(&self, bus: BusRouting, slot: usize, mix: f32) {
        if let Ok(mut config) = self.fx_config.lock() {
            let chain = match bus {
                BusRouting::Bus1 => Some(&mut config.bus1),
                BusRouting::Bus2 => Some(&mut config.bus2),
                BusRouting::Dry => None,
            };
            if let Some(chain) = chain {
                if slot < chain.slots.len() {
                    if let Some(ref mut slot_cfg) = chain.slots[slot] {
                        slot_cfg.mix = mix;
                        config.save();
                    }
                }
            }
        }
        if let Ok(mut tx) = self.command_tx.lock() {
            let _ = tx.push(AudioCommand::SetEffectMix { bus, slot, mix });
        }
    }

    pub fn remove_bus_effect(&self, bus: BusRouting, slot: usize) {
        if let Ok(mut config) = self.fx_config.lock() {
            let chain = match bus {
                BusRouting::Bus1 => Some(&mut config.bus1),
                BusRouting::Bus2 => Some(&mut config.bus2),
                BusRouting::Dry => None,
            };
            if let Some(chain) = chain {
                if slot < chain.slots.len() {
                    chain.slots[slot] = None;
                    config.save();
                }
            }
        }
        if let Ok(mut tx) = self.command_tx.lock() {
            let _ = tx.push(AudioCommand::RemoveBusEffect { bus, slot });
        }
    }

    pub fn set_tempo(&self, bpm: f32) {
        // Mirror the tempo for off-thread effect builds (so a freshly selected
        // tempo-driven effect starts at the live BPM), then drive the audio thread.
        if let Ok(mut tempo) = self.tempo.lock() {
            *tempo = bpm;
        }
        if let Ok(mut tx) = self.command_tx.lock() {
            let _ = tx.push(AudioCommand::SetTempo { bpm });
        }
    }
}
