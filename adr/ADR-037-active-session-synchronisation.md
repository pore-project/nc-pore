# ADR-037: Active Session Synchronisation

- Status: Proposed
- Date: 2026-08-17
- Decision Type: Architecture
- Supersedes: relevant parts of ADR-009

---

## Context

NC-PoRe records participants independently on different devices. A sample-based internal time model is already established by ADR-009, but a shared start time alone cannot guarantee long-session synchronization.

Independent devices may use different clock sources and may exhibit clock drift over the duration of a recording session.

The Ennuicastr architecture demonstrates an active synchronization approach in which clients continuously establish their relationship to a server-side time reference and attach timing information to captured data.

## Decision

NC-PoRe shall treat **active session synchronization** as an explicit architectural concern.

A distributed recording session must maintain a continuously usable temporal relationship between participants rather than relying solely on a common start time.

The concrete synchronization protocol is deliberately not fixed by this ADR. It must support at least:

- a stable session time reference
- participant-specific timing information
- sample- or frame-level positioning
- detection and handling of clock drift
- reconstruction of tracks after interruptions
- sufficiently precise timing for long recordings

The protocol must remain independent of the eventual storage provider and production output format.

## Architectural Principle

> A distributed recording session is synchronized continuously, not merely started simultaneously.

Synchronization metadata is part of the capture information required to reconstruct a recording reliably.

## Consequences

### Positive

- long recordings can remain temporally coherent despite different device clocks
- clock drift becomes an explicit, testable concern
- recovery and rejoin can use the same temporal model
- synchronization remains independent of final media formats

### Negative

- additional protocol and state complexity
- synchronization quality must be measured and tested
- network timing and temporary connectivity must be handled carefully

## Alternatives Considered

### Common start timestamp only

Rejected. Different device clocks can drift, making start-time-only synchronization insufficient for long recordings.

### Post-production manual alignment only

Rejected as the primary mechanism. Manual alignment may remain useful as an optional production tool, but the capture system must provide reliable temporal information itself.

## Relationship to Existing Architecture

This decision refines ADR-009. The existing sample-based track time model remains valid and becomes the data-level representation used by an active synchronization mechanism.

It also interacts directly with the Recording Lifecycle and recovery decisions defined in the Core.

## Future Considerations

A later ADR must define the concrete synchronization protocol, clock estimation, drift correction strategy, tolerance limits and test scenarios for sessions of several hours.

---

# English Version

## Context

NC-PoRe records participants independently on different devices. ADR-009 already establishes a sample-based internal time model, but a shared start time alone cannot guarantee synchronization over long sessions.

Different devices may use different clock sources and may drift over time.

Ennuicastr demonstrates an active synchronization approach using a server-side time relationship and timing information attached to captured data.

## Decision

NC-PoRe shall treat **active session synchronization** as an explicit architectural concern.

A distributed recording session must maintain a usable temporal relationship between participants instead of relying only on a common start time.

The concrete protocol is intentionally left open, but it must support a stable session time reference, participant timing information, sample/frame positioning, clock-drift handling, interruption recovery and long-session accuracy.

## Principle

> A distributed recording session is synchronized continuously, not merely started simultaneously.

Synchronization metadata is part of the capture information required for reliable reconstruction.

## Consequences

- improved long-session temporal coherence
- explicit and testable clock-drift handling
- synchronization can support recovery and rejoin
- protocol remains independent of storage and output formats
- additional protocol and testing complexity

## Future Work

A subsequent ADR shall define the concrete synchronization protocol, drift correction, tolerance limits and long-session tests.
