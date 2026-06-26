# Apply Progress

## Completed Phases
- Phase 1: Foundation (Rust IPC & Core File System)
  - Installed `@tauri-apps/plugin-dialog`.
  - Added `fs::project` module in Rust backend.
  - Implemented `list_directory`, `ingest_sample_to_project`, and `save_project_state` IPC commands.
  - Developed a debounced background save engine using `mpsc` channel.
  - Verified compilation via `cargo check`.
- Phase 2: Pre-listen Audio Engine (Rust `audio-core`)
  - Independent pre-listen channel created bypassing FX/BPM sync.
  - Exposed via `pre_listen_start` IPC command.
- Phase 3: UI - File Browser & Canvas Waveform (Frontend)
  - Created frontend File Browser component utilizing `list_directory` IPC command.
  - Integrated HTML5 Canvas API for dynamic waveform rendering using `OfflineAudioContext` to extract peaks.
  - Wired `requestAnimationFrame` for a real-time playhead overlay.
  - Connected `pre_listen_start` IPC command to browser file interactions.
  - Applied CSS3 hardware-accelerated transitions (glassmorphism panel, active state highlights, smooth transforms) via Vanilla CSS.

- Phase 4: UI - Drag & Drop and Polish (Frontend) + backend default-project-dir gap
  - Backend gap (Strict TDD): added pure helper `resolve_default_project_dir(base)` + `#[cfg(test)]` unit test (RED→GREEN) and thin AppHandle command `get_default_project_dir`, registered in `lib.rs` invoke_handler. Uses Tauri 2 `app.path().app_data_dir()` joined with stable `default-project` subfolder, created via `fs::create_dir_all`.
  - Frontend: file-browser audio items now `draggable="true"` with `dragstart`/`dragend` storing the source path in a module-scoped `internalDragPath` (and `dataTransfer`).
  - Frontend: pad elements handle `dragenter`/`dragover`/`dragleave`/`drop` for the internal DOM drag, gated on `internalDragPath !== null` so the native OS drop path (`onDragDropEvent`) is untouched and both mechanisms coexist.
  - Frontend: on drop, lazily fetch + cache `get_default_project_dir` (module-scoped `cachedProjectDir`), call `ingest_sample_to_project`, then `load_audio` with `projectDir + "/" + relativePath`; status LCD updates via `typeText`; `pulse-success` animation replays via reflow.
  - CSS: added `.drag-target-active` (accent glow + scale-up) and `pulse-success` keyframe (hardware-accelerated transform/box-shadow), reusing `#4ade80`.

## TDD Cycle Evidence (Strict TDD — backend)
| Task | RED | GREEN | REFACTOR |
|------|-----|-------|----------|
| `resolve_default_project_dir` helper + `get_default_project_dir` command | Test added first, failed to compile (`cannot find function resolve_default_project_dir`) | Implemented helper + command; `cargo test` → 9 passed | Extracted pure helper from AppHandle wrapper; no further refactor needed |

## Verification
- `cargo test` → ok, 9 passed; 0 failed.
- `cargo check` → Finished, no errors/unused warnings.
- `npm run build` (tsc && vite) → built successfully, zero TS errors, no unused vars.

## Tauri Invoke Arg Casing (confirmed)
Tauri 2 maps Rust snake_case command params to JS camelCase by default. Confirmed against existing invokes in main.ts (`pad_id` → `padId`, `param_id` → `paramId`). PR4 therefore passes `sourcePath` (→ `source_path`) and `projectDir` (→ `project_dir`).

## Current Status
- PR 1, PR 2, PR 3, and PR 4 implementation is complete.
- PR 4 committed locally on `feature/sp404-library-management-4-dnd` (3 work-unit commits). Not pushed; change left un-archived.
