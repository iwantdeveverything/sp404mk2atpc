# Specification: library-management

## Overview
Core management of project sample assets. Enables projects to be self-contained by automatically copying imported audio files into a project-specific directory, keeping references relative.

## Requirements
- **Project Bundles**: A project must store its audio assets locally within its own directory structure.
- **Copy on Import**: When an audio file is added to a project, it MUST be copied into the project bundle, not just referenced from its original location on disk.
- **Portable References**: All file references within the project state must be relative to the bundle root to allow moving the project folder without breaking links.
