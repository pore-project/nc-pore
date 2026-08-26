# Distributed Recording & Synchronisation Foundation

Date: 2026-08-22

## Status

The architectural foundation defined by **#66 — Distributed Recording & Synchronisation** is implemented through the completed work packages **#140, #143, #144, #145 and #146**.

The foundation is now also verified against the real Nextcloud provider with real CPAL audio capture and a persisted synchronization lifecycle.

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

## Real-provider verification

The established transfer boundary has now been exercised with the real Nextcloud WebDAV provider rather than only provider-neutral tests.

The regression suite verifies these production-relevant paths with real CPAL capture:

1. **Real recording transfer** — a real audio capture becomes a `RecordingArtifact`, is synchronized to Nextcloud, and the remote manifest and payload are verified.
2. **Idempotent repeat** — the same artifact can be processed again without producing an inconsistent synchronization state.
3. **Restart recovery** — after the synchronization orchestrator is dropped, the persisted work store is reconstructed; the artifact remains `Synchronized` and a restarted orchestrator reports `NoPendingWork`.

These checks use the same application orchestration and infrastructure transfer path intended for production. They are therefore retained as explicit integration regression tests rather than being replaced by mocks.

## Deliberate boundaries

The implementation does not introduce:

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

The real-provider path additionally validates the intended runtime boundary:

```text
Real CPAL Capture
      |
      v
RecordingArtifact
      |
      v
Persisted SynchronizationWork
      |
      v
SynchronizationOrchestrator
      |
      v
NextcloudArtifactTransfer
      |
      v
Nextcloud / WebDAV
      |
      v
Persisted Synchronized State
      |
      v
Restart Recovery
```

Local recording therefore remains independent of network availability, while persisted synchronization work can be resumed deterministically after interruption or temporary remote unavailability.
