# Proposal: Drag and Drop

## Intent
Enable drag-and-drop file loading for audio samples directly onto the pads in the SP-404MK2 DAW.

## Scope
- **In Scope:** Native Tauri Drag/Drop, sequential multi-file loading (e.g., if dropped on Pad 1, they fill Pad 1, 2, 3...), visual hover feedback (highlighting or glowing on the hovered pad), LCD error messages for invalid formats (e.g., PDFs).
- **Out of Scope:** Non-audio drag/drop.

## Capabilities
- `drag-drop-interface`: New capability for handling drag-and-drop events via Tauri.
- `lcd-notifications`: Modified/new capability to display transient error messages on the LCD.
