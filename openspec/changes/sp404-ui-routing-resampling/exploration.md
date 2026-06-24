# Exploration: UI Routing & Resampling Integration

## Current State

### Backend (Rust)
- The audio engine already has a `resampling_armed: Arc<AtomicBool>` lock-free flag and logic to write the master mix to a `resampling_buffer` when armed.
- The `trigger_pad` method on `AudioState` accepts a `routing: BusRouting` parameter (which can be `Bus1`, `Bus2`, or `Dry`).
- However, the Tauri IPC command `trigger_pad` in `lib.rs` currently hardcodes the routing to `BusRouting::Dry` and ignores mute groups.
- There is no Tauri IPC command to toggle or set the `resampling_armed` flag.

### Frontend (TypeScript / HTML)
- The UI in `index.html` has pads and a top LCD screen section, but lacks a "Resample" button and Bus routing controls.
- `main.ts` invokes `trigger_pad` with only the `padId`. It does not store or send pad routing configurations.

## Approaches for Resampling Button

### Approach 1: `toggle_resampling` IPC Command
- Add a "RESAMPLE" button to the UI (`index.html`).
- Add a Tauri command `toggle_resampling(state: State<AudioState>) -> bool` that flips the `AtomicBool` and returns the new state.
- **Pros**: Source of truth is entirely in the backend. 
- **Cons**: UI has to wait for response to update visual state (glow/red).

### Approach 2: `set_resampling(armed: bool)` IPC Command
- UI holds the `isResampling` state and sends it via `set_resampling` Tauri command.
- **Pros**: UI can respond immediately and optimistically update.
- **Cons**: State is duplicated, though acceptable for a simple boolean.

*Recommendation*: **Approach 2** is better for UI responsiveness, which is critical in a DAW.

## Approaches for Pad FX Routing

### Approach 1: Global Bus Assignment Mode (SP-404 Style)
- Provide three global buttons: "BUS 1", "BUS 2", "DRY".
- Selecting one enters an "assignment mode". While in this mode, clicking a pad toggles its assignment to that bus.
- **Pros**: True to the hardware SP-404MK2 workflow.
- **Cons**: Slower to see individual pad routing at a glance; requires mode switching.

### Approach 2: Target Pad Bus Selectors (Inspector Style)
- Since the UI already has a "TARGET PAD" concept (selected via right-click for loading), add a set of radio buttons or a dropdown in the LCD screen that controls the routing for the *currently targeted pad*.
- Frontend maintains a `padRoutings: Record<number, 'Bus1' | 'Bus2' | 'Dry'>` state.
- When `trigger_pad` is called, it looks up the routing for that pad and sends it to Rust.
- **Pros**: Clean UI, easy to implement, leverages the existing target pad system.
- **Cons**: Can only view/edit routing for one pad at a time.

### Approach 3: Per-Pad UI Overlays
- Add a small text indicator (e.g., "B1", "B2", "D") on each pad itself. Clicking it cycles through the buses.
- **Pros**: Immediate visual feedback for all pads without selecting them.
- **Cons**: Might clutter the pad UI visually.

## Recommendation

**Resampling**: Implement **Approach 2 (`set_resampling`)** and add a bold "RESAMPLE" button next to the "LOAD SAMPLE" button. It should light up red when armed.

**Pad FX Routing**: Implement a hybrid of **Approach 2 and Approach 3**.
1. Store pad routings in a frontend state array/record.
2. Update the `trigger_pad` Tauri command in `lib.rs` to accept `routing: String` (which maps to `BusRouting`).
3. Add small visual indicators on the pads themselves (e.g., color-coded dots: Orange for Bus 1, Green for Bus 2, Gray for Dry) to show routing without cluttering text.
4. Add routing selection buttons (BUS 1, BUS 2, DRY) in the LCD screen that apply to the currently *target pad*. This keeps the pad area clean for clicking/drumming, while providing full visibility.
