use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
}

pub struct AudioStateInner {
    pub buffers: HashMap<usize, Arc<AudioBuffer>>,
    pub active_events: Vec<PlaybackEvent>,
}

#[derive(Clone)]
pub struct AudioState {
    pub inner: Arc<Mutex<AudioStateInner>>,
}

impl AudioState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(AudioStateInner {
                buffers: HashMap::new(),
                active_events: Vec::new(),
            })),
        }
    }

    pub fn add_buffer(&self, pad_id: usize, buffer: AudioBuffer) {
        let mut inner = self.inner.lock().unwrap();
        inner.buffers.insert(pad_id, Arc::new(buffer));
    }

    pub fn trigger_pad(&self, pad_id: usize) {
        let mut inner = self.inner.lock().unwrap();
        if inner.buffers.contains_key(&pad_id) {
            inner.active_events.push(PlaybackEvent {
                pad_id,
                position: 0.0,
                volume: 1.0,
            });
        }
    }
}
