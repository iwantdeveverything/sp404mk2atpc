# Specification: Effects Engine (Delta: sp404-effects-engine-robustness)

## MODIFIED Requirements

### Requirement: Effect Inventory
The engine MUST declare the full MFX catalog but MUST distinguish implemented effects from declared-but-unimplemented variants. This cycle ships a prioritized batch of 12 newly implemented effects (Chorus, Flanger, Phaser, Tremolo, AutoPan, Compressor, Equalizer, Distortion, Overdrive, Bitcrusher, LoFi, Wah) in addition to the existing 8 (Filter, Isolator, Delay, Reverb, VinylSim, DjfxLooper, Scatter, Slicer), for 20 implemented effects total. The remaining catalog variants MUST remain declared as roadmap but MUST NOT be presented as usable.
(Previously: required the complete suite of 37 effects with no distinction between implemented and unimplemented.)

**Scenario:**
- **Given** the effect catalog is declared
- **When** the engine reports which effects are usable
- **Then** the system MUST report exactly the 20 implemented effects as selectable
- **And** declared-but-unimplemented variants MUST NOT be reported as usable.

**Scenario:**
- **Given** any of the 20 implemented effects is selected
- **When** the audio thread processes a frame through that effect
- **Then** the effect MUST perform real audio processing (no no-op passthrough).

### Requirement: Wet/Dry Mix Control
Every effect slot MUST blend the dry input signal with the wet processed output according to a dedicated mix parameter. The mix MUST be applied as a cross-cutting layer at the effect-slot / EffectChain level so that ALL effects (existing 8 and new 12) inherit it uniformly. The mix MUST be a dedicated control that is ALWAYS present for an active slot and MUST be independent of the effect's own tunable parameters — it MUST NOT consume one of the effect's parameter slots.
(Previously: required per-effect mix control with no cross-cutting slot-level guarantee and no allocation constraint.)

**Scenario:**
- **Given** an active effect processing audio in a slot
- **When** the wet/dry mix parameter is adjusted
- **Then** the slot output MUST blend the unprocessed input with the processed signal according to the mix value (e.g. `out = dry*(1-mix) + wet*mix`).

**Scenario:**
- **Given** the mix value is set to 0.0
- **When** the slot processes a frame
- **Then** the output MUST equal the dry input (fully bypassed wet signal).

**Scenario:**
- **Given** any implemented effect occupies a slot
- **When** the UI inspects the slot's controls
- **Then** the mix control MUST be present regardless of how many tunable parameters the effect declares.

### Requirement: Zero Allocation Processing
The effect engine MUST NOT allocate memory during the audio callback, including the new wet/dry mix blend and parameter-normalization-on-write paths.
(Previously: covered only `process_frame`; did not name the mix blend or normalization paths.)

**Scenario:**
- **Given** an active effect chain
- **When** `process_frame`, the slot-level mix blend, or `set_parameter` (normalization-on-write) executes
- **Then** the operation MUST use pre-allocated state and MUST NOT trigger any heap allocations.

**Scenario:**
- **Given** an effect is being instantiated via `create_effect`
- **When** allocation occurs
- **Then** allocation MAY happen because `create_effect` runs off the audio thread.

## ADDED Requirements

### Requirement: Explicit Unimplemented Handling
`create_effect` MUST return an explicit `None` (or equivalent absent value) for any catalog variant that is not yet implemented. The system MUST NOT substitute a silent passthrough (`pass()`) for an unimplemented variant. The system MUST NOT present an effect to the user that performs no processing.

**Scenario:**
- **Given** a catalog variant that is not implemented this cycle
- **When** `create_effect` is called for that variant
- **Then** it MUST return `None` and MUST NOT return a passthrough effect.

**Scenario:**
- **Given** the engine is queried for usable effects
- **When** an unimplemented variant exists in the catalog
- **Then** that variant MUST NOT appear in the usable/implemented set.

### Requirement: Implemented-Only Selection
The engine MUST expose an authoritative list of implemented effects that is the single source of truth for the frontend selector. The selector MUST present only effects that are actually implemented, with no phantom or no-op entries.

**Scenario:**
- **Given** the frontend requests the list of selectable effects
- **When** the engine responds
- **Then** the response MUST contain exactly the implemented effects and MUST exclude every unimplemented variant.

### Requirement: Parameter Metadata Contract
Each implemented effect MUST declare metadata for every one of its tunable parameters: a stable parameter name, a minimum and maximum value expressed in the parameter's real unit, and a scaling curve (e.g. linear or exponential). The number of parameters MUST be allowed to vary per effect (variable N, not fixed at 3).

**Scenario:**
- **Given** an implemented effect with N tunable parameters
- **When** its metadata is queried
- **Then** the engine MUST return exactly N descriptors, each with a name, a min/max real-unit range, and a curve.

**Scenario:**
- **Given** two effects with different parameter counts (e.g. Compressor with 4, Tremolo with 2)
- **When** their metadata is queried
- **Then** the engine MUST report 4 descriptors for Compressor and 2 for Tremolo.

### Requirement: Normalized Parameter Mapping
The engine MUST map a normalized control value in the range 0.0–1.0 to the target parameter's real-unit range using the parameter's declared scaling curve, before the value is applied to the DSP node. Raw normalized values MUST NOT be written directly to real-unit DSP parameters.

**Scenario:**
- **Given** a parameter declared with range 200–8000 Hz and an exponential curve
- **When** a normalized value of 1.0 is received
- **Then** the engine MUST apply 8000 Hz to the DSP node (not 1.0 Hz).

**Scenario:**
- **Given** a parameter declared with a linear range
- **When** a normalized value of 0.5 is received
- **Then** the engine MUST apply the midpoint of the real-unit range.
