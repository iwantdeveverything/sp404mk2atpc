## Exploration: organizar samples y pads, persistir cambios, guardar proyectos, organizar carpetas, file browser con preescucha, renombrar, catalogar, editar - gestión completísima de librerías

### Current State
- The application handles audio files by directly loading them into transient memory (`AudioBuffer`) through the Tauri file dialog (`tauri-plugin-dialog`) or drag-and-drop events in the webview.
- There is no concept of a "Project" or "Sample Library". Audio mappings (which pad has which sample) are not persisted across application restarts.
- Only the effects configuration (`AppFxConfig`) is persisted to disk using a simple `fx_config.json` via `serde_json`.

### Affected Areas
- **Frontend (TS/HTML/CSS)**: Needs a complete file browser UI with directory navigation, pre-listening controls, and project management (save/load/organize).
- **Backend (Rust/Tauri)**:
    - `src-tauri/src/audio/state.rs`: Needs an extended state to map pads to sample file paths and playback settings.
    - `src-tauri/src/audio/engine.rs`: Needs a dedicated channel or player for "pre-listening" (streaming or quick-loading a file without assigning it to a pad).
    - `src-tauri/src/fs/`: Needs new modules to handle directory reading (for the file browser) and project bundling (copying samples to a project folder).
    - `src-tauri/src/lib.rs`: Needs new Tauri commands (`read_dir`, `save_project`, `load_project`, `prelisten_file`, `stop_prelisten`).

### Approaches
1. **SQLite**: Excellent for global sample cataloging and complex tagging. However, it requires adding SQL dependencies (`rusqlite` or `sqlx`) and might be overkill for standard project-by-project management.
2. **JSON (`serde_json`)**: Already used in the project (`fx_config.json`). Very lightweight, easy to implement, and maps perfectly to a `Project` struct containing pad mappings. Ideal for `project.json` files.
3. **YAML**: More human-readable but requires new dependencies. Not strictly better than JSON for our use case.

**File Management & Portability:**
- If we save absolute paths in a project file, moving samples will break the project. The best DAW approach is to create a project bundle (a directory containing `project.json` and a `samples/` subdirectory) and copy used samples there, referencing them relatively.

### Recommendation
- **Format**: Use **JSON** (`serde_json`) for saving project files.
- **Portability**: Implement a Project Bundle architecture. When saving, create a folder for the project, copy used samples into a `samples/` directory, and save relative paths in `project.json`.
- **File Browser**: Implement custom Tauri commands in Rust to traverse the local filesystem safely and return directories/files to the frontend.
- **Pre-listening**: Add a separate audio channel in the `audio::engine` specifically for pre-listening, which bypassing the pad assignment logic.

### Risks
- **Blocking the UI**: Loading an entire project with multiple large WAV files synchronously will freeze the Tauri UI. We must implement asynchronous loading with progress events.
- **Storage Space**: Copying samples into project folders duplicates files, which consumes more disk space (though it ensures safety and portability).
- **Permissions**: Tauri needs proper scope configuration (`tauri.conf.json`) to allow reading arbitrary directories for the file browser, or we must use `tauri-plugin-fs` carefully.

### Ready for Proposal
Yes
