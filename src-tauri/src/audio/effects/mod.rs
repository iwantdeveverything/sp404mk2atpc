pub trait Effect: Send + Sync {
    fn process_frame(&mut self, frame: &mut [f32; 2]);
    fn set_parameter(&mut self, param_id: u8, value: f32);
    fn reset(&mut self);
    fn set_sample_rate(&mut self, rate: u32);
}

pub struct EffectChain {
    pub slots: [Option<Box<dyn Effect>>; 4],
}

impl EffectChain {
    pub fn new() -> Self {
        Self {
            slots: [None, None, None, None],
        }
    }

    pub fn process_frame(&mut self, frame: &mut [f32; 2]) {
        for slot in self.slots.iter_mut() {
            if let Some(effect) = slot {
                effect.process_frame(frame);
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EffectType {
    Isolator,
    DjfxLooper,
    VinylSim,
    Filter,
    Delay,
    Reverb,
    Scatter,
    Slicer,
}
