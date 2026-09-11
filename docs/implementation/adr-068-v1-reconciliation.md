# ADR-068 and the current V1 recording path

## Purpose

This note reconciles the accepted ADR-068 recording-start model with the recording lifecycle that is currently implemented in the repository. It is deliberately an implementation map, not a second protocol specification.

## Architectural boundary

ADR-068 owns the distributed recording lifecycle. The technical recorder owns local capture and artifact construction. The host connector owns only host-specific media/control integration. Persistence and synchronization remain the existing PoRE boundaries.

The intended direction is therefore:

`host/session lifecycle -> recorder start/stop -> local artifact -> persistence -> synchronization`

A Talk connector may translate Talk events into host-neutral application events, but it must not implement a parallel recording state machine, artifact model, or synchronization protocol.

## Lifecycle mapping

| ADR-068 boundary | Current implementation evidence | Remaining work |
| --- | --- | --- |
| Host requests recording start | `RecordingWorkflow::start_recording` and the application recording orchestration exist as domain/application boundaries | Connect the distributed host command to the application boundary. |
| Recording participant set is frozen | Recording/session domain objects already identify the recording participants | Make the participant snapshot explicit at the ADR-068 start boundary. |
| Participant starts actual local capture | `RecorderApplication::start` owns the technical capture start | Add a host-neutral READY event only after local capture has successfully started. |
| READY aggregation | No distributed READY aggregation is currently implemented | Add this to the session/application layer, not the Talk connector. |
| Opening Sync Signet | Not yet connected to the recording lifecycle | Integrate the existing synchronization/signaling domain boundary before logical recording start. |
| Logical recording start | Domain recording workflow has an explicit start transition | Gate it on the ADR-068 start conditions and opening signet. |
| Host requests stop | `RecordingWorkflow::request_stop` and recorder stop are separate application operations | Connect the distributed host stop to the same application lifecycle. |
| Closing Sync Signet | Not yet connected to technical recorder shutdown | Emit/coordinate the closing signet before invoking technical recorder shutdown. |
| Technical recorder stop | `RecorderApplication::stop` finalizes capture and processes the artifact | Keep this as the technical shutdown boundary; do not use Talk room termination as a substitute. |
| Participant OK | No distributed post-stop confirmation protocol is currently implemented | Add a host-neutral post-stop confirmation and aggregation boundary. |
| Artifact persistence | Existing artifact processor/persistence path is authoritative | Feed finalized browser/host capture into this path rather than adding persistence beside it. |
| Synchronization | Existing provider-neutral synchronization lifecycle remains authoritative | Connect ADR-068 completion to the existing synchronization work rather than creating a new queue/protocol. |

## Layer ownership

### Session/application layer

Owns the shared recording lifecycle and its domain transitions:

- host start/stop intent
- recording participant snapshot
- READY aggregation
- opening/closing signet coordination
- post-stop OK aggregation
- transition from technical recording to completed recording artifact

These events must remain host-neutral.

### Host connector

The Talk connector is an adapter. It may provide:

- Talk-specific host control integration
- access to Talk's active local media track
- Talk lifecycle notifications needed to maintain that integration

It must not provide:

- `RecordingArtifact` persistence
- a Talk-specific synchronization queue
- a second recording state machine
- distributed READY/signets as a Talk-only protocol

### Recorder

The recorder remains responsible for:

- actual local capture
- capture finalization
- technical metadata and track construction
- preservation and artifact processing

A browser `Blob` or other host-produced payload is therefore an input at the capture/application boundary, not proof that a `RecordingArtifact` already exists.

### Persistence and synchronization

The existing durable preservation, artifact, completion, and synchronization paths remain authoritative. ADR-068 adds lifecycle semantics around them; it does not replace them.

## Required implementation order

The smallest coherent remaining slices are:

1. Define the host-neutral start/READY application contract and participant snapshot.
2. Integrate opening/closing synchronization signets with the existing recording workflow.
3. Define the neutral finalized-capture handoff so browser/host payloads enter the normal artifact path without a parallel model.
4. Add post-stop OK aggregation and completion semantics.
5. Validate the complete path against the existing persistence/synchronization lifecycle and real Talk runtime.

The slices should remain separate at their architectural boundaries, but related domain/application work may be combined when it forms one deterministic vertical change.

## Current V1 status

The repository currently proves the local technical recording and artifact path, including explicit application-level start/stop orchestration. It does **not** yet prove the distributed ADR-068 protocol. That distinction is intentional and must remain visible until the missing host-neutral boundaries are implemented and verified.

## Validation guidance

For implementation work, deterministic tests should verify lifecycle ordering and failure behavior. Runtime validation should additionally verify that:

- a host start cannot report READY before local capture is active;
- the logical recording start follows the opening signet;
- host stop triggers the closing signet before technical recorder shutdown;
- room termination alone does not stop the recording;
- a finalized capture reaches the existing artifact/persistence path;
- post-stop confirmation follows successful local finalization;
- retry/recovery continues through the existing synchronization lifecycle.
