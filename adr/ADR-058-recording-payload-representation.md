# ADR-058 Recording Payload Representation

- Status: Accepted
- Date: 2026-08-12
- Related issue: #5 Define recording payload representation

## Context

`CaptureResult` currently transfers technical track and chunk metadata into a `RecordingArtifact`, while `RecordingChunk` contains only a sequence number. This is sufficient to describe artifact structure, but it does not yet describe the actual recording payload.

The payload representation must remain compatible with the existing boundaries established by ADR-054, ADR-055, ADR-056 and ADR-057. In particular, the artifact model must not acquire concrete filesystem knowledge merely because local persistence is the first storage implementation.

## Decision

A `RecordingChunk` represents a logically identifiable section of the technical recording payload. The chunk therefore receives a storage-provider-independent payload reference rather than embedding a concrete filesystem path.

The payload description consists conceptually of:

- a logical payload reference;
- payload size metadata;
- integrity metadata required by later validation and recovery.

The logical payload reference is deliberately independent of the physical persistence provider. It must not encode absolute filesystem paths or other storage-specific locations.

The actual payload bytes are technical recording data and remain below the domain `Recording` boundary. `CaptureResult` remains technically separate from `RecordingArtifact` as defined by ADR-056. The existing artifact factory remains responsible for translating capture-side structures into artifact-side structures.

Concrete filesystem representation and publication rules are deferred to issue #7. The exact integrity mechanism and recovery validation rules are deferred to issue #8.

## Rationale

This preserves the separation between:

```text
CaptureResult
    -> technical capture representation

RecordingArtifact
    -> technical persisted-recording representation

PersistenceProvider
    -> physical storage representation
```

A payload reference allows the artifact model to describe real recording data without coupling the model to the current filesystem implementation. This keeps later storage providers possible without changing the recording domain model.

The decision also preserves the distinction between technical capture tracks and domain participants. A recording track remains a technical audio stream; the payload reference identifies its technical data and does not become a participant or role reference.

## Consequences

### Positive

- `RecordingChunk` can describe actual recording payload without embedding storage-specific paths.
- The `CaptureResult` / `RecordingArtifact` boundary remains intact.
- Filesystem persistence can choose its physical payload layout in #7 without changing the architectural model.
- Integrity and recovery semantics can be defined explicitly in #8.
- Alternative persistence providers remain possible.

### Negative

- The current model needs an additional payload-reference abstraction and associated metadata.
- Payload persistence cannot be completed independently of the decision made here.

## Not decided here

This ADR does not define:

- a concrete audio file format;
- a concrete filesystem path or directory layout;
- atomic publication details for persisted payloads;
- a specific checksum or hashing algorithm;
- recovery behavior for incomplete or corrupted payloads.

Those decisions belong to the subsequent persistence and recovery work, primarily #7 and #8.

## Relation to existing ADRs

- ADR-054 Recording Artifact and Local Recording Data Association
- ADR-055 Filesystem Persistence Layout
- ADR-056 Capture Result and Recording Artifact Data Boundary
- ADR-057 Domain Recording to Recording Artifact Association Boundary
