# NC-PoRe V1 Implementation Plan

## Status

Active — V1 hardening and productization

## Goal

Deliver the first publicly usable NC-PoRe version around the existing local recording artifact and Nextcloud synchronization architecture.

## Work sequence

### 1. Repository and documentation consolidation

- synchronize `develop`, documentation and architecture records
- remove obsolete status claims
- consolidate historical issues and milestones without rewriting history

### 2. Legacy/dead-code audit

Inspect before removing:

- `application/src/lib_source.rs`
- historical recorder modules
- compatibility exports
- feasibility/proof-of-concept code
- apparently unused public APIs

Nothing is removed solely because it looks old; current references and architectural intent must be established first.

### 3. Synchronization hardening

- pass recording metadata through the application/transfer boundary to the Nextcloud connector
- keep provider-specific metadata and transport behavior out of Core
- verify remote payload integrity before reporting success
- preserve idempotency and conflict semantics
- keep V1 upload behavior restart-safe unless real Nextcloud testing demonstrates a need for true chunk continuation

### 4. Real Nextcloud verification

Maintain a reproducible integration/smoke test covering:

```text
persisted RecordingArtifact
        ↓
SynchronizationWork
        ↓
Nextcloud connector
        ↓
remote artifact
        ↓
remote manifest + payload verification
        ↓
repeat identical transfer
        ↓
AlreadySynchronized
```

Also test interrupted upload, conflict, missing/corrupt payload, invalid manifest, invalid credentials and HTTPS-only configuration.

### 5. First usable client

The existing `ClientSessionService` is the application-facing facade. The current HTTP feasibility harness is not the production client.

The first client must cover the minimum vertical workflow:

```text
create production session
        ↓
manage participants
        ↓
start session
        ↓
record locally
        ↓
persist artifact
        ↓
synchronize
        ↓
show synchronization state
        ↓
complete session
```

Transport, serialization, authentication and UI remain outside the domain Core.

### 6. Public V1 gate

Public release requires:

- reproducible green build and tests
- reproducible real-Nextcloud smoke test
- HTTPS-only Nextcloud configuration
- safe credential handling
- verified remote artifact integrity
- tested recovery and retry behavior
- complete minimal client workflow
- documented installation and operation

## Explicit V1 exclusions

- additional remote providers
- custom remote-storage infrastructure
- track/chunk-level synchronization as the domain unit
- delta synchronization
- automatic conflict resolution
- OIDC as a V1 requirement
- premature performance optimization without measurements
