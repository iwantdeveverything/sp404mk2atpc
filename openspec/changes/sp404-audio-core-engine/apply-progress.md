# Apply Progress: Core Audio Engine

## PR 1: Core Lock-Free Infrastructure & Mute Groups
- Created branch `feat/audio-core-engine`.
- Added `rtrb` crate for real-time ring buffer capabilities.
- Refactored `AudioState` in `src-tauri/src/audio/state.rs` to use lock-free communication queue (`AudioCommand` enum for IPC).
- Renamed voice tracking to `PlaybackEvent` and added `mute_group: Option<u8>` to it.
- Re-architected `src-tauri/src/audio/engine.rs` so `write_data` and the audio callback only read from the `rtrb::Consumer` without locking a mutex.
- Implemented mute group choking logic directly inside `write_data` command processing block by retaining non-matching group IDs.
- Added tests for `test_mute_group_choking` inside `engine.rs`.
- Validated via `cargo test` passing successfully.
