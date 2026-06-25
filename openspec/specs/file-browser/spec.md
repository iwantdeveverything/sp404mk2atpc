# Specification: file-browser

## Overview
Full disk exploration and previewing interface that allows users to find and audition audio samples across their entire system before importing them into a project.

## Requirements
- **Full Disk Access**: The browser must be able to navigate the entire local file system, not limited to a sandbox.
- **Pre-listen**: Selecting an audio file should optionally trigger raw audio playback for quick auditioning.
- **Visual Waveforms**: The browser UI MUST display a visual waveform for the currently selected/pre-listened audio file.
- **Active States**: The UI MUST have clear "active" states indicating which file is currently being previewed.
- **Integration**: Must allow dragging files from the browser directly into the project (pads).
