# Reality Check — Nextcloud V1 Archaeology

## Purpose

Document the comparison between the historical Nextcloud V1 implementation and the current `develop` architecture before reusing any provider code.

## Findings

The historical V1 implementation was not merely a prototype. It contained:

- Nextcloud connection/configuration and credential handling
- HTTPS enforcement
- WebDAV transport primitives
- `NextcloudArtifactTransfer`
- configurable remote root
- SHA-256 payload verification
- idempotency and conflict handling
- chunked/resumable upload
- synchronization persistence integration
- orchestration wiring
- provider-specific synchronization composition

The current `develop` branch retains the provider-neutral synchronization boundary and the `RemoteArtifact` view, while the concrete Nextcloud implementation is absent from the active tree.

## Architectural conclusion

The historical implementation should **not** be merged wholesale.

The current architecture has since established a cleaner provider boundary: the complete `RecordingArtifact` remains the transfer unit and provider-neutral metadata is carried alongside it. The historical Nextcloud implementation should therefore be treated as an implementation reference, not as an architectural source of truth.

## Reuse strategy

When rebuilding the Nextcloud provider, reuse only concepts that still match the current contracts:

1. HTTPS-only connection policy.
2. App Password authentication model.
3. WebDAV as the Nextcloud data plane.
4. Remote-root configuration.
5. Manifest/payload integrity checks.
6. Idempotency and conflict semantics.
7. Chunked/resumable upload where required.
8. Provider-specific behavior isolated below the vendor-neutral transfer boundary.

Do not restore legacy application modules or provider-specific concepts into Core merely because they existed in the historical implementation.

## Next step

Implement the smallest current-architecture-compatible Nextcloud connector, then add a deterministic integration test before connecting a real Nextcloud instance.
