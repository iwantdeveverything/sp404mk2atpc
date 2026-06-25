# Tasks: SP-404MK2 Library Management

## Delivery Strategy
- **Strategy**: `auto-chain`
- **Chain Strategy**: `stacked-to-main`
- **Review Workload Forecast**: The scope includes Rust IPC, Audio DSP modifications (Pre-listen channel), Canvas API integration, and DOM drag/drop events. This is estimated to significantly exceed 400 lines of code. Therefore, the implementation is broken down into 4 stacked PRs to keep review payloads focused and safe.

## PR 1: Foundation (Rust IPC & Core File System)
*Target branch: `feature/sp404-library-management-1-foundation`*
- [x] Configure `@tauri-apps/plugin-dialog` in `Cargo.toml` and `tauri.conf.json` (and package.json).
- [x] Implement Rust IPC command to list directory contents safely (e.g., via `fs::read_dir`).
- [x] Implement Rust IPC command `ingest_sample_to_project` that copies a source file to the project's internal `samples/` directory and returns the relative path.
- [x] Implement a debounced, background auto-save engine in Rust that serializes project state to `project.json` inside the project bundle directory.

## PR 2: Pre-listen Audio Engine (Rust `audio-core`)
*Target branch: `feature/sp404-library-management-2-prelisten` (stacked on PR 1)*
- [x] Create an independent audio playback channel in the Rust backend (`fundsp` / `cpal`).
- [x] Hardcode the routing for this channel to mix directly into the final output, explicitly bypassing the main FX bus and BPM sync engine.
- [x] Expose an IPC command `pre_listen_start(path: String)` that loads and plays the raw audio through this new channel.

## PR 3: UI - File Browser & Canvas Waveform (Frontend)
*Target branch: `feature/sp404-library-management-3-browser` (stacked on PR 2)*
- [x] Create the frontend File Browser component utilizing the new IPC commands for directory listing and disk access.
- [x] Integrate the Native HTML5 Canvas API for waveform rendering (load file, decode with `OfflineAudioContext`, extract peak data).
- [x] Render peaks dynamically with Canvas and link `requestAnimationFrame` for a real-time playhead overlay.
- [x] Connect the pre-listen IPC command (`pre_listen_start`) to frontend interactions (e.g., hover/select over a sample).
- [x] Apply CSS3 hardware-accelerated transitions (using vanilla CSS, no bloated frameworks) for micro-interactions.

## PR 4: UI - Drag & Drop and Polish (Frontend)
*Target branch: `feature/sp404-library-management-4-dnd` (stacked on PR 3)*
- [ ] Implement native DOM drag events (`dragstart`, `dragenter`, `dragover`, `dragleave`, `drop`) on the pad components.
- [ ] Add visual feedback CSS classes (`.drag-target-active`) for hover/drag over states (glowing border, scale-up).
- [ ] Create and trigger CSS keyframe animations (e.g., `pulse-success`) upon a successful drop.
- [ ] Wire the `drop` event to call the `ingest_sample_to_project` IPC command and correctly associate the returned relative path with the target pad.
