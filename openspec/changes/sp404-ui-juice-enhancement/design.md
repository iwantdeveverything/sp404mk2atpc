# Technical Design: sp404-ui-juice-enhancement

## 1. CSS Styling: Skeuomorphic Pad Press
**Goal:** Enhance the tactile feel of pad presses with deeper sinking and a retro-modern glow, without changing the layout.
**Changes to `src/styles.css`:**
- Update the `.pad:active, .pad.active` selectors:
  - Increase the sinking effect: `transform: translateY(6px) scale(0.98);`.
  - Deepen the inset shadows to simulate a recessed physical pad: `inset 0 4px 10px rgba(0, 0, 0, 0.6), inset 0 -2px 4px rgba(255, 255, 255, 0.1)`.
  - Intensify the neon glow: `0 0 25px var(--pad-active-glow), 0 0 10px var(--pad-active)`.
- Adjust transition timing on `.pad` to ensure a punchy down-stroke and a smooth up-stroke.

## 2. TypeScript Logic: LCD Typing Text Effect
**Goal:** Create a fast, retro-computer typing effect for LCD messages, ensuring zero latency for audio playback.
**Changes to `src/main.ts`:**
- Implement a new `typeText` utility function to handle the animation:
  ```typescript
  let typingTimeout: number | undefined;
  
  const typeText = async (element: HTMLElement, text: string, speedMs = 20): Promise<void> => {
    return new Promise(resolve => {
      if (typingTimeout) clearTimeout(typingTimeout);
      element.innerText = "";
      let i = 0;
      
      const typeNext = () => {
        if (i < text.length) {
          element.innerText += text.charAt(i);
          i++;
          typingTimeout = window.setTimeout(typeNext, speedMs);
        } else {
          resolve();
        }
      };
      typeNext();
    });
  };
  ```
- **Performance consideration:** Audio triggering (`invoke("trigger_pad")`) must remain synchronous and immediate. Text updates using `typeText` will happen asynchronously so they do not block the audio thread. For pad hits, we will use a very fast speed (e.g., `10ms`) or fallback to instant updates if needed, while boot and loading statuses can use a slightly slower speed (e.g., `40ms`) for the retro feel.

## 3. App Boot Sequence
**Goal:** Simulate a hardware sampler module loading up before the UI is ready to use.
**Changes to `src/main.ts`:**
- Introduce an `isBooting` flag to prevent interaction during the startup sequence.
- On `DOMContentLoaded`, run an async `runBootSequence` function:
  1. Set `isBooting = true`.
  2. Clear the LCD (`statusDisplay`).
  3. Delay 300ms.
  4. Execute `await typeText(statusDisplay, "INIT AUDIO...", 40)`.
  5. Delay 600ms (simulating module loading).
  6. Execute `await typeText(statusDisplay, "READY", 40)`.
  7. Set `isBooting = false`.
- Update all interaction handlers (`triggerPad`, right-click target selection, upload button, drag-and-drop) to exit early (`if (isBooting) return;`) if the sequence is still active.
