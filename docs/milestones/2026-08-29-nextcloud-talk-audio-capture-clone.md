# Milestone: Nextcloud Talk Audio Capture Clone Reality Check

## Status

Validated — primary ADR-072 capture/clone mechanism proven in live integration.

## Scope

Validate the ADR-072 browser-side capture boundary against a real Nextcloud Talk installation without relying on mocks or a second independent microphone capture.

## Test environment

- Nextcloud 34.0.3 running in Docker;
- Nextcloud Talk enabled;
- Firefox 154.0;
- HTTPS through Caddy using a local `mkcert` certificate;
- LAN access through `A-desktop-G.local` / `192.168.192.126`;
- real audio input: RØDECaster Pro Analog Stereo;
- NC-PoRe app 0.1.0 with the early `getUserMedia()` hook;
- Docker resource limits: 4 CPU / 4 GiB for Nextcloud, 2 CPU / 2 GiB for MariaDB, 0.5 CPU / 512 MiB for Caddy.

## Validation

Talk's real `getUserMedia()` calls were intercepted by the NC-PoRe browser hook. Audio capture produced independent cloned `MediaStreamTrack` instances via `MediaStreamTrack.clone()`, while the original stream continued to be returned to Talk.

The cloned track was independently connected to a `MediaStream` and `MediaRecorder`. A real 52.54-second recording was produced as `pore-talk-clone-test.ogx`.

The resulting artifact was verified with `ffprobe` as:

- Ogg container;
- Opus audio;
- stereo;
- 48 kHz input sample rate;
- approximately 130 kbit/s;
- encoder: Mozilla Firefox 154.0.

## Result

The primary ADR-072 capture/clone mechanism is technically validated under real Nextcloud Talk conditions.

This milestone does not validate the production recording lifecycle, chunk persistence, Opening Sync Signet integration, artifact persistence, or transport synchronization. Those remain subsequent implementation steps.

## Cleanup

The temporary debug instrumentation and `/tmp/pore-adr072` test copy were removed. The local `develop` checkout remained unchanged and clean. Talk test rooms were removed after validation.
