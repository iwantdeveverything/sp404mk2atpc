use rtrb::{Producer, RingBuffer};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use crate::audio::effects::EffectType;
use serde::{Serialize, Deserialize};
use std::fs;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EffectSlotConfig {
    pub effect_type: EffectType,
    pub params: [f32; 3],
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
    SetBusEffect { bus: BusRouting, slot: usize, effect: EffectType },
    SetEffectParam { bus: BusRouting, slot: usize, param_id: u8, value: f32 },
    RemoveBusEffect { bus: BusRouting, slot: usize },
    SetTempo { bpm: f32 },
}

#[derive(Clone)]
pub struct AudioState {
    pub command_tx: Arc<std::sync::Mutex<Producer<AudioCommand>>>,
    pub resampling_armed: Arc<AtomicBool>,
    pub fx_config: Arc<std::sync::Mutex<AppFxConfig>>,
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
            },
            consumer,
        )
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
        if let Ok(mut config) = self.fx_config.lock() {
            let chain = match bus {
                BusRouting::Bus1 => Some(&mut config.bus1),
                BusRouting::Bus2 => Some(&mut config.bus2),
                BusRouting::Dry => None,
            };
            if let Some(chain) = chain {
                if slot < chain.slots.len() {
                    chain.slots[slot] = Some(EffectSlotConfig {
                        effect_type: effect,
                        params: [0.5, 0.5, 0.5], // default params
                    });
                    config.save();
                }
            }
        }
        if let Ok(mut tx) = self.command_tx.lock() {
            let _ = tx.push(AudioCommand::SetBusEffect { bus, slot, effect });
        }
    }

    pub fn set_effect_param(&self, bus: BusRouting, slot: usize, param_id: u8, value: f32) {
        if let Ok(mut config) = self.fx_config.lock() {
            let chain = match bus {
                BusRouting::Bus1 => Some(&mut config.bus1),
                BusRouting::Bus2 => Some(&mut config.bus2),
                BusRouting::Dry => None,
            };
            if let Some(chain) = chain {
                if slot < chain.slots.len() {
                    if let Some(ref mut slot_cfg) = chain.slots[slot] {
                        if (param_id as usize) < slot_cfg.params.len() {
                            slot_cfg.params[param_id as usize] = value;
                            config.save();
                        }
                    }
                }
            }
        }
        if let Ok(mut tx) = self.command_tx.lock() {
            let _ = tx.push(AudioCommand::SetEffectParam { bus, slot, param_id, value });
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
        if let Ok(mut tx) = self.command_tx.lock() {
            let _ = tx.push(AudioCommand::SetTempo { bpm });
        }
    }
}
