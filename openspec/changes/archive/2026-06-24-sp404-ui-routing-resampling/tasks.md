# Tasks: sp404-ui-routing-resampling

Decision needed before apply: No
Chained PRs recommended: No
400-line budget risk: Low

## PR 1: UI Routing and Resampling Implementation
- [x] Add CSS animations and classes (`@keyframes blink-red`, `.resample-armed`, `.resample-recording`, `.lcd-resampling`) to the frontend stylesheet.
- [x] Implement Tauri IPC commands `set_resampling` and `set_pad_bus` in the Rust backend to connect UI events with the DSP engine.
- [x] Implement global hold states for Bus 1 and Bus 2 buttons (`mousedown`/`mouseup` or `pointerdown`/`pointerup` events) in the frontend.
- [x] Update pad interaction logic to route the pad via `invoke('set_pad_bus')` instead of playing audio if Bus 1 or Bus 2 is held.
- [x] Implement the "Resample" button logic to toggle the armed state, apply the `.resample-armed` class, and update the LCD to show `[RESAMPLING ARMED]` along with the reddish tint.
- [x] Update pad interaction logic so that hitting a pad while in the armed state transitions the UI to the recording state (applying `.resample-recording` and calling `set_resampling` IPC).
