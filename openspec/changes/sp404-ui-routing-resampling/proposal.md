# Proposal: SP-404 UI Routing and Resampling

## Intent
Hook up frontend UI controls to the newly implemented Rust DSP routing and resampling capabilities, adhering to a retro-hardware workflow.

## Scope
### In Scope
- Resample button with blinking/solid states.
- Bus 1 and Bus 2 global hold-to-route buttons.
- LCD UI updates to reflect state changes.
- IPC integration to send state from the UI down to the DSP engine.

### Out of Scope
- Actual recording file saving (saving the buffer to disk). Only the memory buffer arming/recording is in scope.

## Capabilities
- `ui-bus-routing-hardware-style`
- `resample-arming-ui`
- `lcd-resample-state`

## UX and Design Details
Based on user decisions:
1. **Routing:** Hardware style. The user holds the global "Bus 1" or "Bus 2" button and taps individual pads to route them to the respective bus.
2. **Resample Button:** Skeuomorphic behavior. Blinks deep red when the resampler is armed, and stays solid red when it is actively recording.
3. **LCD Feedback:** When resampling is armed, the LCD shows `[RESAMPLING ARMED]` using the existing typing animation effect. Additionally, the LCD background changes to a reddish tint to indicate the armed state.
