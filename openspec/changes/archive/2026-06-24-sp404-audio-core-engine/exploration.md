# Audio Core Engine Exploration

## Current State Analysis
The current implementation in `src-tauri/src/audio/engine.rs` and `state.rs` provides a very basic playback engine:
- Uses `std::sync::Mutex` in the real-time audio thread (`write_data`), which is a major anti-pattern and can lead to xruns (audio dropouts) if the main thread holds the lock.
- Simple linear nearest-neighbor resampling.
- No voice management: Every pad hit adds a new `PlaybackEvent` to `active_events` without any polyphony limit, meaning 100 fast hits will trigger 100 concurrent voices.
- No choking / mute groups (essential for a drum machine/sampler workflow).
- Hardcoded single output mix without any intermediate bus routing (FX buses).

## Architectural Options & Tradeoffs

### 1. Engine Architecture: Custom Mixer vs DSP Framework
- **Option A: Use an existing DSP framework (`fundsp`, `daspl`)**
  - *Pros*: Built-in resampling, oscillators, and basic FX. Fast time-to-market.
  - *Cons*: Might dictate the architecture too much. Can be bloated for a simple sampler that just needs pristine buffer playback and specific bus routing.
- **Option B: Custom Ring-Buffer Lock-Free Mixer (Recommended)**
  - *Pros*: Absolute control over the audio thread. Using a lock-free structure like `crossbeam-channel` or `ringbuf` for communicating between Tauri commands and the `cpal` real-time thread ensures ultra-low latency and zero dropouts.
  - *Cons*: We need to write the voice allocation and bus mixing manually.

### 2. Polyphony & Voice Management
- **Mute Groups (Choking)**: The SP-404 requires pads to cut each other off (e.g., open hi-hat vs closed hi-hat). 
- **Voice Stealing**: A max global polyphony limit (e.g., 32 voices) should be established. If a 33rd voice is triggered, the oldest/quietest voice is "stolen" with a fast fade-out to prevent popping.
- *Recommendation*: Implement a fixed-size array/pool of `Voice` structs. When a pad is triggered, send a `Command::PlayPad { pad_id, velocity }` via a lock-free queue. The audio thread assigns it to a free voice or steals an existing one. If the pad belongs to a Mute Group, the audio thread automatically fades out any active voice in that same group.

### 3. FX Bus Routing (Bus 1, Bus 2, Master)
- The SP-404 workflow revolves around routing pads to Bus 1, Bus 2, or Dry (Master).
- *Recommendation*: The audio thread should mix active voices into multiple intermediate buffers (`Bus 1 Buffer`, `Bus 2 Buffer`). Then process FX on those buffers. Finally, mix all buses into the `Master Buffer` before copying to the `cpal` output slice.

### 4. Resampling Preparation
- To support resampling, we need to be able to capture the `Master Buffer` (or a specific bus) and write it back into a new sample buffer.
- *Recommendation*: Add a lock-free queue returning chunks of audio back to a background worker thread, or keep a pre-allocated "recording buffer" in the audio thread that captures the mix when a `Command::StartRecording` is received.

## Proposed Path Forward
1. **Refactor `cpal` thread** to use a lock-free message queue (`crossbeam-channel` or `ringbuf`) instead of `Arc<Mutex<...>>`.
2. **Implement Voice Pool & Mute Groups** for robust polyphony management without clicking.
3. **Establish an Intermediate Bus Mixer** to allow routing pads to different FX chains.
