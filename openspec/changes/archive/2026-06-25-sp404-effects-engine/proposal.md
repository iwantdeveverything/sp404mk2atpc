# Proposal: SP-404MK2 Full Effects Engine

## Intent

The audio engine has a placeholder at `engine.rs:138-145` where Bus1/Bus2/Master FX processing should occur. Without it, bus routing is cosmetic — audio passes through unprocessed. This change implements all 37 MFX effects matching the real SP-404MK2, transforming the DAW from a sample player into a production instrument.

## Scope

### In Scope
- `Effect` trait + `EffectChain` (4 slots per bus) for Bus1, Bus2, Master
- 37 effects across 9 categories via FunDSP hybrid (fundsp primitives + custom Rust for VinylSim, Scatter, DJFX Looper, Cassette Sim)
- Lock-free parameter control extending existing `AudioCommand` enum via `rtrb`
- Per-effect wet/dry mix control
- Per-pad persistent bus routing (pad → bus assignment survives triggers)
- BPM source: manual input + tap tempo for beat-synced effects
- Effect config persistence to disk (survive app restart)
- Frontend: effect selector, 3 rotary knobs (CTRL 1-3), bus target indicator
- 5-phase implementation rollout

### Out of Scope
- Firmware-update effects (SX Reverb, Vocoder, Auto Pitch, etc.) — future phase
- Input FX (microphone/line-in processing)
- MIDI CC mapping for effect parameters
- Block-based SIMD processing (design for it, implement later)
- Separate parameter ring buffer (Option B) — only if profiling shows contention

## Capabilities

### New Capabilities
- `effects-engine`: Effect trait, EffectChain, effect factory, FunDSP bridge, all 37 effect implementations, per-effect wet/dry, preset definitions
- `effects-persistence`: Serialize/deserialize effect chain configs to disk, restore on app start
- `bpm-sync`: Manual BPM input + tap tempo, tempo distribution to beat-synced effects (Scatter, Slicer, Sync Delay, Ko-Da-Ma, DJFX Looper)

### Modified Capabilities
- `audio-core`: New `AudioCommand` variants (SetBusEffect, SetEffectParam, RemoveBusEffect), `BusId` enum, FX chain fields on thread state, FX processing integration at placeholder, per-pad persistent routing
- `ui-routing`: Effect selector UI in LCD area, rotary knob controls, bus-to-effect mapping, BPM input/tap tempo controls

## Approach

FunDSP Hybrid (Approach B from exploration): `fundsp` crate for core DSP primitives (filters, delays, reverbs, modulation, dynamics) + custom Rust for SP-404-specific effects requiring beat-sync, noise modeling, or buffer capture. Bridge via `FunDspEffect` wrapper adapting `AudioNode` to our `Effect` trait. Per-frame processing matching current architecture.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src-tauri/src/audio/effects/` | New | Effect trait, chain, factory, bridge, 37 effect implementations |
| `src-tauri/src/audio/engine.rs` | Modified | FX chain fields, FX processing at L138-145, new command handling |
| `src-tauri/src/audio/state.rs` | Modified | New AudioCommand variants, BusId/EffectType enums, persistent routing |
| `src-tauri/src/audio/mod.rs` | Modified | `pub mod effects;` declaration |
| `src-tauri/src/lib.rs` | Modified | New Tauri commands: set_bus_effect, set_effect_param, get_effect_list |
| `src-tauri/Cargo.toml` | Modified | Add `fundsp` + serialization deps |
| `src/main.ts` | Modified | Effect selection UI, knob controls, BPM input |
| `src/styles.css` | Modified | Knob, effect selector, parameter display styles |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Audio glitches from FX CPU cost | Medium | Per-frame budget monitoring, dry fallback on overrun |
| Ring buffer overflow from knob twiddling | Low | Rate-limit UI sends to ~60Hz, 1024 capacity sufficient |
| Memory allocation in audio thread | High | Pre-allocate ALL state at construction, `assert_no_alloc` in debug |
| FunDSP API changes | Low | Pin version, isolate behind Effect trait |

## Rollback Plan

Revert `fundsp` dependency and effect module additions. The placeholder at `engine.rs:138-145` remains functional — audio passes through unprocessed as before. No data migration needed.

## Dependencies

- `fundsp = "0.19"` (or latest stable)
- Serialization crate for effect persistence (serde already in tree via Tauri)

## Success Criteria

- [ ] All 37 effects process audio without glitches at 44.1kHz/48kHz
- [ ] Effect chains (4 slots) work on Bus1, Bus2, and Master independently
- [ ] Parameter changes from UI reach audio thread with no audible latency
- [ ] Effect configurations persist across app restarts
- [ ] Zero allocations in audio callback (verified with assert_no_alloc)
- [ ] Tap tempo and manual BPM correctly sync beat-dependent effects
