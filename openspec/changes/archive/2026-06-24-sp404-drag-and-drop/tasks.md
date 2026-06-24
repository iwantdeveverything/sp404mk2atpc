# Tasks: sp404-drag-and-drop

## PR 1: Implement Native Drag and Drop
- [x] Update `src/styles.css` with a `.drag-hover` class to provide visual feedback (e.g., glowing border or background change) when a file is dragged over a pad.
- [x] Import `getCurrentWebview` from `@tauri-apps/api/webview` in `src/main.ts`.
- [x] Add `onDragDropEvent` listener inside the `DOMContentLoaded` block in `src/main.ts` to intercept native OS file drop events.
- [x] Implement hover logic for the `over` event payload: use `document.elementFromPoint(x, y)` to detect the hovered pad, add the `.drag-hover` class to it, and remove it from others.
- [x] Implement cleanup logic for `leave` or `cancel` event payloads: remove the `.drag-hover` class from all pads and reset any drag-related LCD messages.
- [x] Implement drop logic for the `drop` event payload: determine the target pad via `document.elementFromPoint(x, y)` and ignore the drop if not on a pad.
- [x] Add validation in the drop handler to check that each dropped file has a valid audio extension (e.g., `.wav`, `.mp3`). Update the LCD (`#status-display`) to show "INVALID FORMAT" if any file is invalid.
- [x] Implement sequential loading for valid files: iterate over the dropped files, invoke the `load_audio` backend command for each, and increment the target `padId` for successive files so they populate pads sequentially.
- [x] Update the LCD during the load process to show "LOADING..." and finally "LOADED PAD X" upon completion.
