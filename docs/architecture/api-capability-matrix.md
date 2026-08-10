# NC-PoRe API Capability Matrix

## Deutsche Version ([English version below](#english-version))

---

# Zweck

Dieses Dokument stellt den aktuell definierten fachlichen API-Surface dem tatsächlichen Implementierungsstand von NC-PoRe gegenüber.

Es unterscheidet ausdrücklich zwischen:

* **Defined** – fachlich als API-Fähigkeit definiert
* **Implemented** – im aktuellen Code implementiert
* **Exposed** – über eine externe API erreichbar
* **Deferred** – bewusst noch nicht als API-Fähigkeit definiert

Das Dokument beschreibt den aktuellen Zustand.

Es definiert keinen Kommunikationsmechanismus.

---

# Capability Matrix

| Capability                              | Defined |       Implemented | Exposed | Status                    |
| --------------------------------------- | ------: | ----------------: | ------: | ------------------------- |
| Create Production Session               |     Yes |               Yes |      No | Implemented internally    |
| Get Production Session                  |     Yes |               Yes |      No | Repository capability implemented |
| Start Production Session                |     Yes |               Yes |      No | Implemented internally    |
| Complete Production Session             |     Yes |               Yes |      No | Implemented internally    |
| Add Participant                         |     Yes |               Yes |      No | Implemented internally    |
| List Participants                       |     Yes |               Yes |      No | Implemented internally    |
| Add Recording                           |     Yes |               Yes |      No | Implemented internally    |
| List Recordings                         |     Yes |               Yes |      No | Implemented internally    |
| List Activity History                   |     Yes |               Yes |      No | Implemented internally    |
| Start Recording through external API    |      No |  Yes (internally) |      No | Deferred                  |
| Complete Recording through external API |      No |  Yes (internally) |      No | Deferred                  |
| Recording Artifacts API                 |      No | Yes (technically) |      No | Deferred                  |
| Persistence API                         |      No | Yes (technically) |      No | Deferred                  |

---

# Production Session

## Create Production Session

**Defined:** Yes

**Implemented:** Yes

The Core provides `ProductionSession::new(...)`.

A newly created session starts in the `Created` state and contains an initial `SessionCreated` activity event.

**Exposed:** No

---

## Get Production Session

**Defined:** Yes

**Implemented:** Yes

The Core provides `ProductionSessionRepository` with `store(...)` and `get(...)`.

The repository contract is implemented and validated by tests for:

- storing and retrieving a Production Session
- rejecting duplicate Production Session identifiers
- returning no result for unknown identifiers

The repository currently provides an in-memory reference implementation.
A production persistence adapter has not yet been implemented.

**Exposed:** No

---

## Start Production Session

**Defined:** Yes

**Implemented:** Yes

The Core provides `ProductionSession::start()`.

The operation validates the lifecycle transition from `Created` to `Active` and creates a `SessionStarted` activity event.

**Exposed:** No

---

## Complete Production Session

**Defined:** Yes

**Implemented:** Yes

The Core provides `ProductionSession::complete()`.

The operation requires:

* status `Active`
* an existing Owner

A successful completion changes the session to `Completed` and creates a `SessionCompleted` activity event.

**Exposed:** No

---

# Participants

## Add Participant

**Defined:** Yes

**Implemented:** Yes

The Core provides `ProductionSession::add_participation(...)`.

The Core prevents the same Participant from being added more than once to a Production Session.

**Exposed:** No

---

## List Participants

**Defined:** Yes

**Implemented:** Yes

The Core provides read-only access to the session's Participations.

**Exposed:** No

---

# Recordings

## Add Recording

**Defined:** Yes

**Implemented:** Yes

The Core provides `ProductionSession::add_recording(...)`.

The Production Session owns the relationship between the Production Session and its Recordings.

**Exposed:** No

---

## List Recordings

**Defined:** Yes

**Implemented:** Yes

The Core provides read-only access to the session's Recordings.

**Exposed:** No

---

## Recording Lifecycle

The current Core supports:

```text
Prepared
    ↓
Recording
    ↓
Completed
```

The corresponding domain operations exist internally.

They are not currently defined as external API capabilities.

This is intentional.

The existence of a public Rust method does not automatically make that method part of the external API contract.

---

# Activity History

## List Activity History

**Defined:** Yes

**Implemented:** Yes

The Core exposes the session's Activity History.

The current activity types are:

* `SessionCreated`
* `SessionStarted`
* `SessionCompleted`

**Exposed:** No

---

# Recorder Capabilities

The Recorder contains additional technical capabilities.

These include:

* `RecorderApplication`
* `RecorderWorkflow`
* `CaptureProvider`
* `RecordingArtifact`
* `ArtifactRegistry`
* `ArtifactCoordinator`
* `RecordingArtifactProcessor`
* `PersistenceProvider`
* `FilesystemPersistenceProvider`
* `ArtifactRecoveryService`

These capabilities are implemented internally but are not currently defined as domain API capabilities.

They therefore remain:

**Defined:** No
**Implemented:** Yes
**Exposed:** No
**Status:** Deferred

This distinction is intentional.

Technical implementation does not automatically define an external API.

---

# Exposed API

The current exposed API surface is:

```text
None
```

No external communication mechanism has yet been implemented.

This is consistent with:

* ADR-028 API Design Principles
* ADR-034 Implementation Architecture

A concrete communication mechanism will be selected only when a concrete client or another external system requires it.

---

# Current State

The current implementation can therefore be summarized as:

```text
                    Defined    Implemented    Exposed

Production Session     Yes          Yes          No
Participants           Yes          Yes          No
Recordings             Yes          Yes          No
Activity History       Yes          Yes          No
Session Retrieval      Yes          Yes          No
Recorder Internals     No           Yes          No
External API           —            No           No
```

Production Session retrieval is implemented through the in-memory
`ProductionSessionRepository`.

A production persistence adapter has not yet been implemented.

---

# Relationship to Other Documents

```text
ADR-028 / ADR-034
        ↓
API Surface
        ↓
Capability Matrix
        ↓
Implementation
        ↓
External API
```

The documents have different responsibilities:

* ADRs explain architectural decisions.
* `api-surface.md` defines domain capabilities that may form an API.
* This document compares those capabilities with the implementation.
* Source Code implements the capabilities.
* A future external API exposes selected capabilities through a concrete communication mechanism.

---

# Status

## Draft

This document reflects the implementation state at the time of creation.

It should be updated when:

* a defined capability becomes implemented
* a capability becomes externally exposed
* a deferred capability becomes part of the domain API
* an architectural decision changes the API boundary

---

# Guiding Principle

A capability being implemented internally does not mean that it is part of the external API.

A capability being defined by the API does not mean that it is already implemented.

NC-PoRe keeps these states separate deliberately.

---

# English Version ([Deutsche Version oben](#deutsche-version))


---

# Purpose

This document compares the currently defined domain API surface with the actual implementation state of NC-PoRe.

It explicitly distinguishes between:

* **Defined** – defined as a domain API capability
* **Implemented** – implemented in the current code
* **Exposed** – available through an external API
* **Deferred** – deliberately not yet defined as an API capability

This document describes the current state.

It does not define a communication mechanism.

---

# Capability Matrix

| Capability                              | Defined |       Implemented | Exposed | Status                    |
| --------------------------------------- | ------: | ----------------: | ------: | ------------------------- |
| Create Production Session               |     Yes |               Yes |      No | Implemented internally    |
| Get Production Session                  |     Yes |               Yes |      No | Repository capability implemented |
| Start Production Session                |     Yes |               Yes |      No | Implemented internally    |
| Complete Production Session             |     Yes |               Yes |      No | Implemented internally    |
| Add Participant                         |     Yes |               Yes |      No | Implemented internally    |
| List Participants                       |     Yes |               Yes |      No | Implemented internally    |
| Add Recording                           |     Yes |               Yes |      No | Implemented internally    |
| List Recordings                         |     Yes |               Yes |      No | Implemented internally    |
| List Activity History                   |     Yes |               Yes |      No | Implemented internally    |
| Start Recording through external API    |      No |  Yes (internally) |      No | Deferred                  |
| Complete Recording through external API |      No |  Yes (internally) |      No | Deferred                  |
| Recording Artifacts API                 |      No | Yes (technically) |      No | Deferred                  |
| Persistence API                         |      No | Yes (technically) |      No | Deferred                  |

---

# Production Session

## Create Production Session

**Defined:** Yes

**Implemented:** Yes

The Core provides `ProductionSession::new(...)`.

A newly created session starts in the `Created` state and contains an initial `SessionCreated` activity event.

**Exposed:** No

---

## Get Production Session

**Defined:** Yes

**Implemented:** Yes

The Core provides `ProductionSessionRepository` with `store(...)` and `get(...)`.

The repository contract is implemented and validated by tests for:

- storing and retrieving a Production Session
- rejecting duplicate Production Session identifiers
- returning no result for unknown identifiers

The repository currently provides an in-memory reference implementation.
A production persistence adapter has not yet been implemented.

**Exposed:** No

---

## Start Production Session

**Defined:** Yes

**Implemented:** Yes

The Core provides `ProductionSession::start()`.

The operation validates the lifecycle transition from `Created` to `Active` and creates a `SessionStarted` activity event.

**Exposed:** No

---

## Complete Production Session

**Defined:** Yes

**Implemented:** Yes

The Core provides `ProductionSession::complete()`.

The operation requires:

* status `Active`
* an existing Owner

A successful completion changes the session to `Completed` and creates a `SessionCompleted` activity event.

**Exposed:** No

---

# Participants

## Add Participant

**Defined:** Yes

**Implemented:** Yes

The Core provides `ProductionSession::add_participation(...)`.

The Core prevents the same Participant from being added more than once to a Production Session.

**Exposed:** No

---

## List Participants

**Defined:** Yes

**Implemented:** Yes

The Core provides read-only access to the session's Participations.

**Exposed:** No

---

# Recordings

## Add Recording

**Defined:** Yes

**Implemented:** Yes

The Core provides `ProductionSession::add_recording(...)`.

The Production Session owns the relationship between the Production Session and its Recordings.

**Exposed:** No

---

## List Recordings

**Defined:** Yes

**Implemented:** Yes

The Core provides read-only access to the session's Recordings.

**Exposed:** No

---

## Recording Lifecycle

The current Core supports:

```text
Prepared
    ↓
Recording
    ↓
Completed
```

The corresponding domain operations exist internally.

They are not currently defined as external API capabilities.

This is intentional.

The existence of a public Rust method does not automatically make that method part of the external API contract.

---

# Activity History

## List Activity History

**Defined:** Yes

**Implemented:** Yes

The Core exposes the session's Activity History.

The current activity types are:

* `SessionCreated`
* `SessionStarted`
* `SessionCompleted`

**Exposed:** No

---

# Recorder Capabilities

The Recorder contains additional technical capabilities.

These include:

* `RecorderApplication`
* `RecorderWorkflow`
* `CaptureProvider`
* `RecordingArtifact`
* `ArtifactRegistry`
* `ArtifactCoordinator`
* `RecordingArtifactProcessor`
* `PersistenceProvider`
* `FilesystemPersistenceProvider`
* `ArtifactRecoveryService`

These capabilities are implemented internally but are not currently defined as domain API capabilities.

They therefore remain:

**Defined:** No
**Implemented:** Yes
**Exposed:** No
**Status:** Deferred

This distinction is intentional.

Technical implementation does not automatically define an external API.

---

# Exposed API

The current exposed API surface is:

```text
None
```

No external communication mechanism has yet been implemented.

This is consistent with:

* ADR-028 API Design Principles
* ADR-034 Implementation Architecture

A concrete communication mechanism will be selected only when a concrete client or another external system requires it.

---

# Current State

The current implementation can therefore be summarized as:

```text
                    Defined    Implemented    Exposed

Production Session     Yes          Yes          No
Participants           Yes          Yes          No
Recordings             Yes          Yes          No
Activity History       Yes          Yes          No
Session Retrieval      Yes          Yes          No
Recorder Internals     No           Yes          No
External API           —            No           No
```

Production Session retrieval is implemented through the in-memory
`ProductionSessionRepository`.

A production persistence adapter has not yet been implemented.

---

# Relationship to Other Documents

```text
ADR-028 / ADR-034
        ↓
API Surface
        ↓
Capability Matrix
        ↓
Implementation
        ↓
External API
```

The documents have different responsibilities:

* ADRs explain architectural decisions.
* `api-surface.md` defines domain capabilities that may form an API.
* This document compares those capabilities with the implementation.
* Source Code implements the capabilities.
* A future external API exposes selected capabilities through a concrete communication mechanism.

---

# Status

## Draft

This document reflects the implementation state at the time of creation.

It should be updated when:

* a defined capability becomes implemented
* a capability becomes externally exposed
* a deferred capability becomes part of the domain API
* an architectural decision changes the API boundary

---

# Guiding Principle

An internally implemented capability is not automatically part of the external API.

A capability defined by the API is not necessarily already implemented.

NC-PoRe deliberately keeps these states separate.
