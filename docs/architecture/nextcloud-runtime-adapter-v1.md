# Nextcloud Runtime Adapter V1

## Status

Implemented as the thin normal Nextcloud app boundary for V1 recording artifact delivery.

## V1 boundary

```text
Browser completion
    -> authenticated Nextcloud OCS endpoint
    -> RecordingTransportController
    -> Nextcloud Files API
    -> authoritative stored artifact
```

The browser already owns a fully finalized and durably persisted transfer artifact. V1 therefore does not introduce a second PoRE synchronization queue, sync worker, or remote-transfer lifecycle. Nextcloud is the host and owns the authoritative file storage.

The host-neutral PoRE Runtime remains a separate boundary for PoRE-owned processing and future host variants; it is not required to duplicate the authoritative Nextcloud file in V1.

## Browser request

The browser sends a `multipart/form-data` POST to the normal authenticated Nextcloud OCS endpoint:

`/ocs/v2.php/apps/pore/v1/recordings/finalized-artifact`

The request contains:

- `metadata`: JSON metadata with authoritative and technical identities plus audio format facts
- `payload`: the finalized WAV blob

The browser sets `OCS-APIRequest: true` and uses the existing Nextcloud session. No second authentication mechanism is introduced.

## Nextcloud storage handoff

`RecordingTransportController` validates the request and obtains the uploaded temporary file through `IRequest::getUploadedFile()`.

`NextcloudArtifactStorage` then uses Nextcloud's public `IRootFolder` / `IUserSession` filesystem APIs. It creates the destination path inside the authenticated user's Nextcloud Files tree and streams the temporary upload into the destination in bounded chunks. It does not access Nextcloud's underlying data directory directly.

V1 target layout is:

```text
PoRE/
└── Productions/
    └── <production_id>/
        └── Recordings/
            └── <recording_id>/
                └── <capture_id>.wav
```

## Authoritative completion

A successful HTTP/OCS response alone is not considered sufficient for PoRE completion.

After writing, the adapter verifies both:

1. the stored Nextcloud file size equals the finalized payload size;
2. SHA-256 of the file read back through the Nextcloud Files API equals SHA-256 of the finalized transfer payload.

Only after both checks succeed does the controller return `status: stored` together with the Nextcloud file id, path, size and SHA-256. The browser then marks the durable completion job as completed.

This gives PoRE the concrete host-side acknowledgement it needs: the artifact exists in Nextcloud Files and is bit-for-bit identical to the finalized transfer artifact.

## Explicit non-goals

- No AppAPI dependency
- No ExApp
- No standalone PoRE HTTP server for V1
- No direct browser WebDAV dependency
- No PoRE synchronization work queue for V1
- No V1 sync worker
- No second authoritative persistence path
- No identity aliasing
- No direct access to Nextcloud's private data directory

## Architectural consequence

Nextcloud owns the transport and storage responsibility that it already provides. PoRE owns the capture, durable browser preservation, artifact identity and integrity proof up to the host boundary. V1 completion is the successful, size-checked and hash-checked handoff into Nextcloud Files.
