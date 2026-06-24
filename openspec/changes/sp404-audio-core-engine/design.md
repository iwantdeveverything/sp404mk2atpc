# Design: Core Audio Engine

## Architecture Decisions (Rust)

### 1. Ring Buffers vs Mutexes (Lock-free Communication)
**Decision**: Use `rtrb` (real-time ring buffer) for lock-free communication between the main UI thread and the audio thread.
**Rationale**: Audio threads have strict real-time constraints and cannot afford to block on Mutexes or standard channels. `rtrb` provides a lock-free, wait-free queue that is perfectly suited for passing control messages (Play, Stop, Route) without risking audio dropouts.

### 2. DSP Graph Structure
**Decision**: Implement a simple static pipeline structure instead of a dynamic generic graph.
**Pipeline Flow**: `Voice Array` -> `Bus 1 / Bus 2 / Dry` -> `Master FX` -> `Output`.
**Rationale**: A static pipeline is significantly faster and more cache-friendly than dynamic generic node-based graphs. The SP-404 style routing is mostly fixed (Pad to Bus 1, Bus 2, or Dry, then to Master FX), making a static architecture the most efficient and robust choice.

### 3. Mute Groups / Choking
**Decision**: Add a `mute_group` ID to the `Voice` struct.
**Rationale**: Mute groups (or choke groups) are essential for drum machines (e.g., closed hi-hat chokes open hi-hat). By embedding the `mute_group` ID in the `Voice` struct, the audio engine can simply scan active voices and immediately halt any voice sharing the same mute group ID when a new voice is triggered.

### 4. Resampling Buffer
**Decision**: Allocate a large static vector for the capture buffer at initialization, and use a lock-free boolean flag (e.g., `AtomicBool`) to trigger writing.
**Rationale**: Memory allocation on the audio thread causes immediate dropouts. Pre-allocating a large static vector ensures sufficient capacity for resampling sessions. A lock-free boolean flag provides a highly efficient, real-time safe mechanism to start and stop the capture process.
