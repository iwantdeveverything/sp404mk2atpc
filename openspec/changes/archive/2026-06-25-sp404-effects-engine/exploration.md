# Exploration: SP-404MK2 Full Effects Engine

**Change**: `sp404-effects-engine`  
**Date**: 2026-06-24  
**Status**: Exploration  

---

## 1. Current Architecture Analysis

### 1.1 Audio Engine Overview

The current audio engine lives in three files under `src-tauri/src/audio/`:

| File | Purpose |
|------|---------|
| [engine.rs](file:///home/hstrejoluna/Projects/sp404mk2atpc/src-tauri/src/audio/engine.rs) | Real-time audio callback, DSP graph, bus mixing, resampling capture |
| [state.rs](file:///home/hstrejoluna/Projects/sp404mk2atpc/src-tauri/src/audio/state.rs) | Audio state, commands, playback events, bus routing enum |
| [mod.rs](file:///home/hstrejoluna/Projects/sp404mk2atpc/src-tauri/src/audio/mod.rs) | Module declaration (currently only `engine` + `state`) |

### 1.2 Signal Flow (Current)

```
┌─────────────────────────────────────────────────────────┐
│  UI Thread                                               │
│  ┌─────────┐    rtrb (lock-free)    ┌──────────────────┐ │
│  │ AudioState├──────────────────────►│ AudioCommand     │ │
│  │ .command_tx│  SPSC ring buffer    │ consumer in      │ │
│  └─────────┘                        │ audio thread     │ │
│                                     └──────────────────┘ │
└─────────────────────────────────────────────────────────┘

Audio Thread (write_data callback):
  1. Pop commands → AddBuffer | TriggerPad
  2. For each frame:
     a. Accumulate samples into bus1_mix[], bus2_mix[], dry_mix[]
     b. Sum all three buses (PLACEHOLDER at lines 138-145)
     c. Clamp to [-1.0, 1.0]
     d. Write to output + optional resampling capture
  3. Remove finished PlaybackEvents
```

### 1.3 Key Integration Point

[engine.rs lines 138-145](file:///home/hstrejoluna/Projects/sp404mk2atpc/src-tauri/src/audio/engine.rs#L138-L145) contain the explicit FX processing placeholder:

```rust
let mut frame_mix = vec![0.0; channels];
for c in 0..channels {
    // Apply Bus1 FX -> Apply Bus2 FX -> Dry -> Master FX
    // (Placeholder for actual FX node processing)
    let master_in = bus1_mix[c] + bus2_mix[c] + dry_mix[c];
    // Master FX Processing would go here
    frame_mix[c] = master_in;
}
```

### 1.4 Current Command System

The existing `AudioCommand` enum in [state.rs](file:///home/hstrejoluna/Projects/sp404mk2atpc/src-tauri/src/audio/state.rs) only supports:
- `AddBuffer(pad_id, Arc<AudioBuffer>)` — load sample data
- `TriggerPad { pad_id, mute_group, routing }` — trigger playback

The command channel uses `rtrb` (SPSC ring buffer) with a capacity of 1024 — this is already the correct lock-free pattern for real-time audio.

### 1.5 Frontend State

[main.ts](file:///home/hstrejoluna/Projects/sp404mk2atpc/src/main.ts) already has Bus1/Bus2 button handling (hold-to-route pattern at lines 77-85). The Tauri command `set_pad_bus` exists in [lib.rs](file:///home/hstrejoluna/Projects/sp404mk2atpc/src-tauri/src/lib.rs#L28-L31) but currently only prints to console — it does not actually route in the engine.

### 1.6 Dependencies

From [Cargo.toml](file:///home/hstrejoluna/Projects/sp404mk2atpc/src-tauri/Cargo.toml):
- `cpal = "0.18.1"` — audio I/O
- `rtrb = "0.3.4"` — lock-free SPSC ring buffer
- `hound = "3.5.1"` — WAV loading
- `minimp3 = "0.6.1"` — MP3 decoding

No DSP crates are present yet.

---

## 2. SP-404MK2 Effects Catalog

The SP-404MK2 has **37 core MFX** effects (+ additional firmware-update effects). Each effect has 3 primary control knobs (CTRL 1-3) and many have a second page of sub-parameters.

### 2.1 Complete Effects by Category

#### Filter & EQ (5 effects)

| # | Effect | Description | CTRL 1 | CTRL 2 | CTRL 3 |
|---|--------|-------------|--------|--------|--------|
| 1 | **Filter+Drive** | Resonant LP/HP/BP filter with saturation drive | Cutoff | Resonance | Drive |
| 2 | **Isolator** | 3-band DJ kill EQ (Low/Mid/Hi independent volume) | Low | Mid | High |
| 3 | **Super Filter** | Aggressive resonant filter sweep with envelope | Cutoff | Resonance | Type |
| 4 | **Equalizer** | Parametric EQ for tone shaping | Low | Mid | High |
| 5 | **Hyper-Reso** | Extreme resonance filter for metallic/harmonic textures | Frequency | Resonance | Character |

#### Delay & Echo (5 effects)

| # | Effect | Description | CTRL 1 | CTRL 2 | CTRL 3 |
|---|--------|-------------|--------|--------|--------|
| 6 | **Sync Delay** | Tempo-synced digital delay | Time | Feedback | Level |
| 7 | **Tape Echo** | Vintage tape echo with wow/flutter degradation | Time | Feedback | Tone |
| 8 | **TimeCtrlDly** | Real-time delay time control with pitch artifacts | Time | Feedback | Level |
| 9 | **Ko-Da-Ma** | Multi-tap rhythmic delay (Japanese: "echo") | Pattern | Feedback | Level |
| 10 | **DJFX Looper** | Performance loop capture with stutter/repeat | Length | Speed | Gate |

#### Reverb (2 effects)

| # | Effect | Description | CTRL 1 | CTRL 2 | CTRL 3 |
|---|--------|-------------|--------|--------|--------|
| 11 | **Reverb** | Hall/Room/Plate reverb with pre-delay | Time | Tone | Level |
| 12 | **Zan-Zou** | Shimmer/granular reverb (Japanese: "afterimage") | Time | Shimmer | Level |

#### Modulation (5 effects)

| # | Effect | Description | CTRL 1 | CTRL 2 | CTRL 3 |
|---|--------|-------------|--------|--------|--------|
| 13 | **Chorus** | Classic chorus with depth and rate control | Rate | Depth | Level |
| 14 | **JUNO Chorus** | Emulation of Roland JUNO-106 BBD chorus | Mode (I/II/I+II) | Rate | Level |
| 15 | **Flanger** | Through-zero flanging with feedback | Rate | Depth | Feedback |
| 16 | **Phaser** | Multi-stage all-pass phaser | Rate | Depth | Feedback |
| 17 | **Wah** | Auto-wah / envelope follower | Sensitivity | Frequency | Level |

#### Dynamics (2 effects)

| # | Effect | Description | CTRL 1 | CTRL 2 | CTRL 3 |
|---|--------|-------------|--------|--------|--------|
| 18 | **Compressor** | Bus/master dynamics compressor | Threshold | Ratio | Gain |
| 19 | **Tremolo/Pan** | Volume tremolo and stereo auto-pan | Rate | Depth | Shape |

#### Distortion & Saturation (4 effects)

| # | Effect | Description | CTRL 1 | CTRL 2 | CTRL 3 |
|---|--------|-------------|--------|--------|--------|
| 20 | **Overdrive** | Soft-clipping tube-style overdrive | Drive | Tone | Level |
| 21 | **Distortion** | Hard-clipping distortion with tone shaping | Drive | Tone | Level |
| 22 | **WrmSaturator** | Warm analog-style saturation circuit | Drive | Warmth | Level |
| 23 | **Crusher** | Bit depth reduction + sample rate reduction | Bit Depth | Sample Rate | Level |

#### Lo-Fi & Vinyl (4 effects)

| # | Effect | Description | CTRL 1 | CTRL 2 | CTRL 3 |
|---|--------|-------------|--------|--------|--------|
| 24 | **Lo-fi** | Combined lo-fi degradation (noise, filtering, distortion) | Depth | Tone | Level |
| 25 | **303 VinylSim** | TR-303 era vinyl simulation with crackle/noise | Crackle | Warp | Level |
| 26 | **404 VinylSim** | SP-404 signature vinyl simulation | Crackle | Noise | Depth |
| 27 | **Cassette Sim** | Tape cassette degradation with wow/flutter/hiss | Wow/Flutter | Hiss | Saturation |

#### Performance & DJ (5 effects)

| # | Effect | Description | CTRL 1 | CTRL 2 | CTRL 3 |
|---|--------|-------------|--------|--------|--------|
| 28 | **Scatter** | Beat-synced glitch/stutter with random patterns | Type | Depth | Gate |
| 29 | **Slicer** | Rhythmic volume gate chopping | Pattern | Rate | Depth |
| 30 | **Ring Mod** | Ring modulation for metallic/inharmonic textures | Frequency | Balance | Level |
| 31 | **Chromatic PS** | Chromatic pitch shifter (+/- semitones) | Pitch | Fine | Level |
| 32 | **Stopper** | Turntable stop/start effect (speed ramp to zero) | Speed | Direction | Curve |

#### Spatial & Special (3 effects)

| # | Effect | Description | CTRL 1 | CTRL 2 | CTRL 3 |
|---|--------|-------------|--------|--------|--------|
| 33 | **Resonator** | Pitched resonance / comb filter harmonics | Pitch | Feedback | Level |
| 34 | **SBF** | Side Band Filter — frequency-shifting metallic effect | Frequency | Feedback | Level |
| 35 | **Downer** | Pitch-dropping effect (downward sweep) | Speed | Depth | Feedback |

#### Special Character (2 effects)

| # | Effect | Description | CTRL 1 | CTRL 2 | CTRL 3 |
|---|--------|-------------|--------|--------|--------|
| 36 | **Ha-Dou** | Granular wave effect (Japanese: "wave motion") | Grain | Pitch | Level |
| 37 | **To-Gu-Ro** | Spiral/swirl modulation effect (Japanese: "coil") | Rate | Depth | Feedback |

### 2.2 Firmware Update Effects (Post-Launch)

These were added via firmware updates and should be considered for a second phase:

| Effect | Description |
|--------|-------------|
| **SX Reverb** | Enhanced reverb from SX series |
| **SX Delay** | Enhanced delay from SX series |
| **Cloud Delay** | Granular cloud-based delay |
| **Back Spin** | Turntable backspin simulation |
| **DJFX Delay** | Performance-oriented DJ delay |
| **Auto Pitch** | Real-time pitch correction (Input FX only) |
| **Vocoder** | Voice synthesis vocoder (Input FX only) |
| **Harmony** | Intelligent pitch harmony (Input FX only) |
| **Gt Amp Sim** | Guitar amplifier simulation (Input FX only) |

### 2.3 Implementation Priority

Based on SP-404MK2 usage patterns and DSP complexity:

**Phase 1 — Core** (most used, simplest DSP):
1. Filter+Drive, Isolator, Compressor, Overdrive, Distortion, Lo-fi, Crusher, Equalizer

**Phase 2 — Character** (moderate complexity):
2. Reverb, Sync Delay, Tape Echo, Chorus, JUNO Chorus, Flanger, Phaser, Tremolo/Pan, WrmSaturator

**Phase 3 — Vinyl/Lo-Fi Signature** (SP-404's identity):
3. 303 VinylSim, 404 VinylSim, Cassette Sim, Wah, Ring Mod, Resonator

**Phase 4 — Performance** (complex, requires beat-sync):
4. Scatter, Slicer, DJFX Looper, Chromatic PS, Stopper, Downer, Backspin

**Phase 5 — Exotic** (complex granular/spectral):
5. Ha-Dou, To-Gu-Ro, Ko-Da-Ma, Zan-Zou, SBF, Hyper-Reso, Super Filter, TimeCtrlDly

---

## 3. DSP Architecture

### 3.1 Effect Trait Design

The core abstraction for all effects:

```rust
/// Core trait for all audio effects in the SP-404MK2 engine.
/// Designed for real-time, allocation-free processing.
pub trait Effect: Send {
    /// Process a stereo frame in-place. Called once per sample frame.
    /// `left` and `right` are mutable references to the current sample.
    fn process_frame(&mut self, left: &mut f32, right: &mut f32);

    /// Process a block of interleaved stereo samples.
    /// Default implementation calls process_frame per frame.
    /// Override for SIMD-optimized block processing.
    fn process_block(&mut self, buffer: &mut [f32], channels: usize) {
        for frame in buffer.chunks_mut(channels) {
            if channels >= 2 {
                self.process_frame(&mut frame[0], &mut frame[1]);
            } else {
                self.process_frame(&mut frame[0], &mut frame[0]);
            }
        }
    }

    /// Apply a parameter change received from the UI thread.
    /// Called in the audio thread context — must be non-blocking.
    fn set_parameter(&mut self, param_id: u8, value: f32);

    /// Get current parameter value for UI sync-back.
    fn get_parameter(&self, param_id: u8) -> f32;

    /// Return the effect type identifier.
    fn effect_type(&self) -> EffectType;

    /// Reset internal state (delay lines, filter history, etc.)
    fn reset(&mut self);

    /// Notify the effect of the current sample rate.
    fn set_sample_rate(&mut self, sample_rate: f32);
}
```

### 3.2 Effect Chain

Each bus holds an ordered chain of effect slots:

```rust
/// Maximum effects per bus chain. The real SP-404MK2 uses 1 MFX per bus,
/// but we support chains for flexibility.
const MAX_CHAIN_SLOTS: usize = 4;

pub struct EffectChain {
    slots: [Option<Box<dyn Effect>>; MAX_CHAIN_SLOTS],
    active_count: usize,
    sample_rate: f32,
}

impl EffectChain {
    pub fn process_frame(&mut self, left: &mut f32, right: &mut f32) {
        for slot in self.slots.iter_mut() {
            if let Some(effect) = slot {
                effect.process_frame(left, right);
            }
        }
    }

    pub fn set_effect(&mut self, slot: usize, effect: Option<Box<dyn Effect>>) {
        if slot < MAX_CHAIN_SLOTS {
            if let Some(ref mut fx) = effect {
                fx.set_sample_rate(self.sample_rate);
            }
            self.slots[slot] = effect;
        }
    }
}
```

### 3.3 Lock-Free Parameter Control

Three approaches for getting parameter changes from UI to audio thread:

#### Option A: Extend existing `AudioCommand` enum (Recommended)

```rust
pub enum AudioCommand {
    AddBuffer(usize, Arc<AudioBuffer>),
    TriggerPad { pad_id: usize, mute_group: Option<u8>, routing: BusRouting },
    // NEW: Effect commands
    SetBusEffect { bus: BusId, slot: usize, effect_type: EffectType },
    SetEffectParam { bus: BusId, slot: usize, param_id: u8, value: f32 },
    RemoveBusEffect { bus: BusId, slot: usize },
    SetMasterEffect { slot: usize, effect_type: EffectType },
    SetMasterParam { slot: usize, param_id: u8, value: f32 },
}
```

**Pro**: Uses existing `rtrb` channel, no new dependencies, consistent pattern.
**Con**: All commands share one ring buffer — high-frequency knob twiddling could fill it.

#### Option B: Separate parameter ring buffer

Dedicated `rtrb` channel for high-frequency parameter changes, keeping the command channel for structural changes (load/trigger/route).

```rust
pub enum ParamCommand {
    SetParam { bus: BusId, slot: usize, param_id: u8, value: f32 },
}
// Separate rtrb::Consumer<ParamCommand> in AudioEngineThreadState
```

**Pro**: Knob twiddling won't block sample loading. Higher throughput.
**Con**: Two consumers to poll per callback. Slightly more complexity.

#### Option C: Atomic parameter arrays

Each effect holds `Arc<AtomicF32>` parameters that UI writes directly.

```rust
pub struct AtomicParams {
    values: [AtomicU32; 6], // Bit-cast f32 -> u32 for atomic storage
}
```

**Pro**: Zero-latency parameter updates, no queue processing delay.
**Con**: No ordering guarantees, harder to batch related parameter changes, requires `Arc` sharing.

> **Recommendation**: Start with **Option A** (extend `AudioCommand`). The existing ring buffer capacity of 1024 is more than sufficient for typical parameter update rates (60 Hz UI → 60 commands/sec vs 1024 capacity). Move to Option B only if profiling shows contention.

### 3.4 Per-Sample vs Per-Block Processing

| Approach | Latency | CPU Cache | SIMD Potential | Implementation |
|----------|---------|-----------|----------------|----------------|
| **Per-sample** | Minimal | Poor (function call overhead per sample) | None | Simple, current architecture |
| **Per-block** | Block-size dependent | Excellent | Full SIMD vectorization | Requires buffer management |

**Recommendation**: Design the `Effect` trait with both `process_frame` (per-sample) and `process_block` (per-block) methods. Start with per-frame processing to match the current `write_data` architecture, which already iterates frame-by-frame. Migrate to block processing later for CPU optimization.

The current engine processes frame-by-frame in `output.chunks_mut(channels)`. This naturally supports per-frame effect processing without restructuring.

### 3.5 State Management for Stateful Effects

Effects like delays, reverbs, and filters maintain internal state between calls:

```rust
/// Example: Simple delay line with pre-allocated circular buffer
pub struct DelayEffect {
    buffer: Vec<f32>,        // Pre-allocated at construction time
    write_pos: usize,
    delay_samples: usize,
    feedback: f32,
    mix: f32,
    sample_rate: f32,
}

/// Example: Biquad filter state
pub struct BiquadFilter {
    // Coefficients
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
    // State (per channel)
    x1: [f32; 2], x2: [f32; 2],  // Input history
    y1: [f32; 2], y2: [f32; 2],  // Output history
}
```

**Critical rule**: All state memory (delay line buffers, filter history arrays) MUST be pre-allocated at effect construction time. Zero allocations in the audio callback.

### 3.6 Sample Rate Awareness

Effects must recalculate coefficients when sample rate changes:

```rust
impl BiquadFilter {
    fn recalculate_coefficients(&mut self) {
        let omega = 2.0 * PI * self.frequency / self.sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * self.q);
        // ... coefficient calculation based on filter type
    }
}
```

The sample rate is available as `target_sample_rate` in the `write_data` function (currently `u32`, will need conversion to `f32`). It should be passed to each `EffectChain` at construction time and propagated to all effects.

### 3.7 Analog Circuit Modeling Quality

For authentic SP-404MK2 character, certain DSP techniques are essential:

| Technique | Used In | Implementation |
|-----------|---------|----------------|
| **Soft saturation** | WrmSaturator, Tape Echo, Overdrive | `tanh(x)` waveshaping or polynomial approximation |
| **Wow & flutter** | Cassette Sim, VinylSim | Low-frequency modulated delay line (0.5-3 Hz) |
| **Noise injection** | VinylSim, Lo-fi, Cassette | Filtered noise generator (pink/brown noise) |
| **Sample rate reduction** | Crusher, Lo-fi | Sample-and-hold with variable period |
| **Bit depth reduction** | Crusher | Quantization: `floor(x * levels) / levels` |
| **Circuit resonance** | Filter+Drive, Super Filter | State Variable Filter (SVF) with nonlinear feedback |
| **BBD emulation** | JUNO Chorus | Bucket Brigade Delay: short delay line with clock noise |

---

## 4. Integration Architecture

### 4.1 Modified Signal Flow

```
Per-frame processing:

  Pad Events ──► Sample Accumulation by BusRouting
                      │
            ┌─────────┼──────────┐
            ▼         ▼          ▼
        bus1_mix   bus2_mix   dry_mix
            │         │          │
            ▼         ▼          │
    ┌──────────┐ ┌──────────┐   │
    │ Bus1 FX  │ │ Bus2 FX  │   │
    │ Chain    │ │ Chain    │   │
    └────┬─────┘ └────┬─────┘   │
         │            │          │
         ▼            ▼          ▼
         └──────┬─────┘──────────┘
                ▼
         ┌──────────┐
         │ Master   │
         │ FX Chain │
         └────┬─────┘
              ▼
         ┌──────────┐
         │ Clamp    │
         │ ±1.0     │
         └────┬─────┘
              ▼
         Output + Resample Capture
```

### 4.2 Modified `AudioEngineThreadState`

```rust
struct AudioEngineThreadState {
    // Existing fields
    buffers: HashMap<usize, Arc<AudioBuffer>>,
    active_events: Vec<PlaybackEvent>,
    command_rx: Consumer<AudioCommand>,
    resampling_buffer: Vec<f32>,
    resampling_index: usize,
    resampling_armed: Arc<AtomicBool>,

    // NEW: Effect chains
    bus1_fx: EffectChain,
    bus2_fx: EffectChain,
    master_fx: EffectChain,

    // NEW: Effect factory for constructing effects from commands
    sample_rate: f32,
}
```

### 4.3 Modified `write_data` Integration (lines 138-145)

```rust
// BEFORE (current placeholder):
let mut frame_mix = vec![0.0; channels];
for c in 0..channels {
    let master_in = bus1_mix[c] + bus2_mix[c] + dry_mix[c];
    frame_mix[c] = master_in;
}

// AFTER (with FX processing):
// Process Bus1 FX chain on bus1_mix
let (mut bus1_l, mut bus1_r) = (bus1_mix[0], if channels > 1 { bus1_mix[1] } else { bus1_mix[0] });
thread_state.bus1_fx.process_frame(&mut bus1_l, &mut bus1_r);

// Process Bus2 FX chain on bus2_mix
let (mut bus2_l, mut bus2_r) = (bus2_mix[0], if channels > 1 { bus2_mix[1] } else { bus2_mix[0] });
thread_state.bus2_fx.process_frame(&mut bus2_l, &mut bus2_r);

// Sum: processed Bus1 + processed Bus2 + dry (unprocessed)
let mut master_l = bus1_l + bus2_l + dry_mix[0];
let mut master_r = bus1_r + bus2_r + if channels > 1 { dry_mix[1] } else { dry_mix[0] };

// Process Master FX chain
thread_state.master_fx.process_frame(&mut master_l, &mut master_r);

let frame_mix = if channels > 1 {
    vec![master_l, master_r]
} else {
    vec![master_l]
};
```

### 4.4 Command Processing Extension

In `write_data`, extend the command processing loop:

```rust
while let Ok(command) = thread_state.command_rx.pop() {
    match command {
        AudioCommand::AddBuffer(pad_id, buffer) => { /* existing */ },
        AudioCommand::TriggerPad { .. } => { /* existing */ },

        // NEW
        AudioCommand::SetBusEffect { bus, slot, effect_type } => {
            let effect = create_effect(effect_type, thread_state.sample_rate);
            match bus {
                BusId::Bus1 => thread_state.bus1_fx.set_effect(slot, Some(effect)),
                BusId::Bus2 => thread_state.bus2_fx.set_effect(slot, Some(effect)),
                BusId::Master => thread_state.master_fx.set_effect(slot, Some(effect)),
            }
        },
        AudioCommand::SetEffectParam { bus, slot, param_id, value } => {
            let chain = match bus {
                BusId::Bus1 => &mut thread_state.bus1_fx,
                BusId::Bus2 => &mut thread_state.bus2_fx,
                BusId::Master => &mut thread_state.master_fx,
            };
            if let Some(effect) = chain.slots[slot].as_mut() {
                effect.set_parameter(param_id, value);
            }
        },
        AudioCommand::RemoveBusEffect { bus, slot } => {
            match bus {
                BusId::Bus1 => thread_state.bus1_fx.set_effect(slot, None),
                BusId::Bus2 => thread_state.bus2_fx.set_effect(slot, None),
                BusId::Master => thread_state.master_fx.set_effect(slot, None),
            }
        },
    }
}
```

---

## 5. UI Architecture

### 5.1 Frontend Effect Control Flow

```
┌─────────────────────────────────────────────┐
│  Frontend (TypeScript)                       │
│                                              │
│  ┌──────────┐    ┌────────────┐             │
│  │ Effect   │    │ Parameter  │             │
│  │ Selector │    │ Knobs      │             │
│  │ (type)   │    │ (CTRL 1-3) │             │
│  └────┬─────┘    └─────┬──────┘             │
│       │                │                     │
│       ▼                ▼                     │
│  invoke("set_bus_effect")  invoke("set_effect_param") │
│       │                │                     │
└───────┼────────────────┼─────────────────────┘
        │                │
        ▼                ▼
   Tauri Command    Tauri Command
        │                │
        ▼                ▼
   AudioState.send(AudioCommand::SetBusEffect)
   AudioState.send(AudioCommand::SetEffectParam)
```

### 5.2 New Tauri Commands

```rust
#[tauri::command]
fn set_bus_effect(bus: String, slot: usize, effect_type: String, state: State<'_, AudioState>) -> Result<(), String> {
    let bus_id = parse_bus_id(&bus)?;
    let fx_type = parse_effect_type(&effect_type)?;
    state.send_command(AudioCommand::SetBusEffect { bus: bus_id, slot, effect_type: fx_type })
}

#[tauri::command]
fn set_effect_param(bus: String, slot: usize, param_id: u8, value: f32, state: State<'_, AudioState>) -> Result<(), String> {
    let bus_id = parse_bus_id(&bus)?;
    state.send_command(AudioCommand::SetEffectParam { bus: bus_id, slot, param_id, value })
}

#[tauri::command]
fn get_effect_list() -> Vec<EffectInfo> {
    // Return all available effects with metadata for UI rendering
}
```

### 5.3 UI Layout Concept

The effect controls should integrate into the existing LCD screen area:

```
┌─────────────────────────────────────────┐
│ SP-404MK2 DAW                            │
├─────────────────────────────────────────┤
│ ┌─ LCD Screen ────────────────────────┐ │
│ │ FILTER+DRIVE          [BUS 1]       │ │
│ │                                      │ │
│ │  CUTOFF    RESO     DRIVE           │ │
│ │  ●─────   ●─────   ●─────          │ │
│ │  [0.50]   [0.30]   [0.75]          │ │
│ │                                      │ │
│ │ ◄ PREV    [SELECT]    NEXT ►        │ │
│ └──────────────────────────────────────┘ │
│                                          │
│ [RESAMPLE] [BUS 1] [BUS 2] [LOAD]      │
│                                          │
│ ┌──┐ ┌──┐ ┌──┐ ┌──┐                    │
│ │1 │ │2 │ │3 │ │4 │                    │
│ └──┘ └──┘ └──┘ └──┘                    │
│ ...                                      │
└─────────────────────────────────────────┘
```

Key UI elements to add:
1. **Effect type selector** — dropdown or prev/next navigation through the 37 effects
2. **Three rotary knobs** — mapping to CTRL 1-3 of the selected effect
3. **Bus target indicator** — which bus (1/2/Master) is being edited
4. **Effect on/off toggle** — bypass the effect chain

### 5.4 Knob Interaction Pattern

```typescript
// Rotary knob component that sends parameter changes
const createKnob = (busId: string, slot: number, paramId: number) => {
  const knob = document.createElement('input');
  knob.type = 'range';
  knob.min = '0';
  knob.max = '1';
  knob.step = '0.01';
  knob.addEventListener('input', async (e) => {
    const value = parseFloat((e.target as HTMLInputElement).value);
    await invoke('set_effect_param', { bus: busId, slot, paramId, value });
  });
  return knob;
};
```

---

## 6. Approach Comparison

### Approach A: Custom DSP from Scratch

Build all 37 effects with hand-written Rust DSP code, using only basic math and the existing `cpal`/`rtrb` stack.

#### Architecture

```
src-tauri/src/audio/
├── engine.rs          (modified — integrate FX chains)
├── state.rs           (modified — new AudioCommand variants)
├── mod.rs             (modified — declare new modules)
├── effects/
│   ├── mod.rs         (Effect trait + EffectChain + EffectType enum)
│   ├── chain.rs       (EffectChain implementation)
│   ├── factory.rs     (create_effect() factory function)
│   ├── filter.rs      (Filter+Drive, Isolator, Super Filter, EQ)
│   ├── delay.rs       (Sync Delay, Tape Echo, TimeCtrlDly, Ko-Da-Ma)
│   ├── reverb.rs      (Reverb, Zan-Zou)
│   ├── modulation.rs  (Chorus, JUNO Chorus, Flanger, Phaser, Wah)
│   ├── dynamics.rs    (Compressor, Tremolo/Pan)
│   ├── distortion.rs  (Overdrive, Distortion, WrmSaturator, Crusher)
│   ├── lofi.rs        (Lo-fi, 303 VinylSim, 404 VinylSim, Cassette Sim)
│   ├── performance.rs (Scatter, Slicer, DJFX Looper, Chromatic PS, Stopper)
│   ├── spatial.rs     (Resonator, SBF, Ring Mod, Downer)
│   └── special.rs     (Ha-Dou, To-Gu-Ro)
```

#### Pros
- **Zero external DSP dependencies** — complete control over algorithms
- **Minimal binary size** — no unused code pulled in from libraries
- **Maximum performance** — tuned for this exact use case
- **Authentic character** — can model SP-404MK2 behavior precisely
- **Learning value** — deep understanding of every algorithm

#### Cons
- **Massive implementation effort** — 37 effects × custom DSP = months of work
- **Quality risk** — reverb, chorus, and analog modeling are notoriously hard to get right
- **Maintenance burden** — all bugs and optimizations are on us
- **No established test baselines** — must build reference tests from scratch

#### Dependencies Added
- None (pure Rust math)

#### Estimated Effort
- Phase 1 (8 simple effects): ~3-4 weeks
- Full catalog (37 effects): ~4-6 months

---

### Approach B: FunDSP Integration (Recommended)

Use the `fundsp` crate as the DSP foundation. FunDSP provides high-quality, real-time-safe DSP primitives with an expressive Rust DSL. Build the `Effect` trait as a thin wrapper around FunDSP audio nodes.

#### Architecture

```
src-tauri/src/audio/
├── engine.rs          (modified — integrate FX chains)
├── state.rs           (modified — new AudioCommand variants)
├── mod.rs             (modified — declare new modules)
├── effects/
│   ├── mod.rs         (Effect trait + EffectChain + EffectType enum)
│   ├── chain.rs       (EffectChain implementation)
│   ├── factory.rs     (create_effect() — constructs FunDSP graphs)
│   ├── dsp_bridge.rs  (FunDspEffect wrapper: adapts fundsp::AudioNode to Effect trait)
│   ├── presets.rs     (SP-404MK2 effect preset definitions using FunDSP combinators)
│   └── custom/        (Hand-written effects for SP-404-specific behavior)
│       ├── vinyl.rs   (VinylSim, Cassette — need custom noise/crackle)
│       ├── scatter.rs (Scatter, Slicer — need beat-sync logic)
│       └── looper.rs  (DJFX Looper — needs buffer capture)
```

#### FunDSP Effect Example

```rust
use fundsp::prelude::*;

/// Create a Filter+Drive effect using FunDSP primitives
fn create_filter_drive(sample_rate: f32) -> impl AudioNode {
    // Resonant lowpass filter → soft saturation
    let cutoff = shared(1000.0);   // Hz, controlled by CTRL 1
    let resonance = shared(0.5);    // Q, controlled by CTRL 2
    let drive = shared(1.0);        // gain, controlled by CTRL 3

    (pass() | pass())  // stereo passthrough
        >> (lowpass_hz(cutoff.clone(), resonance.clone())
            | lowpass_hz(cutoff, resonance))
        >> (shape(Shape::Tanh(drive.clone()))
            | shape(Shape::Tanh(drive)))
}
```

#### Bridge Pattern

```rust
pub struct FunDspEffect {
    node: Box<dyn AudioNode<Inputs = U2, Outputs = U2>>,
    params: Vec<Shared>,  // FunDSP Shared variables for lock-free param control
    effect_type: EffectType,
}

impl Effect for FunDspEffect {
    fn process_frame(&mut self, left: &mut f32, right: &mut f32) {
        let input = Frame::from([*left, *right]);
        let output = self.node.tick(&input);
        *left = output[0];
        *right = output[1];
    }

    fn set_parameter(&mut self, param_id: u8, value: f32) {
        if let Some(param) = self.params.get(param_id as usize) {
            param.set(value);  // Lock-free atomic set
        }
    }
}
```

#### Pros
- **Battle-tested DSP primitives** — filters, delays, reverbs, waveshaping all included
- **Lock-free parameter control** — FunDSP's `Shared` type uses atomics internally
- **Real-time safe** — designed for audio callbacks, `no_std` capable
- **Rapid development** — can prototype all 37 effects quickly using combinators
- **High quality** — professional-grade filter and delay implementations
- **Expressive** — the DSL makes signal flow readable and maintainable

#### Cons
- **Dependency** — adds `fundsp` to the dependency tree (~moderate compile time)
- **Abstraction layer** — bridge between FunDSP's `AudioNode` and our `Effect` trait
- **Some effects need custom code** — VinylSim crackle, Scatter beat-sync, DJFX Looper are too specific for generic DSP primitives
- **Less control over exact character** — harder to match precise SP-404MK2 analog quirks

#### Dependencies Added
- `fundsp = "0.19"` (or latest)

#### Estimated Effort
- Phase 1 (8 simple effects): ~1-2 weeks
- Full catalog (37 effects): ~6-8 weeks

---

### Comparison Matrix

| Criteria | Approach A (Custom) | Approach B (FunDSP) |
|----------|-------------------|---------------------|
| **Time to first effect** | ~3 days | ~1 day |
| **Time to full catalog** | ~4-6 months | ~6-8 weeks |
| **Audio quality** | Depends on implementation skill | Professional-grade out of box |
| **SP-404 authenticity** | ★★★★★ (full control) | ★★★★ (good, some custom needed) |
| **Binary size impact** | Minimal | Moderate (~500KB) |
| **Compile time impact** | Minimal | Moderate |
| **Maintenance burden** | Very high | Low (DSP bugs upstream) |
| **Learning curve** | Steep (DSP knowledge required) | Moderate (learn FunDSP API) |
| **Test coverage ease** | Hard (need reference signals) | Easier (unit test combinators) |
| **Lock-free integration** | Must implement | Built-in (`Shared` type) |
| **Future extensibility** | Unlimited | Limited by FunDSP capabilities |

> **Recommendation**: **Approach B (FunDSP)** for the core effects, with custom implementations for SP-404-specific effects (VinylSim, Scatter, DJFX Looper). This hybrid approach gives us professional audio quality from day one while keeping the door open for hand-tuned character effects.

---

## 7. Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| **Audio glitches from FX processing** | High — audible artifacts | Profile each effect's CPU cost. Implement per-frame budget monitoring. Add dry fallback on overrun. |
| **Ring buffer overflow** | Medium — lost commands | Increase buffer capacity for param commands. Consider separate param channel (Option B from §3.3). Rate-limit UI param sends to ~60 Hz. |
| **FunDSP API instability** | Low — breaking changes | Pin version in Cargo.toml. Wrap behind our own `Effect` trait for isolation. |
| **Memory allocation in audio thread** | Critical — realtime violation | Pre-allocate ALL effect state at construction. Audit with `assert_no_alloc` crate in debug builds. |
| **Frontend knob latency** | Medium — sluggish feel | Keep Tauri invoke overhead minimal. Consider WebSocket for param streaming if IPC is too slow. |
| **Sample rate changes** | Low — rare at runtime | Propagate `set_sample_rate()` to all effects. Handle gracefully (recalculate coefficients). |

---

## 8. Open Questions

1. **Should Bus routing be per-pad or per-trigger?** — Currently per-trigger (`TriggerPad.routing`), but the `set_pad_bus` command suggests it should be a persistent pad property. Need to decide.

2. **Effect persistence** — Should effect chain configurations persist across app restarts? If so, need serialization (JSON/MessagePack) to disk.

3. **Wet/Dry mix** — Each effect should have a global wet/dry parameter, or should this be per-effect? The real SP-404MK2 varies by effect type.

4. **Master bus effects** — How many slots on the master bus? The real hardware has a separate Master Compressor and Master EQ. Should these be hard-coded or configurable?

5. **Beat sync** — Effects like Scatter, Slicer, and Sync Delay need tempo information. Where does BPM come from? Manual input? Tap tempo? Pattern-derived?

6. **Resampling capture point** — Should resampling capture the signal BEFORE or AFTER master FX? The real hardware captures after all FX processing (current code does this at the output stage, which is correct).

---

## 9. File Change Inventory

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/src/audio/mod.rs` | Modify | Add `pub mod effects;` |
| `src-tauri/src/audio/state.rs` | Modify | Add new `AudioCommand` variants, `BusId` enum, `EffectType` enum |
| `src-tauri/src/audio/engine.rs` | Modify | Add `EffectChain` fields to thread state, integrate FX processing at lines 138-145, handle new commands |
| `src-tauri/src/audio/effects/mod.rs` | Create | `Effect` trait, `EffectChain`, `EffectType` enum, re-exports |
| `src-tauri/src/audio/effects/chain.rs` | Create | `EffectChain` implementation |
| `src-tauri/src/audio/effects/factory.rs` | Create | `create_effect()` factory function |
| `src-tauri/src/audio/effects/dsp_bridge.rs` | Create | FunDSP → Effect trait adapter |
| `src-tauri/src/audio/effects/presets.rs` | Create | All 37 effect definitions using FunDSP |
| `src-tauri/src/audio/effects/custom/` | Create | Custom effects (vinyl, scatter, looper) |
| `src-tauri/src/lib.rs` | Modify | Add Tauri commands: `set_bus_effect`, `set_effect_param`, `get_effect_list` |
| `src-tauri/Cargo.toml` | Modify | Add `fundsp` dependency |
| `src/main.ts` | Modify | Add effect selection UI, knob controls, bus-to-effect mapping logic |
| `index.html` | Modify | Add effect control HTML elements in LCD area |
| `src/styles.css` | Modify | Add styles for knobs, effect selector, parameter display |
