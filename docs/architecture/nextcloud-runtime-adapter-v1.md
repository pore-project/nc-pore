# Nextcloud Runtime Adapter V1

## Status

Implemented as the first thin Nextcloud host adapter around the host-neutral PoRE Runtime boundary.

## Boundary

```text
Browser completion
    -> Nextcloud OCS endpoint
    -> RecordingTransportController
    -> RuntimeClient
    -> pore-runtime process
    -> PoRE Application / Recorder / Persistence
```

The Nextcloud layer owns only the transport and host concerns. It does not create a second recording lifecycle or persistence implementation.

## Browser request

The browser sends a `multipart/form-data` POST to the normal authenticated Nextcloud OCS endpoint:

`/ocs/v2.php/apps/pore/v1/recordings/finalized-artifact`

The request contains:

- `metadata`: JSON metadata with authoritative and technical identities plus audio format facts
- `payload`: the finalized WAV blob

The browser sets `OCS-APIRequest: true` and uses the existing Nextcloud session. No second authentication mechanism is introduced.

## Nextcloud adapter

`RecordingTransportController` validates the request and obtains the uploaded temporary file through `IRequest::getUploadedFile()`.

`RuntimeClient` starts the configured `pore-runtime` executable without a shell, writes the length-prefixed JSON header followed by the payload bytes to stdin, and decodes the length-prefixed JSON response from stdout.

The payload is copied in bounded chunks from the PHP upload temporary file; it is not base64-encoded.

## Configuration

The current development adapter expects two app configuration values:

- `runtime_binary`: absolute path to the executable `pore-runtime`
- `runtime_persistence_root`: local filesystem path used by the runtime persistence provider

These values are intentionally explicit. V1 packaging and installation automation must later provide the correct runtime binary for the host platform rather than silently guessing a platform-specific executable.

## Completion state

After a successful `stored` response, the browser completion job records the transport as `completed` in its durable IndexedDB manifest.

If the browser or page disappears before completion, the existing recovery scan can prepare the finalized capture again. The transport adapter is therefore downstream of the durable local preservation boundary.

## Explicit non-goals

- No AppAPI dependency
- No ExApp
- No standalone PoRE HTTP server
- No Nextcloud-specific logic inside the Rust runtime
- No direct WebDAV upload from the browser
- No second persistence path
- No identity aliasing
- No production packaging decision for platform-specific runtime binaries

## Next technical step

Complete runtime binary packaging/installation for supported Nextcloud environments and then extend the runtime/application handoff from `stored` to synchronization enqueue and authoritative recording completion.
