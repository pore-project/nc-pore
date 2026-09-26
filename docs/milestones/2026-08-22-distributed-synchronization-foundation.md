# Distributed Recording & Synchronisation Foundation

Date: 2026-08-22

## Status

**Historical architecture snapshot.**

The architectural foundation defined by **#66 — Distributed Recording & Synchronisation** was implemented through the completed work packages **#140, #143, #144, #145 and #146**. This document records that predecessor architecture and its verification history; it is not a description of the current V1 transport coordinator.

## Implemented boundaries

- `RecordingArtifact` has an explicit synchronization lifecycle with recovery and idempotent terminal synchronization semantics.
- Synchronization work is persisted separately from local recording artifact data.
- Pending work can be recovered after process interruption.
- Artifact transfer is defined through a vendor- and transport-neutral application boundary.
- Transfer outcomes distinguish successful, already-synchronized, retryable, conflict, integrity and terminal failure cases.
- Application orchestration processes persisted pending work deterministically.
- Interrupted work is recovered before another processing attempt.
- The persisted local artifact and its manifest identity are validated before successful synchronization is recorded.
- Retryable and offline failures leave synchronization work recoverable while leaving the local artifact available.
- End-to-end application tests cover success, offline/retry, interruption recovery, terminal failure and manifest mismatch.

## Deliberate boundaries

The implementation does not introduce:

- a concrete remote-storage vendor
- network transport or authentication
- background-worker optimization
- track/chunk-level distributed synchronization
- UI/client behavior

These remain later implementation concerns behind the established transfer boundary.

## Architectural result

The synchronization path is now separated into the following layers:

```text
RecordingArtifact
      |
      v
SynchronizationWork
      |
      v
SynchronizationWorkStore
      |
      v
SynchronizationOrchestrator
      |
      v
ArtifactTransfer
      |
      v
TransferResult
```

Local recording therefore remains independent of network availability, while persisted synchronization work can be resumed deterministically after interruption or temporary remote unavailability.


## Current V1 architecture note

The former `SynchronizationWork` / `SynchronizationWorkStore` / `SynchronizationOrchestrator` transport architecture documented above is not part of the current `develop` line.

Current V1 uses the durable browser completion/transport boundary defined by ADR-083:

```text
durable browser capture
      |
      v
persisted completion/transport state
      |
      v
prepare -> direct Nextcloud upload -> verify -> close
      |
      v
Core Recording completion
```

The browser completion job persists transport progress in the durable browser capture manifest and reconstructs recoverable work after browser/application restart. The current V1 does not reintroduce the historical synchronization work queue as a second production transport coordinator.

The historical real-provider tests documented above remain useful architectural evidence, but current V1 repeated-synchronization and restart-recovery behavior is deliberately tracked as real-world Beta validation in #287.
