# Proposal: SP-404MK2 Library Management

## Goal
Implement a robust library management system that allows users to browse their disk for audio samples, preview them efficiently, and manage projects via isolated, self-contained project bundles with background auto-save capabilities.

## Product Decisions

1. **Project Bundles**
   - Sample files will be copied to a project-specific directory when added to a project.
   - Project files will use relative references to these copied samples to ensure the bundle is fully portable.

2. **File Browser Scope**
   - The integrated file browser will allow browsing the entire disk rather than being sandboxed to a specific folder, giving users full access to their sample libraries anywhere on the system.

3. **Pre-listen Mechanism**
   - Pre-listening to samples in the browser will play back the raw audio.
   - This playback will explicitly bypass the effects engine (`effects-engine`) and BPM synchronization (`bpm-sync`) for quick and uncolored auditioning.

4. **Auto-save**
   - The application will provide standard explicit save operations.
   - It will also implement background auto-save cycles to proactively prevent data loss.

## Capabilities

### Affected Existing Capabilities
- **`audio-core`**: Will need an independent raw audio playback mechanism/channel for pre-listen that bypasses the FX routing and bus processing.
- **`bpm-sync`**: Pre-listen needs to bypass this engine entirely.
- **`effects-engine`**: Must not process the pre-listen audio.
- **`ui-routing` / `ui-juice`**: New views and interface elements required for the full-disk File Browser and project saving.
- **`drag-drop`**: Dragging and dropping files into the project must trigger the Project Bundle copy behavior, ensuring the sample is ingested rather than just referenced.

### New Capabilities Introduced
- **`library-management`**: Core management of project sample assets.
- **`file-browser`**: Full disk exploration and previewing.
- **`auto-save`**: Background persistence engine.
