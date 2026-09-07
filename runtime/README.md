# PoRE Runtime

This crate is the host-neutral V1 runtime entry point.

It deliberately has no dependency on Nextcloud or Nextcloud Talk. Host adapters hand finalized browser artifacts to the runtime through a small framed stdin/stdout contract.

## V1 operation

`recording.submit_finalized_artifact`

The runtime accepts host-neutral artifact metadata followed by the finalized payload, then delegates to the existing `application::browser_recording_artifact` boundary and the existing recorder persistence path.

The runtime does **not** define a second recording lifecycle, persistence model, or synchronization path.

## Transport framing

Each request is:

1. 4-byte big-endian JSON header length
2. UTF-8 JSON request header
3. raw payload bytes of the declared length

Each response is a 4-byte big-endian JSON response length followed by the UTF-8 JSON response.

The framing is intentionally independent of HTTP, Nextcloud, Talk, or any specific IPC mechanism. A normal Nextcloud app may later invoke the binary through a local process boundary without changing the PoRE contract.
