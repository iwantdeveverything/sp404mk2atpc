# Exploration: SP404 UI Juice Enhancement

## Goal
Enhance the "alive" feeling of pad interactions, transitions, and LCD boot sequences to make the interface elegant and set a foundation for future features, while maintaining low latency and the current Vanilla TS architecture.

## 1. Pad Interactions & UI Juice
**Current State**: Basic CSS `transform: translateY(4px) scale(0.98)`, `box-shadow`, and hard-coded `setTimeout` cascades in `main.ts`.

**Options**:
*   **Approach A: CSS-Only Enhancements (Keyframes + Transitions + CSS Variables)**
    *   *Pros*: Zero JS execution overhead, providing the lowest visual latency possible for rapid audio triggering. Complex effects (glows, elastic bounces) run on the browser's compositor thread.
    *   *Cons*: Sequential animations require managing multiple CSS classes.
*   **Approach B: View Transitions API**
    *   *Pros*: Great for layout shifts.
    *   *Cons*: Not suited for high-frequency hit states; can cause rendering blocks.
*   **Approach C: Lightweight JS Animation (Anime.js / Motion)**
    *   *Pros*: Granular control over physics-based interactions.
    *   *Cons*: Adds bundle size and relies on `requestAnimationFrame`, introducing a slight latency risk compared to pure CSS compositing.

**Recommendation**: **Approach A (CSS Keyframes + Advanced Transitions)**. For an instrument, latency is critical. We can upgrade the CSS using advanced `cubic-bezier` timing functions for elastic pad hits, intense CSS radial gradients for glows, and only use JS to toggle state classes (`.is-playing`, `.is-target`).

## 2. LCD Boot Sequence & State Transitions
**Current State**: Static text rendering. Direct `innerText` manipulation.

**Options**:
*   **Approach A: Vanilla JS Async Sequences**
    *   *Pros*: Simple to read using `async`/`await` and a `delay` helper. Easy to sync with actual Tauri backend readiness.
    *   *Cons*: Can get messy if interwoven with audio logic.
*   **Approach B: CSS Keyframe "Glitch" and "Blink" effects**
    *   *Pros*: Declarative and visually striking (e.g., CRT scanlines, flickering text).
    *   *Cons*: Hard to synchronize text content changes purely in CSS.

**Recommendation**: **Hybrid (JS Async Sequence + CSS CRT Effects)**. We should create a dedicated `runBootSequence()` async function in `main.ts` that handles the text orchestration (e.g., version number -> loading bar `[====  ]` -> "WELCOME"). We will complement this with CSS-based CRT scanlines, a subtle text flicker/glow, and a blinking cursor for the `.lcd-screen`.

## 3. Architectural Foundation
To prevent `main.ts` from becoming unwieldy with visual logic:
*   Extract visual state management into lightweight Vanilla TS objects/modules (e.g., `LCDController`, `PadController`).
*   This decouples the visual "juice" from the `invoke` calls to Tauri, allowing us to easily add more complex themes or states later without risking audio logic.
