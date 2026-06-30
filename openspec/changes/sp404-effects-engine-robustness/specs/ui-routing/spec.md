# UI Routing and Resampling Specifications (Delta: sp404-effects-engine-robustness)

## MODIFIED Requirements

### Requirement: Effect Selector UI
The LCD area MUST provide an effect selector interface whose list of selectable effects is sourced from the engine's implemented-effects set (the single source of truth). The `set_bus_effect` string→`EffectType` mapping MUST cover all 20 implemented effects (the existing 8 plus the 12 new: Chorus, Flanger, Phaser, Tremolo, AutoPan, Compressor, Equalizer, Distortion, Overdrive, Bitcrusher, LoFi, Wah). The selector MUST NOT present unimplemented variants.
(Previously: offered a fixed list of 37 MFX effects regardless of which were implemented.)

**Scenario:**
- **Given** the UI is active
- **When** the user opens the effect selector
- **Then** the selector MUST list exactly the effects reported as implemented by the engine
- **And** it MUST NOT list any unimplemented catalog variant.

**Scenario:**
- **Given** the user selects any of the 20 implemented effects
- **When** `set_bus_effect` is invoked with that effect's string identifier
- **Then** the mapping MUST resolve it to the corresponding `EffectType`
- **And** the effect MUST become active on the target bus.

### Requirement: Rotary Knob Controls
When an effect is active on a bus, the UI MUST provide rotary knob controls to adjust the current effect's parameters in real-time. The number of knobs MUST match the count of parameters declared in the active effect's metadata (variable N), rather than a fixed count.
(Previously: required exactly 3 rotary knobs CTRL 1–3, which conflicts with the per-effect variable parameter count introduced this cycle.)

**Scenario:**
- **Given** an effect with N tunable parameters is active on a bus
- **When** the UI renders its controls
- **Then** there MUST be N rotary controls available, one per declared parameter.

**Scenario:**
- **Given** the active effect declares 2 parameters
- **When** the user interacts with the controls
- **Then** exactly 2 parameter knobs MUST be presented and adjustable in real-time.
