# Proposal: SP-404MK2 Effects Engine Robustness

## Intent

The effects engine ships a 38-variant `EffectType` catalog but `create_effect()`
only implements 8 effects (Filter, Isolator, Delay, Reverb, VinylSim, DjfxLooper,
Scatter, Slicer). The remaining 30 variants fall through a `_ => pass() | pass()`
arm — a **silent passthrough** that pretends an effect exists while doing nothing.
On top of that, three structural defects undermine the effects that DO exist:

1. **Spec violation — no wet/dry mix.** `openspec/specs/effects-engine/spec.md`
   REQUIRES "Wet/Dry Mix Control" on every effect (`spec.md:29-34`). No mix layer
   exists in code today. The shipped spec is currently violated.
2. **Parameter range bug.** The frontend knob emits `0.0–1.0` (`src/main.ts` knob
   handler), but effects bind real-unit `Shared` values (e.g. Filter cutoff
   `shared(1000.0)` in Hz). A `0–1` knob drives cutoff to `0–1 Hz` — useless.
   There is no parameter metadata (name/range/curve) and no normalization layer.
3. **Phantom effects.** `set_bus_effect` in `lib.rs` maps only 8 strings to
   `EffectType`, so the UI cannot even reach the other 30, yet the catalog implies
   they work.

This change adds **true robustness and power** to the effects algorithms: a
prioritized batch of 12 high-quality effects, a cross-cutting wet/dry mix layer
that satisfies the existing spec, a parameter metadata + normalization system that
fixes the range bug, and the removal of silent passthrough in favor of explicit
unimplemented handling.

## Product Decisions

These four decisions are approved and fixed; specs/design build on them directly.

### Decision 1 — Prioritized batch of 12 effects (not all 37)

Implement a curated, high-quality batch this cycle:
**Chorus, Flanger, Phaser, Tremolo, AutoPan, Compressor, Equalizer, Distortion,
Overdrive, Bitcrusher, LoFi, Wah.** The remaining unimplemented effects are
deferred to future cycles.

**Rationale:** Quality over breadth. Twelve well-tuned, parameter-correct,
mix-enabled effects deliver more musical value than 30 shallow passthroughs.
The batch covers the three families players reach for first: modulation, dynamics
and tone shaping, and drive/lo-fi character.

### Decision 2 — Wet/dry mix as a cross-cutting effect-slot layer

Add wet/dry mix in the `EffectChain` slot wrapper, NOT inside each effect node, so
ALL effects — the existing 8 and the new 12 — inherit it uniformly.

**Rationale:** A single, audited blend (`out = dry*(1-mix) + wet*mix`) at the slot
boundary satisfies the existing-but-violated spec once, keeps individual DSP nodes
focused on the wet signal only, and guarantees consistent behavior across the whole
catalog. The blend runs per-frame on the audio thread and MUST be allocation-free.

### Decision 3 — Per-effect parameter metadata + normalization layer

Add parameter descriptors (name, min/max range, scaling curve — linear/exponential)
per effect, and a normalization layer that maps the knob's `0–1` onto the real
parameter range in the backend before the value reaches the `Shared`.

**Rationale:** Fixes the range bug at its root and turns the UI from a blind `0–1`
sender into a parameter-aware control surface (labels, ranges, sensible curves —
e.g. exponential for cutoff/time, linear for mix/depth). Normalization lives in the
backend so the contract is single-sourced and the UI stays declarative.

### Decision 4 — Explicit unimplemented handling (no silent passthrough)

`create_effect` returns explicit `None` for not-yet-implemented variants. The
frontend selector lists ONLY implemented effects — no phantom entries that do
nothing. The `EffectType` enum keeps every variant as a declared catalog/roadmap.

**Rationale:** Honesty over the illusion of completeness. `None` makes the
unimplemented set queryable, lets the UI filter the selector from a single source of
truth, and turns "the effect does nothing" from a silent surprise into an explicit,
testable state. The enum remains the roadmap of what is coming.

## Scope

### In Scope
- Wet/dry mix layer in the `EffectChain` slot, applied uniformly to all effects
  (existing 8 + new 12), allocation-free on the audio thread.
- Parameter metadata system (name, min/max, curve) per effect + a backend
  normalization layer mapping `0–1` knob input to real parameter ranges.
- 12 new effect implementations via the existing `FunDspWrapper` / custom-Rust
  pattern: Chorus, Flanger, Phaser, Tremolo, AutoPan, Compressor, Equalizer,
  Distortion, Overdrive, Bitcrusher, LoFi, Wah.
- Replace the `_ => pass() | pass()` arm with explicit `None` for unimplemented
  variants.
- IPC/command plumbing: extend `set_bus_effect` string→`EffectType` mapping to the
  new batch; extend the lock-free `AudioCommand` path so normalized params and the
  per-slot mix reach the audio thread without allocation.
- Frontend: parameter-aware knob UI (driven by metadata) and a selector filtered to
  implemented effects only.

### Out of Scope
- The ~18 remaining unimplemented effects not in this batch (RingMod, PitchShifter,
  Fuzz, Octave, Resonator, TapeEcho, Shimmer, Gater, Reverse, Stutter, TapeStop,
  Compressor2, Equalizer2, Chorus2, Flanger2, Phaser2, Delay2, and any others) —
  deferred to future cycles.
- New audio I/O (no input FX, no new devices/channels).
- Changes to bus routing topology (Bus1/Bus2/Master structure stays as-is).
- New persistence format work beyond what mix + params require.

## Capabilities

### Modified Capabilities
- `effects-engine` (primary): wet/dry mix requirement satisfied at the slot layer;
  parameter-metadata + normalization requirements added; 12 new effect
  implementations; explicit `None` for unimplemented variants; effect-list query
  exposes only implemented effects.
- `audio-core`: `AudioCommand` / lock-free path extended to carry normalized
  parameter values and per-slot wet/dry mix without audio-thread allocation.
- `ui-routing`: selector filtered to implemented effects; `set_bus_effect` mapping
  extended to the new batch.
- `ui-juice`: knob UI becomes parameter-aware (labels, ranges, curves from
  metadata) instead of a raw `0–1` sender.

### New Capabilities
- None. This change hardens and extends the existing `effects-engine` rather than
  introducing a new capability surface.

## Approach

Continue the established FunDSP-hybrid pattern. New effects are built as `fundsp`
graphs wrapped in `FunDspWrapper`, or custom Rust where fundsp is awkward, exposing
their tunable `Shared` values as ordered params. Two cross-cutting layers are added
around the effects:

1. **Mix layer** — a thin wrapper at the `EffectChain` slot that captures the dry
   frame, runs the effect to produce the wet frame, and blends per-frame. Pure
   arithmetic, no allocation.
2. **Parameter layer** — each effect declares descriptors `(name, min, max, curve)`.
   A backend normalization function maps incoming `0–1` to the real range using the
   curve, then writes the `Shared`. The UI reads descriptors to render labeled,
   correctly-ranged knobs.

`create_effect` returns `None` for unimplemented variants; an effect-list accessor
filters the catalog to implemented entries for the selector. Realtime discipline is
preserved: `create_effect` and effect swap run off the audio thread; `process_frame`,
the mix blend, and `set_parameter`/normalization-on-write stay allocation-free under
`assert_no_alloc`.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src-tauri/src/audio/effects/mod.rs` | Modified | 12 new effects; remove `_ => pass()` silent passthrough (return `None`); parameter descriptors; mix layer at slot |
| `src-tauri/src/audio/engine.rs` | Modified | Apply per-slot wet/dry blend in chain processing |
| `src-tauri/src/audio/state.rs` | Modified | `AudioCommand` variants for normalized param + mix; command handling |
| `src-tauri/src/lib.rs` | Modified | Extend `set_bus_effect` mapping to new batch; effect-list/param-metadata IPC; `set_effect_param` normalization path |
| `src/main.ts` | Modified | Parameter-aware knobs from metadata; selector filtered to implemented effects |
| `src/styles.css` | Modified | Parameter label/range UI as needed |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Audio-thread allocation in new DSP nodes or mix/normalization path | High | Pre-allocate all state at `create_effect`; keep blend + param-write pure arithmetic; gate every new effect with `assert_no_alloc` tests |
| fundsp expressivity limits for some batch effects (see below) | Medium | Build higher-risk effects as custom Rust where the fundsp graph is awkward; accept simplified-but-musical first versions, refine later |
| Per-effect quality vs the 400-line PR budget | High | Split into chained PRs by family; foundational mix+param-metadata PR lands first |
| Parameter-curve mismatch (wrong range/curve feels wrong) | Medium | Descriptors centralize ranges; tune per effect; expose curve as data so adjustments don't touch DSP |
| Selector-filtering regressions hide working effects | Low | Effect-list accessor is the single source of truth; covered by a list/contract test |

### fundsp risk per batch effect
- **Chorus** — Low. Modulated short delay; straightforward fundsp graph.
- **Flanger** — Low/Medium. Modulated very-short delay with feedback; feedback paths need care but are expressible.
- **Phaser** — Medium. Cascaded allpass with LFO; doable but graph gets verbose.
- **Tremolo** — Low. LFO amplitude modulation; trivial.
- **AutoPan** — Low. LFO panning; trivial.
- **Compressor** — High. Dynamics with envelope detection / sidechain are the weakest fit in fundsp; likely custom Rust gain computer + envelope follower.
- **Equalizer** — Low/Medium. Cascaded bell/shelf filters; pattern already proven by Isolator.
- **Distortion** — Low. Waveshaping/clip; trivial.
- **Overdrive** — Low. Soft-clip waveshaping; trivial.
- **Bitcrusher** — Medium. Sample-rate decimation + bit reduction; may need a small custom sample-and-hold node.
- **LoFi** — Medium. Combination of bandlimit + bitcrush + character; composite, mostly reuse.
- **Wah** — Medium. Resonant bandpass swept by an LFO or envelope; envelope-follower variant trends toward Compressor-level risk.

Highest fundsp risk: **Compressor** (dynamics/sidechain), then **Wah** (if
envelope-driven) and **Bitcrusher** (decimation). PitchShifter — explicitly NOT in
this batch — would be even higher risk and is correctly deferred.

## Chained PR Breakdown (recommended)

Per-effect quality plus mix + parameter infra will exceed a single 400-line PR.
Recommended chain, foundational infra first:

1. **PR 1 — Foundation: wet/dry mix + parameter metadata/normalization + explicit
   `None`.** Slot-level mix layer, parameter descriptors, backend normalization,
   removal of silent passthrough, selector filtering, IPC plumbing. Lands the
   cross-cutting layers the existing 8 effects also benefit from. Satisfies the
   violated wet/dry spec requirement.
2. **PR 2 — Modulation family.** Chorus, Flanger, Phaser, Tremolo, AutoPan.
3. **PR 3 — Dynamics + tone family.** Compressor, Equalizer, Wah. (Highest-risk
   group; isolated so Compressor's custom DSP can be reviewed on its own.)
4. **PR 4 — Drive + lo-fi family.** Distortion, Overdrive, Bitcrusher, LoFi.

Chain strategy (stacked-to-main vs feature-branch-chain) to be confirmed by the user
at tasks/apply time.

## Rollback Plan

Each chained PR is independently revertible. Reverting an effect-family PR removes
those effects; the foundation PR (mix + params) is additive and can stand alone.
Reverting the foundation PR restores the prior `_ => pass()` behavior. No data
migration is required; persisted effect configs only gain optional mix/param fields.

## Success Criteria

- [ ] All 12 batch effects process audio without glitches at 44.1kHz/48kHz.
- [ ] Wet/dry mix blends dry and wet for ALL effects (existing 8 + new 12),
      satisfying `effects-engine` spec `Wet/Dry Mix Control`.
- [ ] Knob `0–1` input maps to correct real parameter ranges via metadata-driven
      normalization (Filter cutoff sweeps musically, not `0–1 Hz`).
- [ ] `create_effect` returns `None` for unimplemented variants; no silent
      passthrough remains.
- [ ] Frontend selector shows only implemented effects.
- [ ] Zero allocations in `process_frame`, mix blend, and param-write paths
      (verified with `assert_no_alloc`).
- [ ] `set_bus_effect` reaches every effect in the new batch.
