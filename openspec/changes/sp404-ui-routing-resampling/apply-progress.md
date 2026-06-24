# Apply Progress: SP-404 UI Routing and Resampling

## Changes Implemented
- Created new feature branch `feat/sp404-ui-routing-resampling`.
- **Frontend CSS**: Added `@keyframes blink-red`, `.resample-armed`, `.resample-recording`, and `.lcd-resampling` in `src/styles.css` for routing and resampling visual feedback.
- **Frontend UI**: Added `RESAMPLE`, `BUS 1`, and `BUS 2` buttons into the `.controls` div in `index.html`.
- **Frontend Logic**: 
  - Added global hold states for Bus 1 and Bus 2 (`mousedown`/`touchstart` and `mouseup`/`touchend`) in `src/main.ts`.
  - Added click event listener to the Resample button to toggle `isResampleArmed`, update the UI classes, and change LCD text.
  - Updated `triggerPad` in `src/main.ts` to intercept pad hits when a Bus button is held, emitting `set_pad_bus` IPC instead of playing audio.
  - Updated `triggerPad` to transition to recording mode (`isResampleRecording = true`), update UI, and emit `set_resampling` IPC when a pad is hit while armed.
- **Backend IPC**: Added `set_resampling` and `set_pad_bus` Tauri commands in `src-tauri/src/lib.rs` and registered them in the invoke handler.
- **Tasks**: Updated `tasks.md` to check off all items for PR 1.
