# Design: SP-404MK2 Library Management

## 1. Architecture Overview
This change introduces a full-disk file browser, project bundling, and a dedicated pre-listen channel. The architecture leans heavily on Tauri's IPC capabilities to safely access the filesystem and manage audio playback via Rust, while the frontend handles fluid, micro-interaction-rich UI rendering.

## 2. File Browser & Tauri Integration (`ui-routing`)
- **Disk Access**: We will use `@tauri-apps/plugin-dialog` to allow users to select project directories or sample folders.
- **File System**: To read arbitrary files across the disk, we will either use Tauri's `asset://` protocol (configured for safe scopes based on user selection) or dedicated Rust IPC commands to list directory contents (`fs::read_dir`).
- **State Management**: The frontend will maintain a tree/list state of the currently browsed directory. 

## 3. Pre-listen Mechanism (`audio-core`, `bpm-sync`, `effects-engine`)
- **Backend Implementation**: 
  - A new, independent audio playback channel will be added to the Rust backend (via `fundsp` / `cpal`).
  - **Routing**: This channel will be hardcoded to mix directly into the final output, explicitly bypassing the main FX bus and the BPM sync engine.
- **Frontend Invocation**: When a user selects or hovers over a sample (depending on UX preference), the frontend calls `invoke('pre_listen_start', { path })`. 

## 4. Waveform Rendering & UI Juice (`ui-juice`)
- **Waveform Rendering**: To meet the "totally accurate, intuitive, and alive" requirement, we will use the **Native HTML5 Canvas API**. 
  - The frontend will load the audio file via the Tauri custom protocol, decode it using `OfflineAudioContext`, and extract the peak data.
  - The Canvas will draw the peaks with dynamic styling (e.g., gradients, real-time playhead overlay driven by `requestAnimationFrame`).
- **Animations & Hover States**: 
  - Use CSS3 hardware-accelerated transitions (e.g., `transform`, `opacity`) with custom `cubic-bezier` curves for hover interactions.
  - No bloated external UI frameworks will be used; vanilla CSS and Canvas give maximum performance and control.

## 5. Drag and Drop Ingestion (`drag-drop`)
- **HTML5 Drag & Drop**: We will implement native DOM drag events (`dragstart`, `dragenter`, `dragover`, `dragleave`, `drop`).
- **Visual Feedback**:
  - **Hover / Drag Over**: When a file is dragged over a valid pad target, the pad will receive a `.drag-target-active` class (triggering a glowing border and slight scale-up).
  - **Success / Fail**: Upon dropping, the pad will play a CSS keyframe animation (`pulse-success`) to give immediate tactile feedback.
- **Bundle Logic**: On drop, the frontend calls `invoke('ingest_sample_to_project', { source_path, target_pad })`. Rust will physically copy the file into the project's internal `samples/` directory and return the relative path.

## 6. Auto-save Engine
- **Implementation**: A lightweight timer will trigger a debounced save operation in the background. The state will be serialized to `project.json` inside the project bundle directory.
- **Non-blocking**: The save operation will be asynchronous on the Rust side to ensure the UI and audio threads never stutter.
