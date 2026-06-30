# Specification: Audio Core Engine (Delta: sp404-effects-engine-robustness)

## MODIFIED Requirements

### Requirement: Lock-Free Parameter Control Commands
The lock-free `AudioCommand` path MUST carry enough information to drive normalized parameters and the dedicated per-slot wet/dry mix. The frontend MUST dispatch parameter changes, mix changes, and effect swaps as `AudioCommand` variants enqueued into the lock-free queue. The audio thread MUST apply both normalized parameter values and the wet/dry mix value received via this lock-free mechanism, without locking inside the callback.
(Previously: enumerated `SetBusEffect`, `SetEffectParam`, and `RemoveBusEffect` only, with no mention of normalized parameters or a dedicated mix value.)

**Scenario:**
- **Given** the UI sends an effect parameter update
- **When** the frontend dispatches a parameter change or effect swap
- **Then** the system MUST enqueue an `AudioCommand` (such as `SetBusEffect`, `SetEffectParam`, or `RemoveBusEffect`) into the lock-free queue for the audio thread to process
- **And** the command MUST carry the information needed to identify the target parameter and apply its normalized value.

**Scenario:**
- **Given** the user adjusts the wet/dry mix of an active slot
- **When** the change is dispatched
- **Then** the mix value MUST be communicated to the audio thread via the same lock-free mechanism
- **And** no mutex or blocking primitive MUST be locked inside the audio callback while the mix value is applied.

## ADDED Requirements

### Requirement: Parameter Metadata Query
The backend MUST expose an IPC command that returns the parameter metadata (per-parameter name, min/max real-unit range, and scaling curve) for a given effect, so the frontend can render N parameter controls with names and ranges. This query MUST execute off the audio thread and MUST NOT interfere with the lock-free audio callback.

**Scenario:**
- **Given** the frontend needs to render controls for an effect
- **When** it issues the parameter-metadata IPC query for that effect
- **Then** the backend MUST return the ordered descriptors (name, min, max, curve) for every tunable parameter of that effect.

**Scenario:**
- **Given** the parameter-metadata query is invoked
- **When** the backend resolves the metadata
- **Then** the read MUST occur off the audio thread and MUST NOT block or lock the audio render callback.
