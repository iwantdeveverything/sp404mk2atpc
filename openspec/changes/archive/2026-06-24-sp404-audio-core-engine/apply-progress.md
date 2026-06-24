# Apply Progress: PR 2

- Checked out new feature branch: `feat/dsp-graph-routing-resampling`
- Updated `src-tauri/src/audio/state.rs` to include `BusRouting` (Bus1, Bus2, Dry), and added it to `AudioCommand` and `PlaybackEvent`.
- Added an `Arc<AtomicBool>` named `resampling_armed` in `AudioState` to support lock-free triggering.
- Updated `src-tauri/src/audio/engine.rs` to initialize a static vector of size 28,800,000 floats (5 minutes of stereo 48kHz audio) for the resampling capture buffer to avoid allocations.
- Implemented static DSP pipeline flow mapping active voices to Bus1, Bus2, and Dry mix channels.
- Added capture logic at the end of the `write_data` function that records to `resampling_buffer` when `resampling_armed` is true.
- Updated test cases in `engine.rs` to properly use the new routing parameters and test setup.
- Updated `tasks.md` to reflect the completion of PR 2 tasks.
