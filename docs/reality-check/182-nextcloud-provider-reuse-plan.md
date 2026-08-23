# Reality Check — Nextcloud Provider Reuse Plan

The historical Nextcloud V1 implementation is substantial and should be reused where it still matches the current architecture rather than rewritten from scratch.

## Reuse directly where compatible

The historical implementation already provides the core infrastructure pieces needed for the real provider:

- `NextcloudCredentials` with debug redaction
- `NextcloudConnectionConfig` with remote-root configuration
- HTTPS-only endpoint validation
- `NextcloudConnection`
- `WebDavClient` and transport abstraction
- `NextcloudArtifactTransfer`
- SHA-256 manifest/payload verification
- idempotency and conflict handling
- small-file upload and large-file chunked upload paths
- provider-specific error mapping

These components live in the infrastructure layer and therefore fit the current provider boundary.

## Adapt rather than copy blindly

The old transfer implementation loads the artifact from persistence because that was the transfer-boundary shape at the time. The current architecture should retain the current provider-neutral request/remote-artifact contract and adapt the infrastructure connector around it.

No legacy application module should be restored merely to make the historical implementation compile.

## Current implementation target

Port the compatible historical infrastructure modules to the current branch with the smallest possible compatibility changes, then add deterministic transport-backed tests. Only after those tests pass should a real Home Nextcloud be connected.
