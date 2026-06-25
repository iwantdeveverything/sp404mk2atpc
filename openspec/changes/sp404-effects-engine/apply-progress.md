# Apply Progress: sp404-effects-engine

## Completed Tasks
- [x] `src-tauri/Cargo.toml` - Add `fundsp` and serialization dependencies.
- [x] `src-tauri/src/audio/effects/mod.rs` - Define `Effect` trait, `EffectType` enum, and `EffectChain` struct.
- [x] `src-tauri/src/audio/state.rs` - Add `SetBusEffect`, `SetEffectParam`, `RemoveBusEffect`, and `SetTempo` to `AudioCommand`.
- [x] `src-tauri/src/audio/engine.rs` - Add FX chain fields to `AudioEngineThreadState` and process at L138-145 placeholder.
- [x] `src-tauri/src/audio/mod.rs` - Declare `pub mod effects`.
- [x] `src-tauri/src/lib.rs` - Add Tauri commands `set_bus_effect` and `set_effect_param`.

## Files Changed
| File | Action | What Was Done |
|------|--------|---------------|
| `src-tauri/Cargo.toml` | Modified | Added `fundsp` dependency. |
| `src-tauri/src/audio/effects/mod.rs` | Created | Defined `Effect` trait, `EffectType` enum, and `EffectChain` struct. |
| `src-tauri/src/audio/mod.rs` | Modified | Declared `effects` module. |
| `src-tauri/src/audio/state.rs` | Modified | Added new `AudioCommand` variants and imported `EffectType`. |
| `src-tauri/src/audio/engine.rs` | Modified | Added FX chains to `AudioEngineThreadState`, command handlers, and audio processing logic. |
| `src-tauri/src/lib.rs` | Modified | Added `set_bus_effect` and `set_effect_param` Tauri commands. |
