# ADR-082: Browser-to-Application Artifact Transport Boundary

- Status: Superseded
- Date: 2026-09-05
- Superseded by: ADR-083

## Historical context

This ADR documented the original browser-to-application transport boundary before a concrete verified Nextcloud transport existed.

The original decision was that the browser-to-application handoff should be distinct from persistence and synchronization, and that the browser must not become responsible for Nextcloud storage details.

Those architectural principles remain valid.

## Superseded transport rule

The original ADR explicitly prohibited browser-to-WebDAV transport. That rule is superseded for the finalized-artifact V1 transport by ADR-083.

The accepted mechanism is now:

```text
browser completion job
    -> authenticated PoRe prepare control request
    -> temporary Nextcloud upload authorization
    -> bounded browser upload to that authorization
    -> authenticated PoRe verification
    -> authenticated/idempotent authorization close
    -> authoritative completion
```

This does **not** restore the former browser -> PoRe PHP multipart -> Nextcloud Files storage path. The PoRe controller no longer receives the finalized audio payload.

## Principles retained

The following architectural constraints remain in force:

- Core remains provider-neutral.
- Browser technical capture identity remains distinct from `core.ProductionId` and `core.RecordingId`.
- No second authoritative persistence store is introduced.
- Local durable preservation remains the recovery source until verified transport completion.
- Nextcloud-specific concepts remain outside Core.
- A successful HTTP response is not itself a semantic completion acknowledgement.

See ADR-083 for the accepted V1 transport lifecycle and verification semantics.
