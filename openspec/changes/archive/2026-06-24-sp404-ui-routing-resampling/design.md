# Design: SP-404 UI Routing and Resampling

## Architecture Decisions

### Frontend (TypeScript + CSS)

1. **Hold-to-Route State:** Use `mousedown` and `mouseup` (or `pointerdown`/`pointerup`) on the Bus global buttons to set a global `isBus1Held` / `isBus2Held` state. When a pad is clicked, check those states before playing audio to override the default behavior and instead route the pad.
2. **CSS Animations:** Define `@keyframes blink-red` and add a `.resample-armed` class that applies the animation and a deep inset shadow. Add `.resample-recording` for the solid state.
3. **LCD Red Tint:** Add a `.lcd-resampling` class to the LCD container that applies a subtle red filter or background tint.
4. **IPC calls:** Identify `invoke('set_resampling', { state: true })` and `invoke('set_pad_bus', { pad: id, bus: 'Bus1' })` Tauri IPC contracts.
