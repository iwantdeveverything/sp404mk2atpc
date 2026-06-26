# Specification: BPM Sync

## Purpose
This specification defines how tempo information is gathered and distributed to beat-synced effects, such as the DJFX Looper, Scatter, Slicer, Sync Delay, and Ko-Da-Ma.

## Requirements

### Requirement: Manual BPM Input
The system MUST allow explicit manual entry of the tempo.
**Scenario:**
- **Given** the UI is active
- **When** the user manually enters a BPM value
- **Then** the system MUST update the global tempo to the specified value
- **And** immediately distribute the new tempo to all beat-synced effects.

### Requirement: Tap Tempo
The system MUST allow users to tap to set the tempo.
**Scenario:**
- **Given** the UI is active
- **When** the user repeatedly clicks the tap tempo button
- **Then** the system MUST calculate the average BPM based on the timing between taps
- **And** update the global tempo accordingly.

### Requirement: Tempo Distribution to Effects
Tempo changes MUST be propagated to all relevant audio effects.
**Scenario:**
- **Given** the global tempo has been updated
- **When** beat-synced effects (like DJFX Looper or Delay) are active
- **Then** these effects MUST adjust their internal timing (e.g., delay times or slice lengths) to perfectly match the new BPM.

### Requirement: Pre-listen Bypass (BPM Sync)
**Scenario:**
- **Given** pre-listen audio is being played from the file browser
- **When** the audio playback is initiated via the `pre_listen_start` command
- **Then** the pre-listen audio MUST bypass the BPM synchronization engine entirely
- **And** raw samples MUST play at their original pitch and speed without tempo adjustment
