# Design: SP-404MK2 Effects Engine Robustness

## 1. Technical Approach

Three cross-cutting layers are added around the existing FunDSP-hybrid `Effect`
pipeline without rewriting it: **(1) static parameter metadata**, **(2) an
off-audio-thread normalization boundary**, and **(3) a slot-level wet/dry blend**.
Effects stay focused on producing the WET signal only. The audio thread keeps doing
pure scalar writes — every metadata lookup and `0–1 → real-range` curve evaluation
happens on the IPC/UI thread, preserving the `rtrb` + `assert_no_alloc` discipline.
`create_effect` returns explicit `None` for unimplemented variants; a single
`implemented_effects()` accessor is the source of truth for the selector.

## 2. Architecture Decisions

| Decision | Options | Decision & Rationale |
|---|---|---|
| **Metadata attachment** | (a) `fn parameters()` on `Effect` trait · (b) standalone `effect_metadata(EffectType) -> &'static [ParamSpec]` registry · (c) return metadata from `create_effect` | **(b)**. Metadata is static and must be queryable WITHOUT instantiating an effect (selector listing, `None` variants, UI render before audio swap). A `&'static` slice = zero alloc, single-sourced, and keeps the realtime `Effect` trait free of descriptor concerns. Trait method (a) forces instantiation; (c) couples query to allocation. |
| **Normalization placement** | (a) engine before `set_parameter` · (b) inside effect from its own metadata · (c) at IPC boundary, OFF audio thread | **(c)**. The knob's `0–1` is mapped to the real range (incl. `powf` for exponential curves) in `AudioState::set_effect_param` / engine-init, then the **already-normalized real value** is pushed through `AudioCommand`. The audio thread only does `Shared::set_value(real)` — no metadata lookup, no curve math, no alloc under `assert_no_alloc`. (a) puts a registry lookup on the callback; (b) bloats every node. |
| **Wet/dry mix** | (a) `MixWrapper: Box<dyn Effect>` wrapping the inner effect · (b) `EffectSlot { effect, mix: f32 }` struct, blend in `EffectChain` | **(b)**. Mix is DATA, not behavior — a dedicated always-present control that must NOT consume a param slot. A plain `f32` field avoids double dynamic dispatch + extra heap box, and the blend is pure arithmetic in `EffectChain::process_frame`. Default `mix = 1.0` (fully wet) reproduces the existing 8 effects exactly — no regression. |
| **Compressor / Wah / Bitcrusher** | FunDSP graph · custom Rust `Effect` impl | **Custom Rust**. FunDSP dynamics/sample-rate-reduction are weak; these get hand-written structs implementing `Effect` directly with pre-allocated per-channel state. |

## 3. Data Flow

```ascii
 Knob (0..1)                              [UI thread]
     │  invoke set_effect_param / set_effect_mix / get_effect_parameters
     ▼
 lib.rs (Tauri cmd)  ── get_effect_parameters / list_effects ──► effect_metadata()   (no audio thread)
     │ normalize(spec, t) -> real          [OFF audio thread]
     ▼
 AudioState ── push AudioCommand{ real value | mix } ─► rtrb ring buffer
                                                            │  (lock-free)
                                                            ▼
                              [AUDIO THREAD]  write_data: pop command
                                 SetEffectParam → Shared::set_value(real)
                                 SetEffectMix   → slot.mix = mix
                                                            │
                          EffectChain::process_frame:  dry = *frame
                                 effect.process_frame(frame)        // wet
                                 frame = dry*(1-mix) + wet*mix      // blend
```

## 4. Interfaces & Contracts

```rust
// effects/mod.rs — static registry (off audio thread)
pub enum Curve { Linear, Exponential }
pub struct ParamSpec {
    pub name: &'static str,   // e.g. "Cutoff"
    pub unit: &'static str,   // e.g. "Hz", "ms", "dB", "%"
    pub min: f32, pub max: f32,
    pub default: f32,         // NORMALIZED 0..1
    pub curve: Curve,
}
pub fn effect_metadata(e: EffectType) -> &'static [ParamSpec];
pub fn is_implemented(e: EffectType) -> bool;       // = create_effect(e).is_some()
pub fn implemented_effects() -> &'static [EffectType]; // selector source of truth

/// Pure, unit-testable. Linear: min + t*(max-min).
/// Exponential (min>0): min * (max/min).powf(t).
pub fn normalize(spec: &ParamSpec, t: f32) -> f32;

// Slot-level mix (audio thread blend, no alloc)
pub struct EffectSlot { pub effect: Box<dyn Effect>, pub mix: f32 } // mix default 1.0
pub struct EffectChain { pub slots: [Option<EffectSlot>; 4] }

// state.rs — new command; SetEffectParam.value now carries the REAL value
pub enum AudioCommand {
    /* …existing… */
    SetEffectMix { bus: BusRouting, slot: usize, mix: f32 },
}

// state.rs — persisted config gains mix + variable params
pub struct EffectSlotConfig {
    pub effect_type: EffectType,
    #[serde(default)] pub params: Vec<f32>, // NORMALIZED 0..1, len = metadata.len()
    #[serde(default = "one")] pub mix: f32, // 0..1
}
```

`Effect` trait is UNCHANGED. `create_effect`'s `_ => Some(pass())` arm becomes
`_ => None`. **Effects are built OFF the audio thread** (per the Zero Allocation
Processing requirement): `AudioState::set_bus_effect` calls `create_effect` +
`set_sample_rate(real)` + `set_tempo` + normalized default params, then enqueues a
`SetBusEffect { slot_fx: Box<EffectSlot> }` carrying the ready effect. An
unimplemented variant yields `None` → no command enqueued (graceful no-op, never
clears a live slot). The audio thread only *moves* the ready slot into place and
hands the previous slot to a **retire ring** (drained by a dedicated `fx-retire`
thread) so neither allocation nor deallocation runs in the cpal callback. The real
device sample rate is published into `AudioState` on engine startup.
`lib.rs` `set_bus_effect` string map extends to all 20 implemented variants; unknown
strings still `Err`. New Tauri commands: `list_effects()` and
`get_effect_parameters(effect) -> Vec<ParamSpecDto>` (pure, never enqueue a command).

## 5. Per-Effect DSP Notes

| Effect | Approach |
|---|---|
| **Compressor** (HIGH) | Custom `Effect` struct. Pre-allocated per-channel envelope `f32`. `process_frame`: peak detect → one-pole envelope follower (attack/release coeffs `exp(-1/(t·sr))`, recomputed on `set_parameter`/`set_sample_rate` — scalar FP, alloc-free) → soft-knee gain computer (threshold, ratio, knee) → apply gain. Params: Threshold(dB), Ratio, Attack(ms,exp), Release(ms,exp). |
| **Wah** | Custom resonant state-variable bandpass; center freq swept by the param (LFO/envelope optional later). Pre-allocated SVF state. Params: Freq(Hz,exp), Resonance, Mix-depth. |
| **Bitcrusher** | Custom sample-and-hold decimator + bit quantizer. State: hold counter + held sample/channel. Params: SampleRate(Hz,exp downsample), Bits(lin). |
| **LoFi** | Composite: bitcrush + `lowpass_hz` bandlimit; reuse Bitcrusher + FunDSP filter. |
| **Tremolo / AutoPan** | FunDSP LFO amplitude/pan mod (`lfo`/`sine_hz`). Trivial. |
| **Chorus / Flanger** | FunDSP modulated short delay (`var(lfo) >> tap`); Flanger adds feedback. |
| **Phaser** | FunDSP cascaded `allpole`/allpass swept by LFO. |
| **Distortion / Overdrive** | FunDSP waveshaping (`shape`/`clip`, soft-clip `tanh`). |
| **Equalizer** | FunDSP cascaded `bell`/`shelf` — reuses the proven Isolator pattern. |

## 6. Testing Strategy (strict TDD — `cargo test`)

| Layer | What | Approach |
|---|---|---|
| Unit | `normalize()` linear + exponential math, endpoints (t=0→min, t=1→max), exp midpoint | Table-driven asserts on pure fn |
| Unit | `effect_metadata()` correctness: every implemented effect has ≥1 spec; `min<max`; exp curves have `min>0` | Per-variant assertions |
| Unit | Mix blend: `mix=0`→dry, `mix=1`→wet, `0.5`→mean | Stub effect, assert frame |
| Unit | `is_implemented`/`implemented_effects()` = exactly the 20 batch+existing | Set equality test |
| Unit | Per-effect instantiation: `create_effect` Some for batch, `None` for deferred; `process_frame` doesn't panic | Extend `test_effect_instantiations` |
| Unit | No-alloc: `process_frame` + mix blend + Compressor under `assert_no_alloc` | Extend `test_process_frame_no_alloc` |
| Integration | `SetEffectMix` / `SetEffectParam(real)` traverse rtrb and mutate slot | Extend `test_ring_buffer_commands` |
| Manual | Musical sweep feel, Compressor pumping, glitch-free at 44.1/48k | Audible verification |

## 7. Migration & Rollout

`EffectSlotConfig` gains `#[serde(default)]` `mix` (→1.0) and `Vec<f32>` params, so
existing `fx_config.json` deserializes unchanged. Additive only — no data migration.
Rollout follows the proposal's chained PRs: **PR1 foundation** (metadata,
normalization, mix, `None`, IPC, selector filter), then modulation / dynamics+tone /
drive+lofi families. Reverting PR1 restores prior fully-wet behavior.

## 8. Open Questions

- [ ] Confirm **mix default = 1.0** (fully wet) to preserve current behavior, vs 0.5
      as a more conventional initial blend. (Design assumes 1.0 for zero regression.)
- [ ] `get_effect_parameters` returns specs with `default` normalized — confirm UI
      seeds knobs from `default` on effect select (vs. resetting to 0.5).
