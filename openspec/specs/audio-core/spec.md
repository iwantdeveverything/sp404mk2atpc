# Specification: Audio Core Engine

## ADDED Requirements

### Requirement: Mute Group Choking
**Scenario:**
- **Given** a pad assigned to mute group 1 is currently playing audio
- **When** another pad assigned to mute group 1 is triggered
- **Then** the first pad's playback is immediately stopped
- **And** the second pad begins playback

### Requirement: FX Bus Routing
**Scenario:**
- **Given** a pad is routed to Bus 1
- **When** the pad is triggered and plays audio
- **Then** its audio is processed by the Bus 1 FX node before reaching the Master output
- **And** the pad-to-bus assignment MUST persist across multiple triggers of the pad.

### Requirement: Resampling Buffer Capture
**Scenario:**
- **Given** the resampling mode is armed
- **When** audio is sent to the Master output
- **Then** the exact same audio stream is also written to the internal record buffer

### Requirement: Lock-Free Execution
**Scenario:**
- **Given** the audio engine is running
- **When** the audio render callback is invoked
- **Then** the system guarantees no mutexes or blocking synchronization primitives are locked inside the callback
- **And** parameter updates and effect changes MUST be communicated via lock-free ring buffers (e.g., `rtrb`).

### Requirement: Lock-Free Parameter Control Commands
**Scenario:**
- **Given** the UI sends an effect update
- **When** the frontend dispatches a parameter change or effect swap
- **Then** the system MUST enqueue an `AudioCommand` (such as `SetBusEffect`, `SetEffectParam`, or `RemoveBusEffect`) into the lock-free queue for the audio thread to process.

### Requirement: Audio Engine State Extensions
**Scenario:**
- **Given** the audio engine is initializing
- **When** the audio state is constructed
- **Then** it MUST include fields for the effect chains and track the active `BusId` and `EffectType` for dynamic routing and parameter management.

### Requirement: Pre-listen Channel (Raw Audio Playback)
**Scenario:**
- **Given** the file browser is active and a user selects an audio file for preview
- **When** the pre-listen is triggered via the `pre_listen_start` IPC command
- **Then** an independent raw audio playback mechanism MUST play the audio through a dedicated pre-listen channel
- **And** this channel MUST explicitly bypass the FX routing and bus processing to ensure uncolored auditioning

### Requirement: Pre-listen Routing Bypass
**Scenario:**
- **Given** pre-listen audio is being played
- **When** the audio passes through the render callback
- **Then** the pre-listen channel MUST NOT be routed through any FX bus or Master bus effects
- **And** the pre-listen signal MUST pass directly to the output without coloration
