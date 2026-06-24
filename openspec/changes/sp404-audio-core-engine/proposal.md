# Proposal: Core Audio Engine

## Intent
Re-architect the Rust audio engine to use a lock-free ring-buffer based DSP graph to ensure zero dropouts and support complex routing.

## Scope
**In Scope:**
- Lock-free audio thread migration
- Voice pooling
- Mute group / choking logic
- FX Bus 1 & 2 + Master routing
- Resampling capture node

**Out of Scope:**
- The actual implementation of the specific FX algorithms like Isolator, only the routing is in scope for now.

## Capabilities
- `lock-free-engine`
- `fx-bus-routing`
- `mute-groups`
- `resample-node`
