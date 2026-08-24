# Reality Check — Transfer Boundary

## Issue
#180 — Reality Check: Transfer boundary must expose persisted artifact payload

## Status
Planned

## Purpose
Close the remaining gap between the vendor-neutral synchronization boundary and a real remote-storage connector.

## Architectural rule
The application transfer boundary remains vendor- and transport-neutral. A concrete connector must be able to obtain the complete persisted `RecordingArtifact` representation required for transfer, including payload bytes and integrity information, without introducing Nextcloud/WebDAV types into the application layer.

## Current boundary
`ArtifactTransferRequest` currently carries:

- artifact identity
- manifest hash
- provider-neutral transfer metadata

The next implementation step must extend this boundary so the transfer implementation can access the persisted artifact content needed for an actual transfer.

## Explicit non-goals
This step does **not** decide:

- Nextcloud/WebDAV authentication
- credential storage
- codec/container format
- upload optimization
- background-worker behavior

## Next reality check
After the boundary is corrected, implement a concrete Nextcloud/WebDAV connector and execute an end-to-end transfer against the Home Nextcloud.
