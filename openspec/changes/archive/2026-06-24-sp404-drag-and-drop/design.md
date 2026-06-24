# Design: Drag and Drop

## Architecture
This feature integrates with Tauri v2's native window drag-and-drop capabilities. We will use `@tauri-apps/api/webview` to listen for drag-and-drop events at the webview level. Since this relies on the native OS file drop mechanism, it bypasses HTML5 drag-and-drop. The frontend will map native window coordinates to DOM elements to determine which pad is being interacted with, and invoke the existing Rust `load_audio` command for each dropped file.

## Data Flow
1. **Drag Over**: 
   - `getCurrentWebview().onDragDropEvent` fires with a payload type of `over`.
   - The payload provides `position: { x, y }`.
   - We use `document.elementFromPoint(x, y)` to find the topmost DOM element.
   - If the element (or its parent) is a `.pad`, we add a `.drag-hover` CSS class to it and remove it from all other pads.
2. **Drag Leave**:
   - Event payload type `leave` or `cancel` is received.
   - We remove `.drag-hover` from all pads and reset the LCD if it was showing a drag message.
3. **Drop**:
   - Event payload type `drop` is received containing `paths` (array of file paths) and `position: { x, y }`.
   - We find the target pad using `document.elementFromPoint`. If no pad is found, the drop is ignored.
   - We iterate through the `paths` array. For each path, we verify it has a valid audio extension (`.wav`, `.mp3`).
   - If invalid, we display "INVALID FORMAT" on the LCD.
   - For valid files, we invoke the `load_audio` command, incrementing the target `padId` for each successive file so they populate sequentially.
   - The LCD displays "LOADING..." during the process and "LOADED PAD X" upon completion.

## Decisions

### Decision: Detecting Drop Target
- **Context**: We need to determine which pad the user dropped files onto from the OS.
- **Options**: Use HTML5 `dragenter`/`drop` events on DOM elements OR use Tauri's native `onDragDropEvent` with coordinate mapping.
- **Selected**: Coordinate mapping (`document.elementFromPoint(x, y)`) inside Tauri's `onDragDropEvent`.
- **Rationale**: Tauri intercepts OS file drops natively at the window level, which is more reliable than standard HTML5 drag-and-drop for external files. The event provides exact window coordinates, making `elementFromPoint` the most accurate way to map the native drop event to our `.pad` DOM elements.

### Decision: Pad Hover States
- **Context**: We must provide visual feedback when files are dragged over pads.
- **Options**: Use CSS `:hover` pseudo-class OR manually toggle a dedicated `.drag-hover` class.
- **Selected**: Manually toggle a `.drag-hover` class during the `over` event.
- **Rationale**: The CSS `:hover` state is not reliably triggered by the browser when dragging external OS files. Manually calculating the hovered element via coordinates and applying a class guarantees responsive and accurate visual feedback.

### Decision: LCD Text Updates
- **Context**: The user needs feedback on whether the files are valid and loading successfully.
- **Options**: Use native OS message boxes/alerts OR update the in-app LCD screen (`#status-display`).
- **Selected**: Update the in-app LCD screen.
- **Rationale**: The proposal specifies using the LCD for error messages and transient states to maintain immersion in the SP-404 hardware aesthetic. We will use custom messages like "INVALID FORMAT", "LOADING...", and "LOADED PAD X" directly on the DOM element.

## File Structure
- **`src/main.ts`**: 
  - Import `getCurrentWebview` from `@tauri-apps/api/webview`.
  - Add the `onDragDropEvent` listener in the `DOMContentLoaded` block.
  - Implement coordinate mapping and sequential loading logic.
- **`src/styles.css`**: 
  - Add the `.drag-hover` CSS class with a visual effect (e.g., a glowing border or background change) for pads.
