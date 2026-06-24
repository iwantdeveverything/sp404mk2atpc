# Exploration: SP-404 Drag and Drop

## 1. Goal
Add support for dragging and dropping audio samples (WAV/MP3) directly onto the SP-404 pads in the UI, updating the target pad and triggering the `load_audio` IPC call.

## 2. Current Implementation
- `src/main.ts` sets up pads using `document.querySelectorAll(".pad")` and `data-pad` attributes.
- Audio loading currently relies on the Tauri Dialog plugin via a "LOAD SAMPLE" button, which calls the `load_audio` IPC with a single file path and the currently selected `currentTargetPad`.
- The Rust backend (`src-tauri`) receives the absolute file path as a string and processes the audio file.

## 3. Technical Options

### Option A: Standard HTML5 Drag & Drop API
- **Approach**: Set `fileDropEnabled: false` in `tauri.conf.json`. Use standard `ondragover` and `ondrop` DOM events on the `.pad` elements.
- **Pros**: Familiar web API, easy to target specific elements directly without coordinate mapping.
- **Cons**: Due to browser security restrictions, the HTML5 `DataTransfer` object does not reliably expose the absolute file system path of the dropped file. We absolutely need the absolute path to pass to the Rust backend (`load_audio` expects a string path).

### Option B: Tauri v2 Webview Drag & Drop API (Recommended)
- **Approach**: Leave `fileDropEnabled: true` (default). Use `@tauri-apps/api/webview` to listen for drag and drop events via `getCurrentWebview().onDragDropEvent(...)`.
- **How it works**:
  - The `tauri://drag-drop` event (via `onDragDropEvent`) provides a payload containing `paths` (an array of absolute file paths) and `position` (`x` and `y` coordinates).
  - In the `drop` event, we can use `document.elementFromPoint(x, y)` to determine which DOM element is under the cursor.
  - If the element (or its closest ancestor) has the `.pad` class, we extract the `data-pad` attribute to determine the target pad ID.
  - We then check if the dropped file has a `.wav` or `.mp3` extension.
  - Finally, we call the `load_audio` IPC command with the first valid file path and the target pad ID.
- **Pros**: Reliably provides absolute file system paths. Better integration with the OS and Tauri's security model.
- **Cons**: Requires mapping `x` and `y` coordinates to DOM elements.

### Option C: HTML5 Drag & Drop with Tauri File Read
- **Approach**: It may be possible to read the file using standard File API as an ArrayBuffer and pass the bytes to Rust.
- **Pros**: HTML5 API.
- **Cons**: Requires changing the backend `load_audio` to accept bytes instead of a path, or creating a new IPC. Too much refactoring compared to just passing the path.

## 4. Proposed Solution
Implement **Option B**:
1. Ensure Tauri drag and drop is enabled (default behavior).
2. Update `src/main.ts` to import `getCurrentWebview` from `@tauri-apps/api/webview`.
3. Add a listener: `getCurrentWebview().onDragDropEvent((event) => { ... })`.
4. In the handler:
   - Handle `dragOver` event to show a visual highlight on the hovered pad (by tracking the element under cursor and adding an `.active` or `.hover` class).
   - Handle `drop` event:
     - Get the dropped element using `document.elementFromPoint(event.payload.position.x, event.payload.position.y)`.
     - Find `.closest('.pad')`.
     - Read the `data-pad` ID.
     - Extract `event.payload.paths[0]`.
     - Verify it's a `.wav` or `.mp3`.
     - Invoke `load_audio` with `padId` and `path`.
5. Remove any leftover visual states on `dragLeave` or `drop`.

## 5. Risks
- Coordinate offsets: OS-specific bugs (e.g. macOS titlebar offsets) might slightly skew `elementFromPoint`. We may need to test on the target OS.
- Missing dependencies: `@tauri-apps/api/webview` needs to be in `package.json`.
