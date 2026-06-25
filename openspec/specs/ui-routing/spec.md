# UI Routing and Resampling Specifications

## ADDED Requirements

### Requirement: Hardware Routing
- **Given** the global "Bus 1" button is being held down
- **When** a pad is tapped
- **Then** the pad MUST be routed to Bus 1

### Requirement: Resample Arming
- **Given** the resampler is idle
- **When** the "Resample" button is clicked
- **Then** the resampler MUST enter the armed state
- **And** the "Resample" button MUST blink deep red
- **And** the LCD MUST type out `[RESAMPLING ARMED]`
- **And** the LCD background MUST change to a reddish tint

### Requirement: Recording State
- **Given** the resampler is in the armed state
- **When** a pad is hit
- **Then** the resampler MUST enter the recording state
- **And** the "Resample" button MUST stay solid red
- **And** the system MUST actively record the audio buffer
