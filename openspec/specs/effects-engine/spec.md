# Specification: Effects Engine

## Purpose
This specification defines the core audio processing capabilities of the effects engine, bringing 37 MFX effects to transform the instrument. It describes the effect trait contract, effect chain structure, and the performance guarantees required for audio thread execution.

## Requirements

### Requirement: Effect Trait Interface
The system MUST define an `Effect` trait that encapsulates audio processing and parameter control.
**Scenario:**
- **Given** an instantiated effect
- **When** the audio thread requests processing
- **Then** the effect MUST expose `process_frame`, `set_parameter`, `reset`, and `set_sample_rate` methods.

### Requirement: Effect Chain Architecture
The audio engine MUST support multiple independent effect chains.
**Scenario:**
- **Given** the audio engine is running
- **When** audio is routed through the buses
- **Then** the system MUST provide an `EffectChain` with exactly 4 effect slots per bus for Bus 1, Bus 2, and the Master bus.

### Requirement: Effect Inventory
The engine MUST support the complete suite of MFX effects.
**Scenario:**
- **Given** an empty effect slot
- **When** the user selects an effect
- **Then** the system MUST allow selection from 37 distinct effects categorized into 9 categories.

### Requirement: Wet/Dry Mix Control
Every effect MUST support independent mix control.
**Scenario:**
- **Given** an active effect processing audio
- **When** the wet/dry mix parameter is adjusted
- **Then** the output audio MUST blend the unprocessed input with the processed signal according to the mix value.

### Requirement: Zero Allocation Processing
The effect engine MUST NOT allocate memory during the audio callback.
**Scenario:**
- **Given** an active effect chain
- **When** the `process_frame` method is executing
- **Then** the effect MUST use pre-allocated state and MUST NOT trigger any heap allocations.

### Requirement: Pre-allocated Effect State
Effect state MUST be established before audio processing begins.
**Scenario:**
- **Given** an effect is being instantiated
- **When** the effect is created
- **Then** all required memory buffers and internal states MUST be fully pre-allocated.

### Requirement: Pre-listen Isolation
**Scenario:**
- **Given** pre-listen audio is being played
- **When** the audio data passes through the render callback from the pre-listen channel
- **Then** the effects engine MUST NOT process audio originating from the pre-listen channel
- **And** the pre-listen signal MUST bypass all effect chains and reach the output unprocessed
