# UI Juice Enhancement Specifications (Delta: sp404-effects-engine-robustness)

## ADDED Requirements

### Requirement: Parameter-Aware Controls
The UI MUST render an effect's parameter controls dynamically from the queried parameter metadata: one control per declared parameter, labeled with the parameter's declared name, and bounded by the declared real-unit range. The number of controls MUST follow the metadata count (variable N) rather than a fixed number.

**Scenario:**
- **Given** the Compressor effect declaring 4 parameters (threshold, ratio, attack, release)
- **When** the UI renders its controls
- **Then** the UI MUST display 4 controls labeled threshold, ratio, attack, and release.

**Scenario:**
- **Given** the Tremolo effect declaring 2 parameters
- **When** the UI renders its controls
- **Then** the UI MUST display exactly 2 parameter controls with their declared names.

**Scenario:**
- **Given** parameter metadata declares a name and real-unit range for a parameter
- **When** the corresponding control is rendered
- **Then** the control MUST show the declared name and MUST be bounded by the declared range.

### Requirement: Dedicated Mix Control
The UI MUST present a wet/dry mix control that is always visible whenever an effect is active, rendered separately from the effect's parameter knobs. The mix control MUST NOT be one of the effect's parameter controls.

**Scenario:**
- **Given** any effect is active in a slot
- **When** the UI renders the slot's controls
- **Then** a dedicated wet/dry mix control MUST be visible
- **And** it MUST be distinct from the effect's parameter knobs.

**Scenario:**
- **Given** an effect that declares zero or many tunable parameters
- **When** the UI renders the slot
- **Then** the mix control MUST still be present regardless of the parameter count.
