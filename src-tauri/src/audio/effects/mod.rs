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
        // PR2: Modulation family. LFO-based effects use a free-running rate in Hz
        // (exponential curve so the low end has fine resolution). Depth/feedback
        // are unitless 0..1 amounts.
        EffectType::Tremolo => &[
            ParamSpec { name: "Rate", unit: "Hz", min: 0.1, max: 20.0, default: 0.4, curve: Exponential },
            ParamSpec { name: "Depth", unit: "", min: 0.0, max: 1.0, default: 0.5, curve: Linear },
        ],
        EffectType::AutoPan => &[
            ParamSpec { name: "Rate", unit: "Hz", min: 0.1, max: 10.0, default: 0.4, curve: Exponential },
            ParamSpec { name: "Depth", unit: "", min: 0.0, max: 1.0, default: 0.7, curve: Linear },
        ],
        EffectType::Chorus => &[
            ParamSpec { name: "Rate", unit: "Hz", min: 0.1, max: 5.0, default: 0.3, curve: Exponential },
            ParamSpec { name: "Depth", unit: "", min: 0.0, max: 1.0, default: 0.5, curve: Linear },
        ],
        EffectType::Flanger => &[
            ParamSpec { name: "Rate", unit: "Hz", min: 0.05, max: 5.0, default: 0.25, curve: Exponential },
            ParamSpec { name: "Depth", unit: "", min: 0.0, max: 1.0, default: 0.6, curve: Linear },
            ParamSpec { name: "Feedback", unit: "", min: 0.0, max: 0.9, default: 0.5, curve: Linear },
        ],
        EffectType::Phaser => &[
            ParamSpec { name: "Rate", unit: "Hz", min: 0.05, max: 4.0, default: 0.3, curve: Exponential },
            ParamSpec { name: "Depth", unit: "", min: 0.0, max: 1.0, default: 0.7, curve: Linear },
        ],
        // PR3: Dynamics + tone family.
        EffectType::Compressor => &[
            ParamSpec { name: "Threshold", unit: "dB", min: -60.0, max: 0.0, default: 0.7, curve: Linear },
            ParamSpec { name: "Ratio", unit: ":1", min: 1.0, max: 20.0, default: 0.2, curve: Linear },
            ParamSpec { name: "Attack", unit: "ms", min: 0.1, max: 100.0, default: 0.2, curve: Exponential },
            ParamSpec { name: "Release", unit: "ms", min: 5.0, max: 1000.0, default: 0.4, curve: Exponential },
        ],
        EffectType::Equalizer => &[
            ParamSpec { name: "Low", unit: "", min: 0.0, max: 2.0, default: 0.5, curve: Linear },
            ParamSpec { name: "Mid", unit: "", min: 0.0, max: 2.0, default: 0.5, curve: Linear },
            ParamSpec { name: "High", unit: "", min: 0.0, max: 2.0, default: 0.5, curve: Linear },
        ],
        EffectType::Wah => &[
            ParamSpec { name: "Freq", unit: "Hz", min: 200.0, max: 3000.0, default: 0.4, curve: Exponential },
            ParamSpec { name: "Resonance", unit: "", min: 0.5, max: 12.0, default: 0.6, curve: Linear },
            ParamSpec { name: "Depth", unit: "", min: 0.0, max: 1.0, default: 1.0, curve: Linear },
        ],
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
        // PR2: Modulation family
        EffectType::Tremolo,
        EffectType::AutoPan,
        EffectType::Chorus,
        EffectType::Flanger,
        EffectType::Phaser,
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
        // PR2: Modulation family
        "Tremolo" => EffectType::Tremolo,
        "AutoPan" => EffectType::AutoPan,
        "Chorus" => EffectType::Chorus,
        "Flanger" => EffectType::Flanger,
        "Phaser" => EffectType::Phaser,
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

/// Hand-written feed-forward compressor. FunDSP's dynamics primitives are weak,
/// so this implements the `Effect` trait directly with pre-allocated per-channel
/// envelope state. `process_frame` performs only scalar FP work — no allocation.
///
/// Signal path per frame: peak detect (max abs across channels) → one-pole
/// envelope follower (attack/release coeffs `exp(-1/(t·sr))`) → soft-knee gain
/// computer (threshold, ratio, fixed knee) → apply the resulting linear gain to
/// both channels. Coeffs are recomputed off the audio thread whenever a time
/// constant or the sample rate changes.
pub struct Compressor {
    sample_rate: f32,
    // Parameters in real units.
    threshold_db: f32,
    ratio: f32,
    attack_ms: f32,
    release_ms: f32,
    // Derived one-pole smoothing coefficients (recomputed on param/SR change).
    attack_coeff: f32,
    release_coeff: f32,
    // Pre-allocated running envelope (linear peak estimate). Shared across the
    // stereo pair so both channels get identical gain (no image shift).
    envelope: f32,
    // Fixed soft-knee width in dB.
    knee_db: f32,
}

impl Compressor {
    /// One-pole smoothing coefficient for a time constant `t_ms` (milliseconds)
    /// at `sample_rate` Hz: `exp(-1 / (t_seconds * sample_rate))`. Pure scalar FP,
    /// safe to call off the audio thread. `t_ms == 0` collapses to instantaneous.
    pub fn envelope_coeff(t_ms: f32, sample_rate: f32) -> f32 {
        let t_seconds = t_ms / 1000.0;
        if t_seconds <= 0.0 || sample_rate <= 0.0 {
            return 0.0;
        }
        (-1.0 / (t_seconds * sample_rate)).exp()
    }

    pub fn new(sample_rate: u32) -> Self {
        let md = effect_metadata(EffectType::Compressor);
        let mut c = Self {
            sample_rate: sample_rate as f32,
            threshold_db: normalize(&md[0], md[0].default),
            ratio: normalize(&md[1], md[1].default),
            attack_ms: normalize(&md[2], md[2].default),
            release_ms: normalize(&md[3], md[3].default),
            attack_coeff: 0.0,
            release_coeff: 0.0,
            envelope: 0.0,
            knee_db: 6.0,
        };
        c.recompute_coeffs();
        c
    }

    fn recompute_coeffs(&mut self) {
        self.attack_coeff = Self::envelope_coeff(self.attack_ms, self.sample_rate);
        self.release_coeff = Self::envelope_coeff(self.release_ms, self.sample_rate);
    }
}

impl Effect for Compressor {
    fn process_frame(&mut self, frame: &mut [f32; 2]) {
        // Peak detect: rectified max across the stereo pair.
        let peak = frame[0].abs().max(frame[1].abs());
        // One-pole envelope follower: attack when rising, release when falling.
        let coeff = if peak > self.envelope { self.attack_coeff } else { self.release_coeff };
        self.envelope = coeff * self.envelope + (1.0 - coeff) * peak;

        // Convert envelope to dBFS (guard against log of zero).
        let env_db = 20.0 * (self.envelope.max(1e-9)).log10();

        // Soft-knee gain computer: how many dB to reduce the output.
        let half_knee = self.knee_db * 0.5;
        let over = env_db - self.threshold_db;
        let gain_reduction_db = if over <= -half_knee {
            0.0
        } else if over >= half_knee {
            over * (1.0 - 1.0 / self.ratio)
        } else {
            // Quadratic interpolation across the knee region.
            let x = over + half_knee; // 0..knee_db
            (1.0 - 1.0 / self.ratio) * (x * x) / (2.0 * self.knee_db)
        };

        let gain = 10.0_f32.powf(-gain_reduction_db / 20.0);
        frame[0] *= gain;
        frame[1] *= gain;
    }

    fn set_parameter(&mut self, param_id: u8, value: f32) {
        match param_id {
            0 => self.threshold_db = value,
            1 => self.ratio = value.max(1.0),
            2 => {
                self.attack_ms = value;
                self.recompute_coeffs();
            }
            3 => {
                self.release_ms = value;
                self.recompute_coeffs();
            }
            _ => {}
        }
    }

    fn reset(&mut self) {
        self.envelope = 0.0;
    }

    fn set_sample_rate(&mut self, rate: u32) {
        self.sample_rate = rate as f32;
        self.recompute_coeffs();
    }
}

/// A phase-0 sine LFO source. Unlike `sine()`, whose start phase is seeded from
/// the node hash (`rnd1(hash)`), this pins the initial phase so that independent
/// same-rate LFOs (stereo channels, cascaded Phaser stages) stay phase-coherent.
fn lfo_sine() -> An<Sine> {
    An(Sine::with_phase(0.0))
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
        // ── PR2: Modulation family ──────────────────────────────────────────
        // A controllable LFO is `var(&rate) >> lfo_sine()` (sine in -1..1 at
        // `rate` Hz). `lfo01 = (sine + 1) * 0.5` maps it to 0..1. Each subgraph
        // needs its own `var` clone because `var(&x)` consumes the borrow per use.
        // `lfo_sine()` pins the start phase to 0 so that same-rate LFOs across
        // channels/stages stay phase-coherent — bare `sine()` seeds its phase from
        // its node hash (`rnd1(hash)`), which would decohere stereo modulation (and
        // Phaser stage alignment) the moment anything seeds the graph via `ping()`.
        // Stereo coherence (Tremolo, AutoPan) and notch alignment (Phaser) depend
        // on this; do NOT replace `lfo_sine()` with `sine()`.
        EffectType::Tremolo => {
            // Amplitude modulation: gain swings between (1 - depth) and 1.
            // gain = (1 - depth) + depth * lfo01  (matches the Scatter gate shape).
            let rate = shared(normalize(&effect_metadata(EffectType::Tremolo)[0], 0.4));
            let depth = shared(0.5);
            let (r1, r2) = (rate.clone(), rate.clone());
            let (d1a, d1b, d2a, d2b) = (depth.clone(), depth.clone(), depth.clone(), depth.clone());
            let lfo1 = ((var(&r1) >> lfo_sine()) + 1.0) * 0.5;
            let lfo2 = ((var(&r2) >> lfo_sine()) + 1.0) * 0.5;
            let ch1 = pass() * ((dc(1.0) - var(&d1a)) + (var(&d1b) * lfo1));
            let ch2 = pass() * ((dc(1.0) - var(&d2a)) + (var(&d2b) * lfo2));
            let node = ch1 | ch2;
            Some(Box::new(FunDspWrapper::new(Box::new(node), vec![rate, depth])))
        }
        EffectType::AutoPan => {
            // Opposed gains swept by the LFO: when sine=+1 the right channel is
            // full and the left drops by `depth`; sine=-1 mirrors it. Center
            // (sine=0) attenuates both by depth*0.5 (standard linear pan dip).
            let rate = shared(normalize(&effect_metadata(EffectType::AutoPan)[0], 0.4));
            let depth = shared(0.7);
            let (r_l, r_r) = (rate.clone(), rate.clone());
            let (d_l, d_r) = (depth.clone(), depth.clone());
            // left_amount = (sine + 1)/2 ; right_amount = (1 - sine)/2 ; both 0..1
            let left_amt = ((var(&r_l) >> lfo_sine()) + 1.0) * 0.5;
            let right_amt = (dc(1.0) - (var(&r_r) >> lfo_sine())) * 0.5;
            let ch_l = pass() * (dc(1.0) - (var(&d_l) * left_amt));
            let ch_r = pass() * (dc(1.0) - (var(&d_r) * right_amt));
            let node = ch_l | ch_r;
            Some(Box::new(FunDspWrapper::new(Box::new(node), vec![rate, depth])))
        }
        EffectType::Chorus => {
            // Dry + a single voice read from a delay line whose time is swept
            // ~15..25 ms by the LFO. `tap` reads input1 (seconds) from the line.
            let rate = shared(normalize(&effect_metadata(EffectType::Chorus)[0], 0.3));
            let depth = shared(0.5);
            let chorus_ch = |r: Shared, d: Shared| {
                let lfo01 = ((var(&r) >> lfo_sine()) + 1.0) * 0.5;
                // delay_time = 0.015 + lfo01 * depth * 0.010  (seconds)
                let delay_time = dc(0.015) + (lfo01 * var(&d) * 0.010);
                pass() & ((pass() | delay_time) >> tap(0.005, 0.040))
            };
            let ch1 = chorus_ch(rate.clone(), depth.clone());
            let ch2 = chorus_ch(rate.clone(), depth.clone());
            let node = ch1 | ch2;
            Some(Box::new(FunDspWrapper::new(Box::new(node), vec![rate, depth])))
        }
        EffectType::Flanger => {
            // Very short (~1..5 ms) modulated delay with feedback. `feedback(x)`
            // sums x's previous output back into its input; x here is the
            // modulated tap scaled by the feedback amount.
            let rate = shared(normalize(&effect_metadata(EffectType::Flanger)[0], 0.25));
            let depth = shared(0.6);
            let fb = shared(0.5);
            let flanger_ch = |r: Shared, d: Shared, f: Shared| {
                let lfo01 = ((var(&r) >> lfo_sine()) + 1.0) * 0.5;
                // delay_time = 0.001 + lfo01 * depth * 0.004  (seconds)
                let delay_time = dc(0.001) + (lfo01 * var(&d) * 0.004);
                let loop_node = ((pass() | delay_time) >> tap(0.0005, 0.010)) * var(&f);
                pass() & feedback(loop_node)
            };
            let ch1 = flanger_ch(rate.clone(), depth.clone(), fb.clone());
            let ch2 = flanger_ch(rate.clone(), depth.clone(), fb.clone());
            let node = ch1 | ch2;
            Some(Box::new(FunDspWrapper::new(Box::new(node), vec![rate, depth, fb])))
        }
        EffectType::Phaser => {
            // Dry + a cascade of 4 swept allpass stages. fundsp's `allpass()` SVF
            // takes (audio, center-freq Hz, Q); sweeping the center frequency with
            // the LFO moves the notches. Center sweeps 300..(300 + depth*1500) Hz —
            // i.e. up to 1800 Hz at depth=1.0, ~1350 Hz at the default depth 0.7.
            let rate = shared(normalize(&effect_metadata(EffectType::Phaser)[0], 0.3));
            let depth = shared(0.7);
            // One allpass stage fed by the shared LFO (its own var clones).
            let stage = |r: Shared, d: Shared| {
                let lfo01 = ((var(&r) >> lfo_sine()) + 1.0) * 0.5;
                // center = 300 + lfo01 * depth * 1500  (Hz)
                let center = dc(300.0) + (lfo01 * var(&d) * 1500.0);
                (pass() | center | dc(0.7)) >> allpass()
            };
            let phaser_ch = |r: &Shared, d: &Shared| {
                let cascade = stage(r.clone(), d.clone())
                    >> stage(r.clone(), d.clone())
                    >> stage(r.clone(), d.clone())
                    >> stage(r.clone(), d.clone());
                pass() & cascade
            };
            let node = phaser_ch(&rate, &depth) | phaser_ch(&rate, &depth);
            Some(Box::new(FunDspWrapper::new(Box::new(node), vec![rate, depth])))
        }
        // ── PR3: Dynamics + tone family ──────────────────────────────────────
        EffectType::Compressor => Some(Box::new(Compressor::new(44100))),
        EffectType::Equalizer => {
            // Three-band tone shaper reusing the proven Isolator shelf/bell
            // cascade, but centered on musical EQ bands (low shelf 120 Hz, mid
            // bell 1.2 kHz, high shelf 6 kHz). Each band's gain is a 0..2 linear
            // multiplier (1.0 = flat), driven by a Shared param.
            let low = shared(1.0);
            let mid = shared(1.0);
            let high = shared(1.0);
            let eq_ch = || (pass() | dc(120.0) | dc(1.0) | var(&low)) >> lowshelf()
                         >> (pass() | dc(1200.0) | dc(1.0) | var(&mid)) >> bell()
                         >> (pass() | dc(6000.0) | dc(1.0) | var(&high)) >> highshelf();
            let node = eq_ch() | eq_ch();
            Some(Box::new(FunDspWrapper::new(Box::new(node), vec![low, mid, high])))
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
            EffectType::Tremolo,
            EffectType::AutoPan,
            EffectType::Chorus,
            EffectType::Flanger,
            EffectType::Phaser,
        ]
        .into_iter()
        .collect();
        assert_eq!(got, expected);
        assert_eq!(implemented_effects().len(), 13);
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
        assert!(create_effect(EffectType::RingMod).is_none());
        assert!(create_effect(EffectType::PitchShifter).is_none());
    }

    #[test]
    fn test_modulation_effects_no_alloc() {
        // PR2: every modulation effect must process a frame without allocating.
        for &eff in &[
            EffectType::Tremolo,
            EffectType::AutoPan,
            EffectType::Chorus,
            EffectType::Flanger,
            EffectType::Phaser,
        ] {
            let mut fx = create_effect(eff).expect("modulation effect must instantiate");
            fx.set_sample_rate(44100);
            let mut frame = [0.5, -0.5];
            assert_no_alloc(|| {
                fx.process_frame(&mut frame);
            });
        }
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
            EffectType::Tremolo,
            EffectType::AutoPan,
            EffectType::Chorus,
            EffectType::Flanger,
            EffectType::Phaser,
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

    // ── PR3: Dynamics + tone family ─────────────────────────────────────────

    /// Drive a compressor to steady state with a constant-amplitude signal and
    /// return the linear gain it applies (output / input) once the envelope has
    /// settled. Used to assert gain-reduction monotonicity.
    fn compressor_settled_gain(threshold_db: f32, ratio: f32, amp: f32) -> f32 {
        let mut c = Compressor::new(48000);
        c.set_parameter(0, threshold_db);
        c.set_parameter(1, ratio);
        c.set_parameter(2, 5.0); // attack ms
        c.set_parameter(3, 50.0); // release ms
        let mut out = [0.0_f32; 2];
        for _ in 0..20000 {
            out = [amp, amp];
            c.process_frame(&mut out);
        }
        out[0] / amp
    }

    #[test]
    fn test_compressor_envelope_coeff() {
        // coeff = exp(-1 / (t_seconds * sample_rate)); t_seconds = ms / 1000.
        let sr = 48000.0;
        let coeff = Compressor::envelope_coeff(10.0, sr);
        let expected = (-1.0_f32 / (0.010 * sr)).exp();
        assert!((coeff - expected).abs() < 1e-6, "got {coeff}, expected {expected}");
        // Longer time constant -> coeff closer to 1 (slower response).
        let slow = Compressor::envelope_coeff(100.0, sr);
        let fast = Compressor::envelope_coeff(1.0, sr);
        assert!(slow > coeff && coeff > fast, "coeffs must increase with time");
        assert!(slow < 1.0 && fast > 0.0);
    }

    #[test]
    fn test_compressor_gain_reduction_monotonic() {
        // Both inputs are above the -20 dB threshold. The louder input is further
        // over the threshold, so it MUST receive more gain reduction (smaller gain).
        let quiet = compressor_settled_gain(-20.0, 4.0, 0.3);
        let loud = compressor_settled_gain(-20.0, 4.0, 0.6);
        assert!(quiet <= 1.0 + 1e-4, "gain must never exceed unity, got {quiet}");
        assert!(loud < quiet, "louder input must be reduced more: loud={loud}, quiet={quiet}");
        // A signal below threshold must pass at (approximately) unity gain.
        let below = compressor_settled_gain(-20.0, 4.0, 0.02); // -34 dB, under threshold
        assert!((below - 1.0).abs() < 1e-2, "below-threshold gain must be ~unity, got {below}");
    }

    #[test]
    fn test_compressor_no_alloc() {
        let mut fx = create_effect(EffectType::Compressor).expect("Compressor must instantiate");
        fx.set_sample_rate(44100);
        let mut frame = [0.7, -0.7];
        assert_no_alloc(|| {
            fx.process_frame(&mut frame);
        });
    }

    #[test]
    fn test_equalizer_instantiates_and_processes() {
        let mut fx = create_effect(EffectType::Equalizer).expect("Equalizer must instantiate");
        fx.set_sample_rate(48000);
        // At the default (flat) settings a non-zero input must yield a finite,
        // non-silent output — proves the cascade is wired and not a no-op/NaN.
        let mut frame = [0.5, -0.5];
        for _ in 0..64 {
            frame = [0.5, -0.5];
            fx.process_frame(&mut frame);
        }
        assert!(frame[0].is_finite() && frame[1].is_finite());
        assert!(frame[0].abs() > 1e-3, "flat EQ must pass signal, got {}", frame[0]);
    }

    #[test]
    fn test_equalizer_no_alloc() {
        let mut fx = create_effect(EffectType::Equalizer).expect("Equalizer must instantiate");
        fx.set_sample_rate(44100);
        let mut frame = [0.5, -0.5];
        assert_no_alloc(|| {
            fx.process_frame(&mut frame);
        });
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
