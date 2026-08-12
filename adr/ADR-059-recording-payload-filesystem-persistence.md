# ADR-059 Recording Payload Filesystem Persistence

- Status: Accepted
- Date: 2026-08-12
- Related issue: #7 Persist actual recording payload

## Context

ADR-055 defines the physical RecordingArtifact boundary and the
Artifact → Track → Chunk filesystem structure. ADR-058 defines a
storage-provider-independent logical payload reference and explicitly
defers concrete payload persistence to #7.

The recorder now carries actual payload bytes from `CaptureResult` into
`RecordingArtifact`. The filesystem provider therefore needs a concrete,
provider-local representation for those bytes without introducing a
specific audio codec or format decision.

## Decision

The `FilesystemPersistenceProvider` persists each RecordingChunk payload
as one opaque payload file below the chunk directory:

```text
<root>/
    <artifact-id>/
        artifact.json
        tracks/
            <track-id>/
                chunks/
                    chunk-000001.payload
                    chunk-000002.payload
```

The `.payload` suffix deliberately does not claim an audio codec or
container format. The bytes are persisted exactly as supplied by the
capture layer. A future audio-format decision may replace this physical
representation without changing the logical payload reference boundary
from ADR-058.

`artifact.json` stores the logical payload reference and payload size for
each chunk. The payload bytes themselves remain in the chunk payload file
and are not embedded into JSON.

A complete artifact is written into a temporary artifact directory first.
Each payload file is written and published inside that temporary
structure. The complete temporary directory is then renamed to the final
artifact directory. Therefore the final artifact directory is not exposed
until its metadata and payload files have been written.

Loading an artifact requires every declared payload file to exist and to
have the declared size. A missing or size-mismatched payload therefore
makes the artifact unavailable to the current provider load/list path.

## Not decided here

This ADR does not define:

- a concrete audio codec or container format;
- a checksum or hashing algorithm;
- corruption recovery semantics beyond rejecting incomplete payloads;
- remote or database persistence;
- payload encryption or compression.

Those concerns remain subject to later decisions, in particular #8 for
integrity and recovery validation.

## Consequences

- The current filesystem provider persists actual recording bytes.
- Payload metadata and payload bytes remain physically separate.
- The existing `PersistenceProvider` interface remains unchanged.
- The artifact model remains independent of absolute filesystem paths.
- Missing or truncated payload files cannot silently produce a loaded
  artifact.
- The current `.payload` representation is intentionally format-neutral.
