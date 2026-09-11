# ADR-068 Implementation Reconciliation — V1

* Status: Accepted ADR-068 reconciliation
* Date: 2026-09-03
* Decision Type: Architecture / Implementation

---

## Purpose

This document maps the current recording implementation evidence to the accepted ADR-068 lifecycle. It does **not** change ADR-068 and does not introduce a second recording-start or synchronization protocol.

## Lifecycle mapping

| ADR-068 step | Current implementation evidence | Architectural owner |
|---|---|---|
| Host starts recording | Browser command invokes the existing Application/Core recording boundary. | Core / Application |
| Recording participant set is frozen | Recording participation is derived from Core session/recording state; the browser does not maintain a parallel participant set. | Core / Application |
| Local capture starts | Browser runtime starts its local `MediaRecorder` capture. | Browser client |
| `READY` | Client reports readiness only after local capture is actually active. | Browser client → Application/Core |
| Opening Sync Signet | Existing recording workflow owns the opening synchronization barrier; the browser slice does not implement a parallel signet protocol. | Application / Core |
| Recording active | Client renders and executes the Core/Application decision; it is not the domain authority. | Core / Application |
| Host stops recording | Stop is initiated through the existing recording boundary. | Core / Application |
| Closing Sync Signet | Existing recording workflow coordinates the closing synchronization barrier before technical recorder shutdown. | Application / Core |
| Local recorder stops | Client processes the stop command and acknowledges actual technical completion. | Browser client |
| Stop `OK` | Participant acknowledgement is part of the existing recording workflow barrier. | Browser client → Application/Core |
| Artifact persistence | Finalized browser capture is handed through `BrowserRecordingHandoff` into the existing `CaptureResult` / `RecordingArtifact` / persistence path. | Browser client / Recorder |
| Recording completion | Completion requires the persisted artifact to be present and valid before Core completion is accepted. | Application / Core |

## Layer boundaries

### Core / Application

Core and Application remain authoritative for recording lifecycle and state, recording participant selection, host-controlled start/stop, synchronization barriers, participant acknowledgements, artifact association and completion.

The browser client must not reproduce these rules locally as a second authoritative state machine.

### Browser client

The browser client is responsible for the command surface, local capability/capture execution, local recorder control, readiness after actual capture activation, technical stop acknowledgement, neutral capture handoff, and presentation of Core/Application state.

The current `session_client_recording_vertical_slice` is deliberately a boundary-validation and runtime-demonstration surface. Its minimal HTTP/JSON transport and browser `MediaRecorder` payload handling are test infrastructure, not the production Talk protocol.

### Talk connector

The Talk connector remains responsible only for Talk-specific media lifecycle and for adapting the active Talk audio track into the generic PoRE capture boundary.

Talk-specific behavior must not leak into Core recording rules, and Core must not depend on Talk DOM or browser-specific details.

## Implementation evidence

The current vertical slice demonstrates the client-facing path:

`session → recording authorization → local capture → READY → opening workflow → recording → stop → stop acknowledgement → artifact persistence → completion`

It also enforces that completion cannot be accepted merely because stop acknowledgements have arrived: the persisted artifact must exist and be valid first.

## Remaining V1 work

These are implementation tasks, not changes to ADR-068:

1. Connect the real Nextcloud Talk media lifecycle through the generic capture boundary.
2. Validate the complete lifecycle with real Talk sessions across the target browser families.
3. Implement the production Talk UI according to ADR-072 and the V1 product-facing surface defined in #230.
4. Keep reconnect/rejoin recovery, drift correction, continuous synchronization and automatic DAW alignment outside this V1 reconciliation.

## Relationship to existing decisions

- **ADR-068** remains authoritative for the distributed recording start/stop and synchronization lifecycle.
- **ADR-072** remains authoritative for visual recording status semantics.
- **ADR-073** remains authoritative for the local recording safety cutoff after connectivity loss.
- Existing capture-boundary, artifact, persistence and workflow decisions remain unchanged.

The reconciliation therefore documents the actual boundary usage without changing the architecture.
