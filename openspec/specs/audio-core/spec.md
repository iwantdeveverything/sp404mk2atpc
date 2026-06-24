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
