# ADR-082: Browser-to-Application Artifact Transport Boundary

- Status: Accepted
- Date: 2026-09-05

## Context

NC-PoRe now has a browser-local recording finalization boundary and an application-side adapter that can transform the finalized browser payload into the existing `RecordingArtifactProcessor` path.

The repository does not currently contain a server runtime for the Rust `application` crate. The Nextcloud app is a PHP host integration, while the Rust workspace contains domain, application, recorder, and infrastructure libraries. Therefore a browser HTTP endpoint must not be implemented as if the Rust application library were already an HTTP server.

The existing persistence and synchronization architecture is already complete and must remain the only artifact path:

`Browser capture -> Application handoff -> RecordingArtifact -> PersistenceProvider -> SynchronizationWork -> ArtifactTransfer -> Nextcloud`

## Decision

The browser-to-application handoff is a distinct transport boundary.

The transport carries:

- `Core.ProductionId`
- `Core.RecordingId`
- `Recorder.RecordingSessionId`
- browser technical capture identity
- track/capture metadata
- the finalized payload

The transport MUST NOT:

- write directly to WebDAV
- create a second persistence store
- create a second recording lifecycle
- use `ProductionId` or `RecordingId` as an `ArtifactId`
- make the browser depend on recorder persistence or Nextcloud WebDAV details

The Rust `application` crate remains transport-neutral. A concrete HTTP runtime may be introduced later as a composition root that invokes the existing application boundary. The Nextcloud PHP application may provide host authentication/session integration, but it must not become a second recording persistence implementation.

Until such a runtime exists, no endpoint is considered an accepting persistence endpoint. A route that returns a successful acceptance without invoking the application boundary is explicitly forbidden.

## Consequences

The browser transport contract can be implemented and tested independently from deployment/runtime concerns.

A future runtime must prove this chain end-to-end before the browser transport is considered complete:

1. authenticated browser request
2. transport decoding/validation
3. application `BrowserRecordingArtifact`
4. existing `RecordingArtifactProcessor`
5. existing `PersistenceProvider`
6. synchronization enqueue
7. existing `ArtifactTransfer`
8. authoritative recording completion

The existing recorder and Nextcloud provider require no architectural change for this boundary.

## Non-decisions

This ADR does not choose:

- a Rust HTTP framework
- a process model or deployment topology
- whether Nextcloud proxies to a separate PoRE service
- REST versus another transport protocol
- authentication token format
- remote artifact path semantics

Those decisions belong to the concrete runtime/composition slice and must be made before implementing a real accepting HTTP endpoint.
