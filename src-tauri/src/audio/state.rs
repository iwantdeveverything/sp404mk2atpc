use rtrb::{Producer, RingBuffer};
use std::sync::Arc;

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
}

pub enum AudioCommand {
    TriggerPad {
        pad_id: usize,
        mute_group: Option<u8>,
    },
    AddBuffer(usize, Arc<AudioBuffer>),
}

#[derive(Clone)]
pub struct AudioState {
    pub command_tx: Arc<std::sync::Mutex<Producer<AudioCommand>>>,
}

impl AudioState {
    pub fn new(capacity: usize) -> (Self, rtrb::Consumer<AudioCommand>) {
        let (producer, consumer) = RingBuffer::new(capacity);
        (
            Self {
                command_tx: Arc::new(std::sync::Mutex::new(producer)),
            },
            consumer,
        )
    }

    pub fn add_buffer(&self, pad_id: usize, buffer: AudioBuffer) {
        if let Ok(mut tx) = self.command_tx.lock() {
            let _ = tx.push(AudioCommand::AddBuffer(pad_id, Arc::new(buffer)));
        }
    }

    pub fn trigger_pad(&self, pad_id: usize, mute_group: Option<u8>) {
        if let Ok(mut tx) = self.command_tx.lock() {
            let _ = tx.push(AudioCommand::TriggerPad { pad_id, mute_group });
        }
    }
}
