# Specification: auto-save

## Overview
Background persistence engine to proactively prevent data loss without interrupting user workflow.

## Requirements
- **Background Cycle**: The system MUST periodically save the current project state in the background.
- **Explicit Save**: Standard explicit save operations MUST still be available and function synchronously or with clear user feedback.
- **Non-blocking**: Auto-save operations MUST NOT interrupt audio playback or cause UI stutter.
