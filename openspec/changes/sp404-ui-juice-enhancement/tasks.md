# Tasks: sp404-ui-juice-enhancement

Decision needed before apply: No
Chained PRs recommended: No
400-line budget risk: Low

## PR 1: UI Juice Enhancements (CSS & Text Logic)
- [x] **CSS Styling:** Update `.pad:active, .pad.active` in `src/styles.css` to increase the sinking effect (`translateY(6px) scale(0.98)`), deepen inset shadows, and intensify neon glow.
- [x] **CSS Styling:** Adjust transition timing on `.pad` in `src/styles.css` for punchy down-stroke and smooth up-stroke.
- [x] **TypeScript Logic:** Implement the asynchronous `typeText` utility function in `src/main.ts` for the fast retro-computer typing effect on the LCD.
- [x] **TypeScript Logic:** Integrate `typeText` for pad hits and LCD messages ensuring it doesn't block the synchronous audio thread in `src/main.ts`.
- [x] **App Boot Sequence:** Implement an `isBooting` flag and an async `runBootSequence` function in `src/main.ts` on `DOMContentLoaded` (showing "INIT AUDIO..." then "READY").
- [x] **App Boot Sequence Guard:** Update interaction handlers in `src/main.ts` (e.g., `triggerPad`, target selection, upload, drag-and-drop) to exit early if `isBooting` is true.
