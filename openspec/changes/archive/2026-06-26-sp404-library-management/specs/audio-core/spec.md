# audio-core (delta)

## Changes
- **Pre-listen Channel**: Add an independent raw audio playback mechanism/channel specifically for the file browser pre-listen feature.
- **Routing Bypass**: This pre-listen channel MUST explicitly bypass the FX routing and bus processing to ensure uncolored auditioning.
