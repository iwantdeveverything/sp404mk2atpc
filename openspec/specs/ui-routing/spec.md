# UI Routing and Resampling Specifications

## ADDED Requirements

### Requirement: Hardware Routing
- **Given** the global "Bus 1" button is being held down
- **When** a pad is tapped
- **Then** the pad MUST be routed to Bus 1
- **And** the UI MUST display a bus target indicator showing which bus the pad is currently routed to.

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

### Requirement: Effect Selector UI
- **Given** the UI is active
- **When** the user wishes to change an effect
- **Then** the LCD area MUST provide an effect selector interface allowing the choice of 37 MFX effects.

### Requirement: Rotary Knob Controls
- **Given** an effect is active on a bus
- **When** the user interacts with the UI
- **Then** there MUST be exactly 3 rotary knobs (CTRL 1, CTRL 2, CTRL 3) available to adjust the current effect's parameters in real-time.

### Requirement: BPM Input Controls
- **Given** the UI is active
- **When** tempo adjustments are needed
- **Then** the interface MUST provide both a manual BPM numeric input and a dedicated tap tempo button.
