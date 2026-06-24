# Design: Drag and Drop Feedback Enhancement

## Architecture & Decisions

### 1. Visual Feedback (Cascade Effect)
**Decision**: Use a transient CSS class `.cascade-glow` applied sequentially via `setTimeout` in the drag-and-drop handler to create a visual cascade effect across the pads where files were dropped.
- **Why**: CSS animations ensure smooth performance (hardware acceleration) while `setTimeout` provides precise control over the delay between each pad lighting up, matching standard SP-404 behavior.
- **Mechanism**:
  - CSS: Define `.cascade-glow` with a bright glowing box-shadow and background transition.
  - JS/TS: When a drop event concludes, trigger a function `playCascadeAnimation(padIndices)` that iterates through the affected pads, adding `.cascade-glow`, and removing it after a brief duration (e.g., 300ms) to create a ripple/cascade effect.

### 2. LCD Text Formatting (Typescript Logic)
**Decision**: Implement a utility function `formatDropMessage(count: number, startPad: number, endPad: number): string` to handle the LCD display formatting.
- **Why**: Needs to support pluralization and ranges dynamically based on the number of files dropped.
- **Mechanism**:
  - If `count === 1`: Return `"Loaded 1 in Pad {startPad}"`
  - If `count > 1`: Return `"Loaded {count} in Pads {startPad}-{endPad}"`
  - This logic will be encapsulated in a pure formatting function within the UI or Audio state logic, to ensure it is easily testable and decoupled from the DOM event handlers.
