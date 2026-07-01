## PR 2: Modulation family (Chorus, Flanger, Phaser, Tremolo, AutoPan)
*Target branch: `feature/sp404-effects-engine-robustness-2-modulation` (stacked on PR 1)*
*Satisfies: effects-engine (Effect Inventory, Parameter Metadata Contract, Zero Allocation Processing); ui-routing (Effect Selector UI).*

- [x] `src-tauri/src/audio/effects/mod.rs` - Write failing instantiation + no-alloc tests (`assert_no_alloc` on `process_frame`) for Tremolo (LFO amplitude mod); implement via fundsp graph; add `effect_metadata` (2 params); register in `create_effect`/`implemented_effects`; make green. [Effect Inventory, Parameter Metadata Contract, Zero Allocation Processing]
- [x] `src-tauri/src/audio/effects/mod.rs` - Write failing tests for AutoPan (LFO pan mod); implement fundsp graph; add metadata; register; make green. [Effect Inventory, Parameter Metadata Contract]
- [x] `src-tauri/src/audio/effects/mod.rs` - Write failing tests for Chorus (modulated short delay `var(lfo) >> tap`); implement; add metadata; register; make green. [Effect Inventory, Parameter Metadata Contract]
- [x] `src-tauri/src/audio/effects/mod.rs` - Write failing tests for Flanger (modulated very-short delay + feedback); implement; add metadata; register; make green. [Effect Inventory, Parameter Metadata Contract]
- [x] `src-tauri/src/audio/effects/mod.rs` - Write failing tests for Phaser (cascaded allpass/allpole swept by LFO); implement; add metadata; register; make green. [Effect Inventory, Parameter Metadata Contract]
- [x] `src-tauri/src/lib.rs` - Extend the `set_bus_effect` string→`EffectType` mapping with Chorus, Flanger, Phaser, Tremolo, AutoPan. [Effect Selector UI]
- [x] `src-tauri/src/audio/effects/mod.rs` - Update the `implemented_effects()` set-equality test to include the 5 modulation effects (13 total). [Implemented-Only Selection]

## PR 3: Dynamics + tone family (Compressor, Equalizer, Wah)
*Target branch: `feature/sp404-effects-engine-robustness-3-dynamics` (stacked on PR 2)*
*Satisfies: effects-engine (Effect Inventory, Parameter Metadata Contract, Zero Allocation Processing); ui-routing (Effect Selector UI). HIGHEST-RISK group: custom Rust DSP.*

- [x] `src-tauri/src/audio/effects/mod.rs` - Write failing tests for Compressor custom struct: envelope-coeff math (`exp(-1/(t·sr))`), gain-reduction monotonicity (more input over threshold → more reduction), and `assert_no_alloc` on `process_frame`; implement custom `Effect` (pre-allocated per-channel envelope `f32`, peak detect → one-pole follower → soft-knee gain computer); coeffs recomputed on `set_parameter`/`set_sample_rate` (scalar, alloc-free); add metadata (Threshold dB, Ratio, Attack ms exp, Release ms exp); register; make green. [Effect Inventory, Parameter Metadata Contract, Zero Allocation Processing]
- [x] `src-tauri/src/audio/effects/mod.rs` - Write failing tests for Equalizer (cascaded bell/shelf, reuse Isolator pattern); implement fundsp cascade; add metadata; register; make green. [Effect Inventory, Parameter Metadata Contract]
- [x] `src-tauri/src/audio/effects/mod.rs` - Write failing tests for Wah custom resonant SVF bandpass (pre-allocated state, center freq from param) incl. no-alloc; implement custom `Effect`; add metadata (Freq Hz exp, Resonance, Mix-depth); register; make green. [Effect Inventory, Parameter Metadata Contract, Zero Allocation Processing]
- [x] `src-tauri/src/lib.rs` - Extend the `set_bus_effect` mapping with Compressor, Equalizer, Wah. [Effect Selector UI]
- [x] `src-tauri/src/audio/effects/mod.rs` - Update the `implemented_effects()` set-equality test to include the 3 dynamics/tone effects (16 total). [Implemented-Only Selection]

## PR 4: Drive + lo-fi family (Distortion, Overdrive, Bitcrusher, LoFi)
*Target branch: `feature/sp404-effects-engine-robustness-4-drive` (stacked on PR 3)*
*Satisfies: effects-engine (Effect Inventory, Parameter Metadata Contract, Zero Allocation Processing); ui-routing (Effect Selector UI).*

- [ ] `src-tauri/src/audio/effects/mod.rs` - Write failing tests for Distortion (fundsp waveshaping `shape`/`clip`); implement; add metadata; register; make green. [Effect Inventory, Parameter Metadata Contract]
- [ ] `src-tauri/src/audio/effects/mod.rs` - Write failing tests for Overdrive (soft-clip `tanh` waveshaping); implement; add metadata; register; make green. [Effect Inventory, Parameter Metadata Contract]
- [ ] `src-tauri/src/audio/effects/mod.rs` - Write failing tests for Bitcrusher custom sample-and-hold decimator + bit quantizer (state: hold counter + held sample/channel) incl. no-alloc; implement custom `Effect`; add metadata (SampleRate Hz exp downsample, Bits linear); register; make green. [Effect Inventory, Parameter Metadata Contract, Zero Allocation Processing]
- [ ] `src-tauri/src/audio/effects/mod.rs` - Write failing tests for LoFi composite (Bitcrusher + fundsp `lowpass_hz` bandlimit); implement reusing Bitcrusher; add metadata; register; make green. [Effect Inventory, Parameter Metadata Contract]
- [ ] `src-tauri/src/lib.rs` - Extend the `set_bus_effect` mapping with Distortion, Overdrive, Bitcrusher, LoFi. [Effect Selector UI]
- [ ] `src-tauri/src/audio/effects/mod.rs` - Update the final `implemented_effects()` set-equality test to the full 20 (existing 8 + new 12). [Implemented-Only Selection, Effect Inventory]
