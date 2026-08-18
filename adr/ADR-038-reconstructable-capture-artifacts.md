# ADR-038: Reconstructable Capture Artifacts

- Status: Proposed
- Date: 2026-08-17
- Decision Type: Architecture

---

## Context

A recording captured on a participant device may be interrupted by network failures, browser or application crashes, device problems or temporary loss of connectivity.

The capture process therefore cannot be required to produce a perfect final media file at recording time.

The comparison with Ennuicastr highlights a useful principle: captured data should remain sufficiently complete and temporally identifiable so that a coherent track can be reconstructed later.

NC-PoRe also needs to distinguish source capture data from derived production outputs.

## Decision

NC-PoRe treats **raw capture data as a first-class artifact** of a Recording.

The raw capture artifact is the recoverable source from which later production artifacts can be reconstructed or regenerated.

Capture data must be:

- sufficiently complete to reconstruct the recorded material
- associated with participant and session identity
- associated with temporal positioning
- identifiable at chunk/segment level where required
- distinguishable between complete and incomplete capture state
- preserved independently of derived processing outputs according to storage policy

A capture artifact is not required to be a finished WAV, AIFF, FLAC or other production file.

## Architectural Principle

> Capture does not have to be perfect. It has to be complete and reconstructable enough.

Processing is responsible for turning reconstructed capture data into production artifacts.

## Rejoin and Interrupted Streams

An interruption must not automatically make the entire participant track unusable.

Where technically and semantically possible, a subsequent capture segment from the same participant and session may be associated with the existing track and reconstructed into one coherent timeline.

The exact rejoin semantics remain subject to the synchronization model and later implementation decisions.

## Artifact Layers

The intended conceptual separation is:

```text
Recording
   |
   +-- Raw Capture Artifact(s)
   |       |
   |       +-- chunks / segments
   |       +-- timing information
   |       +-- capture metadata
   |
   +-- Derived Production Artifact(s)
           |
           +-- lossless master candidates
           +-- exports
           +-- processed variants
```

Raw capture is the source of truth for reconstruction. Derived artifacts may be regenerated when processing rules or export requirements change.

## Audio Format Consideration

A lossless format such as FLAC is a strong candidate for a canonical derived audio master because it is lossless while reducing storage requirements compared with uncompressed PCM containers such as WAV or AIFF.

This ADR deliberately does **not** mandate FLAC for local capture. Browser and recorder implementations may use another capture representation as long as the resulting capture artifact satisfies the reconstruction requirements.

## Consequences

### Positive

- failures during capture do not necessarily destroy the complete recording
- raw source data remains available for reprocessing
- production artifacts can be regenerated
- capture and production formats remain decoupled
- recovery semantics become explicit

### Negative

- more metadata must be persisted
- incomplete and complete artifact states must be modeled
- reconstruction logic becomes a first-class subsystem
- storage requirements may increase when raw capture is retained

## Alternatives Considered

### Store only the final media file

Rejected. A partially written or corrupted final file can make recovery impossible and prevents reliable reprocessing from source capture data.

### Make capture directly produce the canonical production format

Rejected as a general architectural requirement. It would couple browser/client capture to production format and make recovery and future processing changes harder.

## Relationship to Existing Architecture

This decision builds on ADR-026 (Session Data and Storage Architecture) and ADR-035 (Domain Lifecycle and State Transition Management).

It introduces a more explicit distinction between source capture artifacts and derived production artifacts without making a concrete storage technology or audio codec mandatory.

## Future Considerations

A later implementation decision must define artifact identity, chunk addressing, integrity information, completion semantics, retention and reconstruction rules.

---

# English Version

## Context

Participant recordings can be interrupted by network failures, browser or application crashes, device failures or temporary connectivity loss.

Capture therefore cannot be required to produce a perfect final media file at recording time.

Captured data should instead remain sufficiently complete and temporally identifiable for later reconstruction.

## Decision

NC-PoRe treats **raw capture data as a first-class Recording artifact**.

It is the recoverable source from which production artifacts can later be reconstructed or regenerated.

Capture data must retain sufficient identity, timing, segmentation and completion information to support reconstruction.

A capture artifact is not required to be a finished WAV, AIFF, FLAC or other production file.

## Principle

> Capture does not have to be perfect. It has to be complete and reconstructable enough.

Processing creates derived production artifacts from reconstructed capture data.

## Rejoin

Where technically and semantically possible, a later segment from the same participant and session may be associated with an existing track and reconstructed into one coherent timeline.

## Audio Format

FLAC is a strong candidate for a canonical lossless derived audio master, but this ADR deliberately does not mandate FLAC for local capture.

## Consequences

- improved failure recovery
- source data remains available for reprocessing
- derived artifacts can be regenerated
- capture remains decoupled from production formats
- additional metadata and reconstruction logic are required

## Future Work

Later decisions must define artifact identity, chunk addressing, integrity, completion semantics, retention and reconstruction rules.
