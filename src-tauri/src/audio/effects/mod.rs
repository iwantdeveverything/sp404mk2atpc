use fundsp::hacker32::*;
use std::sync::Arc;

pub trait Effect: Send + Sync {
    fn process_frame(&mut self, frame: &mut [f32; 2]);
    fn set_parameter(&mut self, param_id: u8, value: f32);
    fn reset(&mut self);
    fn set_sample_rate(&mut self, rate: u32);
    fn set_tempo(&mut self, _bpm: f32) {}
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

    pub fn set_tempo(&mut self, bpm: f32) {
        for slot in self.slots.iter_mut() {
            if let Some(effect) = slot {
                effect.set_tempo(bpm);
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

pub struct FunDspWrapper {
    node: Box<dyn AudioUnit + Send + Sync>,
    params: Vec<Shared>,
    tempo_param: Option<Shared>,
}

impl Effect for FunDspWrapper {
    fn process_frame(&mut self, frame: &mut [f32; 2]) {
        let input = [frame[0], frame[1]];
        let mut output = [0.0; 2];
        self.node.tick(&input, &mut output);
        frame[0] = output[0];
        frame[1] = output[1];
    }

    fn set_parameter(&mut self, param_id: u8, value: f32) {
        if let Some(p) = self.params.get(param_id as usize) {
            p.set_value(value);
        }
    }

    fn reset(&mut self) {
        self.node.reset();
    }

    fn set_sample_rate(&mut self, rate: u32) {
        self.node.set_sample_rate(rate as f64);
    }

    fn set_tempo(&mut self, bpm: f32) {
        if let Some(t) = &self.tempo_param {
            t.set_value(bpm);
        }
    }
}

impl FunDspWrapper {
    pub fn new(node: Box<dyn AudioUnit + Send + Sync>, params: Vec<Shared>) -> Self {
        Self { node, params, tempo_param: None }
    }

    pub fn new_with_tempo(node: Box<dyn AudioUnit + Send + Sync>, params: Vec<Shared>, tempo_param: Shared) -> Self {
        Self { node, params, tempo_param: Some(tempo_param) }
    }
}

pub fn create_effect(effect_type: EffectType) -> Option<Box<dyn Effect>> {
    match effect_type {
        EffectType::Filter => {
            let cutoff = shared(1000.0);
            let filter_ch = || (pass() | var(&cutoff) | dc(1.0)) >> lowpass();
            let node = filter_ch() | filter_ch();
            Some(Box::new(FunDspWrapper::new(Box::new(node), vec![cutoff])))
        }
        EffectType::Isolator => {
            let low = shared(0.0);
            let mid = shared(0.0);
            let high = shared(0.0);
            let iso_ch = || (pass() | dc(100.0) | dc(1.0) | var(&low)) >> lowshelf()
                         >> (pass() | dc(1000.0) | dc(1.0) | var(&mid)) >> bell()
                         >> (pass() | dc(5000.0) | dc(1.0) | var(&high)) >> highshelf();
            let node = iso_ch() | iso_ch();
            Some(Box::new(FunDspWrapper::new(Box::new(node), vec![low, mid, high])))
        }
        EffectType::Delay => {
            let time = shared(0.5);
            let delay_ch = || pass() & (pass() | var(&time)) >> tap(0.0, 2.0);
            let node = delay_ch() | delay_ch();
            Some(Box::new(FunDspWrapper::new(Box::new(node), vec![time])))
        }
        EffectType::Reverb => {
            let node = reverb_stereo(10.0, 1.0, 0.5);
            Some(Box::new(FunDspWrapper::new(Box::new(node), vec![])))
        }
        EffectType::VinylSim => {
            let noise = shared(0.05);
            let vinyl_ch = || (pass() & (sink() | (pink() * var(&noise)))) >> lowpass_hz(8000.0, 1.0);
            let node = vinyl_ch() | vinyl_ch();
            Some(Box::new(FunDspWrapper::new(Box::new(node), vec![noise])))
        }
        EffectType::DjfxLooper => {
            let bpm = shared(120.0);
            let depth = shared(0.5); // Feedback amount
            let b1 = bpm.clone();
            let b2 = bpm.clone();
            let d1 = depth.clone();
            let d2 = depth.clone();
            let ch1 = pass() & (pass() | (var(&b1) >> map(|b| 60.0 / b[0].max(40.0)))) >> tap(0.0, 2.0) * var(&d1);
            let ch2 = pass() & (pass() | (var(&b2) >> map(|b| 60.0 / b[0].max(40.0)))) >> tap(0.0, 2.0) * var(&d2);
            let node = ch1 | ch2;
            Some(Box::new(FunDspWrapper::new_with_tempo(Box::new(node), vec![depth], bpm)))
        }
        EffectType::Scatter => {
            let bpm = shared(120.0);
            let b1 = bpm.clone();
            let b2 = bpm.clone();
            let ch1 = pass() * ((var(&b1) >> map(|b| b[0] / 60.0 * 4.0)) >> square() >> map(|x| if x[0] > 0.0 { 1.0 } else { 0.0 }));
            let ch2 = pass() * ((var(&b2) >> map(|b| b[0] / 60.0 * 4.0)) >> square() >> map(|x| if x[0] > 0.0 { 1.0 } else { 0.0 }));
            let node = ch1 | ch2;
            Some(Box::new(FunDspWrapper::new_with_tempo(Box::new(node), vec![], bpm)))
        }
        EffectType::Slicer => {
            let bpm = shared(120.0);
            let b1 = bpm.clone();
            let b2 = bpm.clone();
            let ch1 = pass() * ((var(&b1) >> map(|b| b[0] / 60.0 * 2.0)) >> square() >> map(|x| if x[0] > 0.0 { 1.0 } else { 0.0 }));
            let ch2 = pass() * ((var(&b2) >> map(|b| b[0] / 60.0 * 2.0)) >> square() >> map(|x| if x[0] > 0.0 { 1.0 } else { 0.0 }));
            let node = ch1 | ch2;
            Some(Box::new(FunDspWrapper::new_with_tempo(Box::new(node), vec![], bpm)))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_no_alloc::assert_no_alloc;

    #[test]
    fn test_effect_instantiations() {
        let effect_types = [
            EffectType::Filter,
            EffectType::Isolator,
            EffectType::Delay,
            EffectType::Reverb,
            EffectType::VinylSim,
            EffectType::DjfxLooper,
            EffectType::Scatter,
            EffectType::Slicer,
        ];

        for &eff_type in &effect_types {
            let mut fx = create_effect(eff_type).expect("Effect should instantiate");
            fx.set_sample_rate(44100);
            
            let mut frame = [0.0, 0.0];
            fx.process_frame(&mut frame);
            
            // Should not panic, and we should be able to set parameters
            fx.set_parameter(0, 0.5);
            fx.reset();
        }
    }

    #[test]
    fn test_process_frame_no_alloc() {
        let mut fx = create_effect(EffectType::Filter).unwrap();
        fx.set_sample_rate(44100);
        let mut frame = [1.0, -1.0];

        assert_no_alloc(|| {
            fx.process_frame(&mut frame);
        });
    }
}
