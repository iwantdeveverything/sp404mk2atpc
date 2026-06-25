# Specification: Audio Core Engine (Delta: sp404-effects-engine)

## MODIFIED Requirements

### Requirement: FX Bus Routing
**Scenario:**
- **Given** a pad is routed to Bus 1
- **When** the pad is triggered and plays audio
- **Then** its audio is processed by the Bus 1 FX node before reaching the Master output
- **And** the pad-to-bus assignment MUST persist across multiple triggers of the pad.

### Requirement: Lock-Free Execution
**Scenario:**
- **Given** the audio engine is running
- **When** the audio render callback is invoked
- **Then** the system guarantees no mutexes or blocking synchronization primitives are locked inside the callback
- **And** parameter updates and effect changes MUST be communicated via lock-free ring buffers (e.g., `rtrb`).

## ADDED Requirements

### Requirement: Lock-Free Parameter Control Commands
**Scenario:**
- **Given** the UI sends an effect update
- **When** the frontend dispatches a parameter change or effect swap
- **Then** the system MUST enqueue an `AudioCommand` (such as `SetBusEffect`, `SetEffectParam`, or `RemoveBusEffect`) into the lock-free queue for the audio thread to process.

### Requirement: Audio Engine State Extensions
**Scenario:**
- **Given** the audio engine is initializing
- **When** the audio state is constructed
- **Then** it MUST include fields for the effect chains and track the active `BusId` and `EffectType` for dynamic routing and parameter management.
