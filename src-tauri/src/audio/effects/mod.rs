use fundsp::hacker32::*;
use serde::{Serialize, Deserialize};

pub trait Effect: Send + Sync {
    fn process_frame(&mut self, frame: &mut [f32; 2]);
    fn set_parameter(&mut self, param_id: u8, value: f32);
    fn reset(&mut self);
    fn set_sample_rate(&mut self, rate: u32);
    fn set_tempo(&mut self, _bpm: f32) {}
}

/// Scaling curve used to map a normalized 0..1 control value onto a parameter's
/// real-unit range. Evaluated OFF the audio thread (see `normalize`).
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Curve {
    Linear,
    Exponential,
}

/// Static descriptor for a single tunable effect parameter. Returned as a
/// `&'static` slice from `effect_metadata`, so querying metadata never allocates
/// and never requires instantiating an effect.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct ParamSpec {
    /// Stable display name, e.g. "Cutoff".
    pub name: &'static str,
    /// Real unit, e.g. "Hz", "ms", "dB", "%". Empty string when unitless.
    pub unit: &'static str,
    /// Minimum real-unit value (maps from normalized t = 0.0).
    pub min: f32,
    /// Maximum real-unit value (maps from normalized t = 1.0).
    pub max: f32,
    /// Default control position, NORMALIZED in 0..1 (seeds the UI knob).
    pub default: f32,
    /// Scaling curve applied by `normalize`.
    pub curve: Curve,
}

/// Map a normalized control value `t` (0..1) onto a parameter's real-unit range
/// using its declared curve. Pure and unit-testable; runs off the audio thread.
///
/// - Linear: `min + t * (max - min)`
/// - Exponential (`min > 0`): `min * (max / min).powf(t)`
pub fn normalize(spec: &ParamSpec, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match spec.curve {
        Curve::Linear => spec.min + t * (spec.max - spec.min),
        Curve::Exponential => spec.min * (spec.max / spec.min).powf(t),
    }
}

/// Static parameter metadata for an effect. Empty slice for variants that
/// declare no tunable parameters. `&'static` => zero allocation.
pub fn effect_metadata(effect_type: EffectType) -> &'static [ParamSpec] {
    use Curve::{Exponential, Linear};
    match effect_type {
        EffectType::Filter => &[ParamSpec {
            name: "Cutoff",
            unit: "Hz",
            min: 20.0,
            max: 20000.0,
            default: 0.5,
            curve: Exponential,
        }],
        EffectType::Isolator => &[
            ParamSpec { name: "Low", unit: "", min: 0.0, max: 2.0, default: 0.5, curve: Linear },
            ParamSpec { name: "Mid", unit: "", min: 0.0, max: 2.0, default: 0.5, curve: Linear },
            ParamSpec { name: "High", unit: "", min: 0.0, max: 2.0, default: 0.5, curve: Linear },
        ],
        EffectType::Delay => &[ParamSpec {
            name: "Time",
            unit: "s",
            min: 0.0,
            max: 2.0,
            default: 0.25,
            curve: Linear,
        }],
        EffectType::Reverb => &[ParamSpec {
            name: "Damping",
            unit: "Hz",
            min: 200.0,
            max: 18000.0,
            default: 1.0,
            curve: Exponential,
        }],
        EffectType::VinylSim => &[ParamSpec {
            name: "Noise",
            unit: "",
            min: 0.0,
            max: 0.5,
            default: 0.1,
            curve: Linear,
        }],
        EffectType::DjfxLooper => &[ParamSpec {
            name: "Feedback",
            unit: "",
            min: 0.0,
            max: 1.0,
            default: 0.5,
            curve: Linear,
        }],
        EffectType::Scatter => &[ParamSpec {
            name: "Depth",
            unit: "",
            min: 0.0,
            max: 1.0,
            default: 1.0,
            curve: Linear,
        }],
        EffectType::Slicer => &[ParamSpec {
            name: "Depth",
            unit: "",
            min: 0.0,
            max: 1.0,
            default: 1.0,
            curve: Linear,
        }],
        _ => &[],
    }
}

/// Authoritative set of effects that are actually implemented this cycle. Single
/// source of truth for the frontend selector and `is_implemented`.
pub fn implemented_effects() -> &'static [EffectType] {
    &[
        EffectType::Filter,
        EffectType::Isolator,
        EffectType::Delay,
        EffectType::Reverb,
        EffectType::VinylSim,
        EffectType::DjfxLooper,
        EffectType::Scatter,
        EffectType::Slicer,
    ]
}

/// Whether `create_effect` yields a real processing effect for this variant.
pub fn is_implemented(effect_type: EffectType) -> bool {
    implemented_effects().contains(&effect_type)
}

/// Map a frontend effect identifier string to its `EffectType`. Returns `None`
/// for unknown or unimplemented identifiers. Shared by the IPC layer.
pub fn effect_type_from_str(name: &str) -> Option<EffectType> {
    let effect = match name {
        "Isolator" => EffectType::Isolator,
        "DjfxLooper" => EffectType::DjfxLooper,
        "VinylSim" => EffectType::VinylSim,
        "Filter" => EffectType::Filter,
        "Delay" => EffectType::Delay,
        "Reverb" => EffectType::Reverb,
        "Scatter" => EffectType::Scatter,
        "Slicer" => EffectType::Slicer,
        _ => return None,
    };
    Some(effect)
}

/// A populated effect slot: the effect plus its dedicated wet/dry mix amount.
/// `mix` is DATA, not behavior — it never consumes a parameter slot. Default
/// `mix = 1.0` (fully wet) reproduces pre-mix behavior exactly.
pub struct EffectSlot {
    pub effect: Box<dyn Effect>,
    pub mix: f32,
}

pub struct EffectChain {
    pub slots: [Option<EffectSlot>; 4],
}

impl EffectChain {
    pub fn new() -> Self {
        Self {
            slots: [None, None, None, None],
        }
    }

    pub fn process_frame(&mut self, frame: &mut [f32; 2]) {
        for slot in self.slots.iter_mut() {
            if let Some(slot) = slot {
                let dry = *frame;
                slot.effect.process_frame(frame); // frame is now the wet signal
                let mix = slot.mix;
                frame[0] = dry[0] * (1.0 - mix) + frame[0] * mix;
                frame[1] = dry[1] * (1.0 - mix) + frame[1] * mix;
            }
        }
    }

    pub fn set_tempo(&mut self, bpm: f32) {
        for slot in self.slots.iter_mut() {
            if let Some(slot) = slot {
                slot.effect.set_tempo(bpm);
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum EffectType {
    Isolator,
    DjfxLooper,
    VinylSim,
    Filter,
    Delay,
    Reverb,
    Scatter,
    Slicer,
    Chorus, Flanger, Phaser, Tremolo, AutoPan, Compressor, Equalizer, LoFi,
    Bitcrusher, RingMod, PitchShifter, Distortion, Overdrive, Fuzz, Wah,
    Octave, Resonator, TapeEcho, Shimmer, Gater, Reverse, Stutter, TapeStop,
    Compressor2, Equalizer2, Chorus2, Flanger2, Phaser2, Delay2,
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

/// Instantiate an effect. Returns `None` for catalog variants not implemented
/// this cycle — NEVER a silent passthrough. Runs off the audio thread, so
/// allocation here is allowed.
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
            // Reverb gains a controllable damping lowpass after the reverb tail.
            // Default cutoff (18 kHz) is effectively transparent, so the slot's
            // sound is preserved until the user turns the Damping control down.
            let damp = shared(18000.0);
            let damp_ch = || (pass() | var(&damp) | dc(1.0)) >> lowpass();
            let node = reverb_stereo(10.0, 1.0, 0.5) >> (damp_ch() | damp_ch());
            Some(Box::new(FunDspWrapper::new(Box::new(node), vec![damp])))
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
            // Tempo-driven gate with a Depth control. Depth = 1.0 (default) gates
            // fully to silence (original behavior); lower depth raises the floor.
            let bpm = shared(120.0);
            let depth = shared(1.0);
            let b1 = bpm.clone();
            let b2 = bpm.clone();
            let d_a = depth.clone();
            let d_b = depth.clone();
            let d_c = depth.clone();
            let d_d = depth.clone();
            let gate1 = (var(&b1) >> map(|b| b[0] / 60.0 * 4.0)) >> square() >> map(|x| if x[0] > 0.0 { 1.0 } else { 0.0 });
            let gate2 = (var(&b2) >> map(|b| b[0] / 60.0 * 4.0)) >> square() >> map(|x| if x[0] > 0.0 { 1.0 } else { 0.0 });
            let ch1 = pass() * ((dc(1.0) - var(&d_a)) + (var(&d_b) * gate1));
            let ch2 = pass() * ((dc(1.0) - var(&d_c)) + (var(&d_d) * gate2));
            let node = ch1 | ch2;
            Some(Box::new(FunDspWrapper::new_with_tempo(Box::new(node), vec![depth], bpm)))
        }
        EffectType::Slicer => {
            let bpm = shared(120.0);
            let depth = shared(1.0);
            let b1 = bpm.clone();
            let b2 = bpm.clone();
            let d_a = depth.clone();
            let d_b = depth.clone();
            let d_c = depth.clone();
            let d_d = depth.clone();
            let gate1 = (var(&b1) >> map(|b| b[0] / 60.0 * 2.0)) >> square() >> map(|x| if x[0] > 0.0 { 1.0 } else { 0.0 });
            let gate2 = (var(&b2) >> map(|b| b[0] / 60.0 * 2.0)) >> square() >> map(|x| if x[0] > 0.0 { 1.0 } else { 0.0 });
            let ch1 = pass() * ((dc(1.0) - var(&d_a)) + (var(&d_b) * gate1));
            let ch2 = pass() * ((dc(1.0) - var(&d_c)) + (var(&d_d) * gate2));
            let node = ch1 | ch2;
            Some(Box::new(FunDspWrapper::new_with_tempo(Box::new(node), vec![depth], bpm)))
        }
        // Catalog variants not implemented this cycle: explicit None, never a
        // phantom passthrough effect.
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_no_alloc::assert_no_alloc;
    use std::collections::HashSet;

    /// Test-only effect that doubles the input, so the dry/wet blend is easy to
    /// reason about: dry = x, wet = 2x.
    struct DoubleEffect;
    impl Effect for DoubleEffect {
        fn process_frame(&mut self, frame: &mut [f32; 2]) {
            frame[0] *= 2.0;
            frame[1] *= 2.0;
        }
        fn set_parameter(&mut self, _param_id: u8, _value: f32) {}
        fn reset(&mut self) {}
        fn set_sample_rate(&mut self, _rate: u32) {}
    }

    #[test]
    fn test_normalize_linear() {
        let spec = ParamSpec {
            name: "x", unit: "", min: 200.0, max: 8000.0, default: 0.5, curve: Curve::Linear,
        };
        assert!((normalize(&spec, 0.0) - 200.0).abs() < 1e-3);
        assert!((normalize(&spec, 1.0) - 8000.0).abs() < 1e-3);
        // midpoint of a linear range is the arithmetic mean
        assert!((normalize(&spec, 0.5) - 4100.0).abs() < 1e-2);
    }

    #[test]
    fn test_normalize_exponential() {
        let spec = ParamSpec {
            name: "x", unit: "", min: 200.0, max: 8000.0, default: 0.5, curve: Curve::Exponential,
        };
        assert!((normalize(&spec, 0.0) - 200.0).abs() < 1e-3);
        assert!((normalize(&spec, 1.0) - 8000.0).abs() < 1.0);
        let expected_mid = 200.0 * (8000.0_f32 / 200.0).powf(0.5);
        assert!((normalize(&spec, 0.5) - expected_mid).abs() < 1e-2);
    }

    #[test]
    fn test_effect_metadata_validity() {
        for &effect in implemented_effects() {
            let specs = effect_metadata(effect);
            assert!(!specs.is_empty(), "{:?} must declare at least one ParamSpec", effect);
            for spec in specs {
                assert!(spec.min < spec.max, "{:?}.{} requires min < max", effect, spec.name);
                if spec.curve == Curve::Exponential {
                    assert!(spec.min > 0.0, "{:?}.{} exponential curve requires min > 0", effect, spec.name);
                }
            }
        }
    }

    #[test]
    fn test_mix_blend() {
        // mix = 0.0 -> fully dry
        let mut chain = EffectChain::new();
        chain.slots[0] = Some(EffectSlot { effect: Box::new(DoubleEffect), mix: 0.0 });
        let mut frame = [1.0, 1.0];
        chain.process_frame(&mut frame);
        assert_eq!(frame, [1.0, 1.0]);

        // mix = 1.0 -> fully wet
        chain.slots[0] = Some(EffectSlot { effect: Box::new(DoubleEffect), mix: 1.0 });
        let mut frame = [1.0, 1.0];
        chain.process_frame(&mut frame);
        assert_eq!(frame, [2.0, 2.0]);

        // mix = 0.5 -> arithmetic mean of dry (1.0) and wet (2.0)
        chain.slots[0] = Some(EffectSlot { effect: Box::new(DoubleEffect), mix: 0.5 });
        let mut frame = [1.0, 1.0];
        chain.process_frame(&mut frame);
        assert_eq!(frame, [1.5, 1.5]);
    }

    #[test]
    fn test_implemented_effects_set() {
        let got: HashSet<EffectType> = implemented_effects().iter().copied().collect();
        let expected: HashSet<EffectType> = [
            EffectType::Filter,
            EffectType::Isolator,
            EffectType::Delay,
            EffectType::Reverb,
            EffectType::VinylSim,
            EffectType::DjfxLooper,
            EffectType::Scatter,
            EffectType::Slicer,
        ]
        .into_iter()
        .collect();
        assert_eq!(got, expected);
        assert_eq!(implemented_effects().len(), 8);
    }

    #[test]
    fn test_create_effect_some_none() {
        for &effect in implemented_effects() {
            let mut fx = create_effect(effect).expect("implemented effect must instantiate");
            fx.set_sample_rate(44100);
            let mut frame = [0.0, 0.0];
            fx.process_frame(&mut frame); // must not panic
        }
        // representative deferred variants return None, never a passthrough
        assert!(create_effect(EffectType::Chorus).is_none());
        assert!(create_effect(EffectType::PitchShifter).is_none());
    }

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

    #[test]
    fn test_chain_process_frame_no_alloc() {
        // process_frame + slot-level mix blend together must not allocate.
        let mut chain = EffectChain::new();
        let mut fx = create_effect(EffectType::Filter).unwrap();
        fx.set_sample_rate(44100);
        chain.slots[0] = Some(EffectSlot { effect: fx, mix: 0.5 });
        let mut frame = [1.0, -1.0];

        assert_no_alloc(|| {
            chain.process_frame(&mut frame);
        });
    }
}
