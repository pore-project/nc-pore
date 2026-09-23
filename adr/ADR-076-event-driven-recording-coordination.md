# ADR-076: Event-Driven Recording Coordination

## Status

Accepted

## Date

2026-09-21

## Context

NC-PoRe coordinates distributed recording across multiple Talk clients. The Core remains the authoritative source for fachliche recording state and lifecycle transitions.

Periodic polling of the Core or participant state is not part of the production recording architecture. It creates unnecessary request traffic and turns a diagnostic mechanism into a runtime dependency.

## Decision

NC-PoRe uses event-driven coordination.

One-shot Core reads are permitted only at concrete lifecycle or user-action boundaries, especially during bootstrap, explicit recording start, and recovery from a missing predecessor event.

Ongoing recording transitions are distributed through the PoRE-owned recording coordination transport using a dedicated event type, `pore-recording`, with an explicit protocol version and recording identity.

Signaling messages are transport events, not a second authoritative state source. The Core remains responsible for accepting or rejecting fachliche transitions. A received event triggers the corresponding local technical action and, where required, the authorized Core command.

A client that cannot establish the Talk signaling connection does not silently start a distributed recording. It reports the missing coordination transport instead.

## Coordination events

- `begin` tells participating clients to start their local capture.
- `ready` reports that a client has completed local technical preparation and readiness.
- `opening` tells clients to record and confirm the Opening Signet.
- `stop` tells participating clients to perform their controlled local stop.
- `production_closed` propagates production closure for the UI state.

## Consequences

- No production recording-state polling timer exists in PoRE.
- One-shot reads remain available for lifecycle boundaries and recovery.
- Talk is the coordination transport, not the owner of PoRE local capture.
- Core remains authoritative for fachliche lifecycle state.
- Signaling remains separate from local recording persistence and completion processing.
