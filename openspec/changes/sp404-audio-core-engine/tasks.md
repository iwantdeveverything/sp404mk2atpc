# Tasks: Core Audio Engine

Decision needed before apply: No
Chained PRs recommended: Yes
400-line budget risk: Medium

## PR 1: Core Lock-Free Infrastructure & Mute Groups
- [x] Add `rtrb` dependency and set up lock-free communication channels between UI and audio threads.
- [x] Migrate audio render callback to ensure lock-free execution.
- [x] Add `mute_group` property to the `Voice` struct.
- [x] Implement mute group choking logic: scanning active voices and halting matching groups when a new voice triggers.

## PR 2: DSP Graph Routing & Resampling
- [x] Implement static DSP pipeline flow (`Voice Array` -> `Bus 1 / Bus 2 / Dry` -> `Master FX` -> `Output`).
- [x] Allocate large static vector for resampling buffer at initialization.
- [x] Add lock-free boolean flag (`AtomicBool`) to trigger resampling capture.
- [x] Implement audio stream capture to the internal record buffer when resampling mode is armed.
