# UI Routing and Resampling Specifications (Delta: sp404-effects-engine)

## MODIFIED Requirements

### Requirement: Hardware Routing
- **Given** the global "Bus 1" button is being held down
- **When** a pad is tapped
- **Then** the pad MUST be routed to Bus 1
- **And** the UI MUST display a bus target indicator showing which bus the pad is currently routed to.

## ADDED Requirements

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
