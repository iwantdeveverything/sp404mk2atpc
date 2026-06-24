# Tasks: sp404mk2-daw-architecture

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~600-800 lines (including Tauri boilerplate) |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | PR 1 → PR 2 → PR 3 → PR 4 |
| Delivery strategy | ask-on-risk |
| Chain strategy | pending |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: pending
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | Project Initialization | PR 1 | Scaffold base Tauri app with Vite + Vanilla TS |
| 2 | Audio Engine Core | PR 2 | `cpal` stream, thread-safe state, on-the-fly resampling |
| 3 | File Loader & IPC | PR 3 | WAV/MP3 parsing, `load_audio` and `trigger_pad` commands |
| 4 | UI Integration | PR 4 | Connect frontend UI pads/file picker to Tauri IPC |

## Phase 1: Project Initialization

- [x] 1.1 Scaffold Tauri application with Vite and Vanilla TypeScript.
- [x] 1.2 Clean up default Vite boilerplate and establish base file structure.

## Phase 2: Audio Engine Core

- [x] 2.1 Add Rust dependencies (`cpal`, `hound`, MP3 decoder) to `src-tauri/Cargo.toml`.
- [x] 2.2 Create `src-tauri/src/audio/state.rs` for thread-safe audio buffers and playback events.
- [x] 2.3 Create `src-tauri/src/audio/engine.rs` to initialize `cpal` stream and audio thread.
- [x] 2.4 Implement basic on-the-fly resampling algorithm in the audio callback.
- [x] 2.5 Implement audio mixer to handle simultaneous triggers.

## Phase 3: File Loading and IPC

- [ ] 3.1 Create `src-tauri/src/fs/audio.rs` to handle parsing both WAV and MP3 files.
- [ ] 3.2 Implement `load_audio` Tauri command to read audio into `AudioState`.
- [ ] 3.3 Implement `trigger_pad` Tauri command.
- [ ] 3.4 Wire up state and commands in `src-tauri/src/main.rs`.

## Phase 4: UI Integration

- [ ] 4.1 Update `src/index.html` with pad grid layout and load button.
- [ ] 4.2 Write `src/styles.css` for SP-404 style visual feedback (active states).
- [ ] 4.3 Update `src/main.ts` to attach click/keyboard events that invoke `trigger_pad`.
- [ ] 4.4 Add file picker logic invoking `load_audio` and updating the UI state.

## Phase 5: Testing

- [ ] 5.1 Write unit tests for WAV/MP3 parsing logic.
- [ ] 5.2 Write unit tests for the basic resampling algorithm and audio mixer.
