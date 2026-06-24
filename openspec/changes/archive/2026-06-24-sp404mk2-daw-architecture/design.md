# Design: sp404mk2-daw-architecture

## Technical Approach

We are building a Tauri desktop application with a Rust backend for low-latency audio processing and a Vanilla TypeScript frontend for a highly responsive UI. The audio engine will use `cpal` directly for minimal overhead and `hound` for WAV parsing. The frontend will trigger audio via Tauri IPC commands, ensuring UI rendering does not block the audio thread. The architecture is designed around an in-memory buffer pool that can capture its own output, setting the foundation for destructive resampling.

## Architecture Decisions

### Decision: Audio Engine Library

**Choice**: `cpal`
**Alternatives considered**: `rodio`, `kira`
**Rationale**: We need absolute control over the audio callback buffer to achieve the lowest possible latency and to implement our own resampling logic. High-level libraries introduce abstractions that complicate destructive audio manipulation.

### Decision: Frontend Stack

**Choice**: Vanilla TypeScript + Vite + Vanilla CSS
**Alternatives considered**: React, Vue, Tailwind CSS
**Rationale**: The SP-404 workflow requires an instantly responsive UI. Avoiding virtual DOM overhead and framework lifecycles guarantees pad clicks register instantly. Vanilla CSS provides sufficient styling capabilities without external dependencies or build-step overhead.

### Decision: Audio Data Storage

**Choice**: In-memory `f32` buffer pool in Rust
**Alternatives considered**: Streaming directly from disk, Web Audio API
**Rationale**: Streaming from disk introduces I/O latency on trigger. Web Audio API is tied to the browser's audio context, which may introduce unpredictable latency. Loading entire WAV files into memory as `f32` vectors in Rust allows instantaneous playback upon IPC command receipt.

## Data Flow

    [Frontend (Vite/TS)]                      [Backend (Rust)]
         │                                          │
         ├─ File Selection (IPC) ─────────────────> │ (hound parses WAV)
         │                                          ↓
         │                                    [Buffer Pool]
         │                                          │
         ├─ Pad Trigger (IPC: Play Buf X) ────────> │
         │                                          ↓
         │                                    [Audio Thread] (cpal)
         │                                          │
         │                                          ↓
                                              [System Audio]

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/Cargo.toml` | Create | Define Rust dependencies (`tauri`, `cpal`, `hound`). |
| `src-tauri/src/main.rs` | Create | Entry point, sets up Tauri app, initializes audio engine state, and registers IPC commands. |
| `src-tauri/src/audio/engine.rs` | Create | Initializes `cpal` stream, manages the audio callback thread, and mixes active buffers. |
| `src-tauri/src/audio/state.rs` | Create | Thread-safe state (`Mutex`/`RwLock`) containing loaded audio buffers and active playback events. |
| `src-tauri/src/fs/wav.rs` | Create | File loading and parsing logic using `hound`. |
| `src/index.html` | Create | Main HTML structure for the pad grid and file explorer button. |
| `src/main.ts` | Create | Frontend logic for handling keyboard/mouse events and dispatching Tauri IPC commands. |
| `src/styles.css` | Create | Vanilla CSS for pad visuals, active states, and layout. |

## Interfaces / Contracts

**Tauri IPC Commands**:

```rust
// Invoked when the user selects a file
#[tauri::command]
fn load_wav(path: String, pad_id: usize, state: State<AudioState>) -> Result<(), String>

// Invoked when a pad is clicked or mapped key pressed
#[tauri::command]
fn trigger_pad(pad_id: usize, state: State<AudioState>) -> Result<(), String>
```

**Audio Buffer Structure**:
```rust
struct AudioBuffer {
    samples: Vec<f32>,
    channels: u16,
    sample_rate: u32,
}
```

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit | WAV Parsing | Provide sample WAV files and assert correct sample extraction and normalization via `cargo test`. |
| Unit | Audio Mixer | Feed dummy sine wave buffers into the mixing function and verify the output buffer summation. |
| Integration | IPC to Engine | Call Tauri command handlers programmatically and assert that the `AudioState` is updated correctly. |
| E2E | Pad Trigger | Use a testing framework (e.g., Tauri's testing tools or WebDriver) to click a pad and verify IPC invocation. |

## Migration / Rollout

No migration required. This is the initial architectural foundation for the greenfield project.

## Open Questions

- [ ] Should we support bit depths other than 16-bit and 24-bit WAVs in the MVP?
- [ ] How should we handle sample rate conversion if a loaded WAV does not match the system's `cpal` stream sample rate?
