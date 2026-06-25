use rtrb::{Producer, RingBuffer};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use crate::audio::effects::EffectType;

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
    SetBusEffect { bus: BusRouting, slot: usize, effect: EffectType },
    SetEffectParam { bus: BusRouting, slot: usize, param_id: u8, value: f32 },
    RemoveBusEffect { bus: BusRouting, slot: usize },
    SetTempo { bpm: f32 },
}

#[derive(Clone)]
pub struct AudioState {
    pub command_tx: Arc<std::sync::Mutex<Producer<AudioCommand>>>,
    pub resampling_armed: Arc<AtomicBool>,
}

impl AudioState {
    pub fn new(capacity: usize) -> (Self, rtrb::Consumer<AudioCommand>) {
        let (producer, consumer) = RingBuffer::new(capacity);
        (
            Self {
                command_tx: Arc::new(std::sync::Mutex::new(producer)),
                resampling_armed: Arc::new(AtomicBool::new(false)),
            },
            consumer,
        )
    }

    pub fn add_buffer(&self, pad_id: usize, buffer: AudioBuffer) {
        if let Ok(mut tx) = self.command_tx.lock() {
            let _ = tx.push(AudioCommand::AddBuffer(pad_id, Arc::new(buffer)));
        }
    }

    pub fn trigger_pad(&self, pad_id: usize, mute_group: Option<u8>, routing: BusRouting) {
        if let Ok(mut tx) = self.command_tx.lock() {
            let _ = tx.push(AudioCommand::TriggerPad { pad_id, mute_group, routing });
        }
    }

    pub fn set_bus_effect(&self, bus: BusRouting, slot: usize, effect: EffectType) {
        if let Ok(mut tx) = self.command_tx.lock() {
            let _ = tx.push(AudioCommand::SetBusEffect { bus, slot, effect });
        }
    }

    pub fn set_effect_param(&self, bus: BusRouting, slot: usize, param_id: u8, value: f32) {
        if let Ok(mut tx) = self.command_tx.lock() {
            let _ = tx.push(AudioCommand::SetEffectParam { bus, slot, param_id, value });
        }
    }
}
