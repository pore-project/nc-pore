# NC-PoRE Recording Information Surface V1

## Purpose

This document describes the product-facing information surface defined by ADR-086. It is an implementation-oriented companion to the architectural/product decision.

The surface is a PoRE layer inside the host experience. It is not a standalone browser recording application and must not depend on brittle host DOM selectors.

## 1. Visibility model

Recording information is visible according to the concrete recording participant set and role.

| User | Recording information |
|---|---|
| **Host** | Full recording control plus participant/readiness information |
| **Recording participant** | Own recording state, own READY state and local recording indication |
| **Session member not participating in this recording** | No recording status information |

Session membership alone does not grant recording visibility.

## 2. Host surface

### Before recording

The host sees:

- preparation state;
- the expected recording participants;
- current READY state while preparation is in progress;
- aggregate readiness such as `2 / 3 bereit` where useful;
- the primary **Aufnahme starten** action when the fachliche start boundary permits it.

### During recording

The host sees:

- an unmistakable active recording indication;
- recording start time and/or elapsed time;
- the recording participant set;
- **Aufnahme beenden**.

### After stop

The host sees:

- stopping or transfer state;
- confirmed completion when the server-side boundary has been reached;
- Production status;
- where applicable, **Production endgültig schließen**.

No artificial percentage is shown when no reliable progress source exists.

## 3. Recording participant surface

A recording participant sees:

- the own recording state;
- own READY state during preparation;
- clear local-recording indication;
- the relevant completion state after stop.

Technical IDs, hashes, transport URLs and connector internals are not part of the normal product surface.

## 4. Non-participant surface

A session member who is not part of the recording sees no recording information.

The UI must not expose another participant's:

- recording state;
- READY state;
- stop/transfer progress;
- artifact status.

## 5. State presentation

The surface distinguishes:

- preparation;
- READY;
- Opening;
- Recording;
- stopping or transfer;
- recording stopped;
- recording confirmed;
- Production closed;
- error.

The UI must not invent additional fachliche states merely to simplify rendering.

## 6. Lifecycle presentation

The normal start path is presented as:

```text
Preparation
    ->
Recording start requested by Host
    ->
Recording participant set fixed
    ->
Local capture
    ->
READY
    ->
Opening
    ->
Recording
```

The normal completion path is:

```text
Recording
    ->
Stop requested by Host
    ->
Stopping
    ->
Local completion
    ->
Artifact/transport processing
    ->
Server confirmation
    ->
Recording Completed
```

Production closure remains separate from Recording completion.

## 7. Transparency and accessibility

The surface must communicate recording status without relying on color alone.

Active recording, preparation, stopping/transfer, confirmed completion and Production closure must remain distinguishable through text and accessible semantics.

## 8. Integration boundary

The UI consumes the explicit PoRE/Talk integration context.

It must not derive fachliche state from:

- arbitrary Talk DOM elements;
- CSS classes;
- versionspecific element IDs;
- incidental Talk UI wording.

Core/Application state remains authoritative.

## 9. Scope boundary

This document does not define:

- complete technical diagnostics;
- exact pixels or icon geometry;
- a standalone recording application;
- artificial transport percentages;
- automatic DAW synchronization;
- drift correction.

Visual refinement and host-specific presentation details may evolve during alpha hardening while the information model remains stable.

## Related decision

- ADR-086 Talk Recording Control Surface V1
