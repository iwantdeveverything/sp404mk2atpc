# Tasks: SP-404MK2 Full Effects Engine

**Review Workload Forecast**: This will be a massive change (>400 lines). Risk is High. Chained PRs recommended: Yes. Delivery strategy: ask-on-risk. Chain strategy: stacked-to-main. Decision needed before apply: No.

## Phase 1: Infrastructure
- [x] `src-tauri/Cargo.toml` - Add `fundsp` and serialization dependencies.
- [x] `src-tauri/src/audio/effects/mod.rs` - Define `Effect` trait, `EffectType` enum, and `EffectChain` struct.
- [x] `src-tauri/src/audio/state.rs` - Add `SetBusEffect`, `SetEffectParam`, `RemoveBusEffect`, and `SetTempo` to `AudioCommand`.
- [x] `src-tauri/src/audio/engine.rs` - Add FX chain fields to `AudioEngineThreadState` and process at L138-145 placeholder.
- [x] `src-tauri/src/audio/mod.rs` - Declare `pub mod effects`.
- [x] `src-tauri/src/lib.rs` - Add Tauri commands `set_bus_effect` and `set_effect_param`.

## Phase 2: Core Effects
- [x] `src-tauri/src/audio/effects/mod.rs` - Implement `FunDspEffect` bridge adapter.
- [x] `src-tauri/src/audio/effects/mod.rs` - Implement Filter, Isolator, Delay, Reverb, and VinylSim effects.
- [x] `src-tauri/src/audio/engine.rs` - Verify bus routing and validate zero-allocation processing (`assert_no_alloc`).

## Phase 3: UI Integration
- [ ] `src/main.ts` - Build frontend effect selector and wire to `set_bus_effect` command.
- [ ] `src/main.ts` - Add 3 rotary knobs (CTRL 1-3) and wire to `set_effect_param` command.
- [ ] `src/styles.css` - Add styling for knobs, effect selector, and parameter display.

## Phase 4: BPM & Beat Sync
- [ ] `src/main.ts` - Implement manual/tap BPM input and wire to `SetTempo` command.
- [ ] `src-tauri/src/audio/effects/mod.rs` - Build beat-synced effects: DJFX Looper, Scatter, Slicer.
- [ ] `src-tauri/src/audio/engine.rs` - Distribute tempo updates via `AudioCommand` to sync effects.

## Phase 5: Complete Catalog & Persistence
- [ ] `src-tauri/src/audio/effects/mod.rs` - Implement the remaining 29 standard MFX effects.
- [ ] `src-tauri/src/audio/state.rs` - Add serialization traits to effect state structs.
- [ ] `src-tauri/src/audio/engine.rs` - Wire serialization to persist and load FX configs on app restart.
