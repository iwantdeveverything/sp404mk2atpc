# Tasks: Drag and Drop Feedback Enhancement

## PR 1: Drag and Drop Feedback Enhancement
- [x] Add `.cascade-glow` class to CSS with a glowing box-shadow and transition.
- [x] Create `formatDropMessage(count: number, startPad: number, endPad: number)` utility function to format LCD text.
- [x] Update Drag and Drop handler to calculate pad indices affected and set LCD text using `formatDropMessage`.
- [x] Implement `playCascadeAnimation(padIndices: number[])` to sequentially apply and remove `.cascade-glow` (e.g., 300ms duration).
- [x] Wire up the formatting and cascade animation to trigger after a successful drop event, ensuring LCD message reverts after 2 seconds.
