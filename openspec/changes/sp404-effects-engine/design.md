# Design: SP-404MK2 Full Effects Engine

## 1. Technical Approach
The effects engine uses a **FunDSP Hybrid** approach. FunDSP provides the core DSP primitives (filters, delays, modulation, dynamics), while custom Rust code handles SP-404-specific effects requiring beat-sync, noise modeling, or buffer manipulation (e.g., VinylSim, DJFX Looper, Cassette Sim). A `FunDspEffect` wrapper bridges `AudioNode` structures to our standard `Effect` trait, performing per-frame processing within the audio thread.

## 2. Architecture Decisions

| Decision Area | Options | Tradeoffs | Decision |
|---|---|---|---|
| **Audio Thread Communication** | A: Mutex-locked state<br>B: `rtrb` lock-free ring buffer for `AudioCommand` | A: Simpler, but risks priority inversion & audio dropouts.<br>B: Guarantees zero-allocation, lock-free performance in audio thread, extending existing patterns. | **Option B**: Extend existing `rtrb` `AudioCommand` queue. |
| **Effect Chain Execution** | A: Block-based processing<br>B: Per-frame processing | A: More efficient (SIMD friendly), but requires massive engine rewrite.<br>B: Matches existing `engine.rs` architecture, easier to implement and verify. | **Option B**: Per-frame processing. |
| **BPM Sync Distribution** | A: Polling global state per effect<br>B: Push via `AudioCommand` | A: Simpler state, requires shared atomic variables.<br>B: Centralized control, strictly synchronized with the audio thread boundary. | **Option B**: Push via `AudioCommand`. |

## 3. Data Flow Diagram

```ascii
 Pad Buffers
      │
      ├── [Trigger Events] ──┐
      │                      │
      ▼                      ▼
[Mixer: Bus1]          [Mixer: Bus2]          [Mixer: Dry]
      │                      │                      │
      ▼                      ▼                      │
[EffectChain 1]        [EffectChain 2]              │
  (4 Slots)              (4 Slots)                  │
      │                      │                      │
      └──────────────────────┼──────────────────────┘
                             ▼
                       [Mixer: Master]
                             │
                             ▼
                      [Master EffectChain]
                          (4 Slots)
                             │
                             ▼
                     [Output/Resampling]
```

## 4. Interfaces & Contracts

```rust
pub trait Effect: Send + Sync {
    fn process_frame(&mut self, frame: &mut [f32; 2]);
    fn set_parameter(&mut self, param_id: u8, value: f32);
    fn reset(&mut self);
    fn set_sample_rate(&mut self, rate: u32);
}

pub struct EffectChain {
    pub slots: [Option<Box<dyn Effect>>; 4],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EffectType {
    Isolator,
    DjfxLooper,
    VinylSim,
    // ... 34 others
}

// Extended AudioCommand
pub enum AudioCommand {
    // ... existing ...
    SetBusEffect { bus: BusRouting, slot: usize, effect: EffectType },
    SetEffectParam { bus: BusRouting, slot: usize, param_id: u8, value: f32 },
    RemoveBusEffect { bus: BusRouting, slot: usize },
    SetTempo { bpm: f32 },
}
```

## 5. File Changes

| File | Change Type | Description |
|---|---|---|
| `src-tauri/src/audio/effects/mod.rs` | New | Effect trait, chain, factory, bridge, and effect implementations |
| `src-tauri/src/audio/engine.rs` | Modify | Add FX chain fields to `AudioEngineThreadState`, integrate processing at `L138-145` |
| `src-tauri/src/audio/state.rs` | Modify | Add new `AudioCommand` variants, `EffectType` enum, extend `BusRouting` |
| `src-tauri/src/audio/mod.rs` | Modify | Declare `pub mod effects;` |
| `src-tauri/src/lib.rs` | Modify | Add Tauri commands: `set_bus_effect`, `set_effect_param` |
| `src-tauri/Cargo.toml` | Modify | Add `fundsp` and serialization dependencies |
| `src/main.ts` | Modify | Add effect selector UI, CTRL 1-3 knobs, BPM input/tap tempo |
| `src/styles.css` | Modify | Styling for knobs, selector, and param display |

## 6. Testing Strategy
- **Audio Thread Safety:** Unit tests to verify zero allocations in `process_frame` via `assert_no_alloc`.
- **Instantiations:** Unit tests confirming FunDSP graph instantiations do not panic and respect `set_sample_rate`.
- **Integration:** Test that `SetBusEffect` and `SetEffectParam` correctly pass through the ring buffer and manipulate the internal `EffectChain` without dropping events.
- **Manual Verification:** Confirm BPM tap tempo dynamically updates beat-synced effects (like Scatter) without audio tearing.

## 7. Migration & Rollout
1. **Phase 1 (Infrastructure):** Define `Effect` trait, `EffectChain`, `fundsp` integration. Extend `AudioCommand` and add states to `AudioEngineThreadState`.
2. **Phase 2 (Core Effects):** Implement the first 5 core effects (Filter, Isolator, Delay, Reverb, VinylSim) and test basic bus routing.
3. **Phase 3 (UI Integration):** Build frontend effect selector, 3 rotary knobs (CTRL 1-3), and wire them to the new Tauri commands.
4. **Phase 4 (BPM & Beat Sync):** Implement manual/tap BPM. Build beat-synced effects (DJFX Looper, Scatter, Slicer).
5. **Phase 5 (Arsenal & Persistence):** Implement the remaining 29 effects and wire serialization/deserialization to persist configs on app restart.
