# NC-PoRe API Surface

## Deutsche Version ([English version below](#english-version))

---

# Zweck

Dieses Dokument beschreibt die fachlichen Fähigkeiten von NC-PoRe, die grundsätzlich über eine API zugänglich gemacht werden können.

Es definiert zunächst **keinen konkreten Kommunikationsmechanismus**.

Insbesondere werden hier nicht festgelegt:

* HTTP
* REST
* WebSocket
* gRPC
* JSON
* konkrete Netzwerkarchitektur

Das Dokument beschreibt die fachliche API-Oberfläche.

Die konkrete technische Kommunikationsgrenze wird erst festgelegt, wenn ein konkreter Client oder ein anderes externes System sie benötigt.

---

# Grundsatz

Die API beschreibt die **Fähigkeiten des Systems**.

Sie beschreibt nicht:

* interne Rust-Datenstrukturen
* konkrete Modulgrenzen
* interne Implementierungsdetails
* verwendete Persistence Provider
* Audio- oder Hardwareimplementierungen
* interne Workflow-Koordination

Die API ist damit eine stabile fachliche Grenze zwischen NC-PoRe und seiner Umgebung.

Bezug:

* ADR-028 API Design Principles
* ADR-034 Implementation Architecture

---

# Production Session

Die Production Session ist die zentrale fachliche Einheit von NC-PoRe.

Die API muss daher die wesentlichen Operationen auf einer Production Session abbilden können.

---

## Create Production Session

### Zweck

Eine neue Production Session erzeugen.

### Eingabe

* Production Identifier

### Ergebnis

* erzeugte Production Session
* initialer Status `Created`

### Fachliche Eigenschaften

Eine neu erzeugte Session enthält zunächst:

* keine Participants
* keine Recordings
* eine Activity History mit `SessionCreated`

---

## Get Production Session

### Zweck

Eine bestehende Production Session abrufen.

### Eingabe

* Production Identifier

### Ergebnis

Die fachlich relevanten Eigenschaften der Production Session:

* Identifier
* Status
* Participants / Participations
* Recordings
* Activity History

### Fehler

* Session nicht vorhanden

---

## Start Production Session

### Zweck

Eine Production Session starten.

### Eingabe

* Production Identifier

### Ergebnis

* aktualisierte Production Session
* Status `Active`

### Voraussetzungen

Die Session muss sich im Status `Created` befinden.

### Fehler

* ungültiger Zustandsübergang

### Activity

Beim erfolgreichen Start wird ein `SessionStarted` Activity Event erzeugt.

---

## Complete Production Session

### Zweck

Eine laufende Production Session abschließen.

### Eingabe

* Production Identifier

### Ergebnis

* aktualisierte Production Session
* Status `Completed`

### Voraussetzungen

* Session befindet sich im Status `Active`
* Session besitzt einen Owner

### Fehler

* ungültiger Zustandsübergang
* fehlender Owner

### Activity

Beim erfolgreichen Abschluss wird ein `SessionCompleted` Activity Event erzeugt.

---

# Participants

Participants beschreiben Personen oder andere Identitäten, die an einer Production Session beteiligt sind.

Die Verantwortung innerhalb einer Production Session wird durch eine Participation und deren Role beschrieben.

---

## Add Participant

### Zweck

Einen Participant einer Production Session zuordnen.

### Eingabe

* Production Session Identifier
* Participant Identifier
* Participant Role

### Ergebnis

* erzeugte Participation

### Fachliche Regeln

Ein Participant kann innerhalb einer Production Session nur einmal beteiligt sein.

### Fehler

* Participant bereits vorhanden

---

## List Participants

### Zweck

Die Participants einer Production Session abrufen.

### Eingabe

* Production Session Identifier

### Ergebnis

* Liste der Participants bzw. Participations der Session

---

# Participant Roles

Die derzeit definierten Rollen sind:

* `Owner`
* `Producer`
* `Participant`
* `Guest`

Die Role beschreibt die Verantwortung eines Participants **innerhalb einer Production Session**.

Die API darf daher Participant Identity und Session-spezifische Role nicht miteinander vermischen.

---

# Recordings

Recordings gehören fachlich zu einer Production Session.

Das derzeitige Domainmodell definiert den Lifecycle:

```text
Prepared
    ↓
Recording
    ↓
Completed
```

---

## Add Recording

### Zweck

Ein Recording einer Production Session zuordnen.

### Eingabe

* Production Session Identifier
* Recording

### Ergebnis

* Recording ist der Production Session zugeordnet

### Fachliche Eigenschaft

Die Production Session besitzt die Beziehung zwischen Production Session und Recording.

---

## List Recordings

### Zweck

Die Recordings einer Production Session abrufen.

### Eingabe

* Production Session Identifier

### Ergebnis

* Liste der Recordings

---

# Recording Lifecycle

Das derzeitige Recording-Modell unterstützt folgende fachliche Zustandsänderungen:

```text
Prepared → Recording → Completed
```

Die API kann diese Zustände anzeigen.

Eine eigenständige API-Operation zum Starten oder Abschließen eines Recordings wird **in dieser Version noch nicht als externe API-Fähigkeit definiert**.

Der aktuelle Core stellt diese Operationen zwar intern bereit, aber der konkrete externe Produktionsablauf für Recordings ist noch nicht definiert.

Dies verhindert, dass interne Domain-Methoden vorschnell zu einem stabilen externen Vertrag werden.

---

# Activity History

Die Production Session führt eine fachliche Activity History.

Der aktuelle Core definiert:

* `SessionCreated`
* `SessionStarted`
* `SessionCompleted`

---

## List Activity History

### Zweck

Die Activity History einer Production Session abrufen.

### Eingabe

* Production Session Identifier

### Ergebnis

* chronologische Liste der Activity Events

Die Activity History ist Teil der fachlichen Session-Darstellung und nicht lediglich ein internes Logging-System.

---

# Fehlersemantik

Die fachliche API muss fachliche Fehler von technischen Fehlern unterscheiden.

Der aktuelle Core definiert für Production Sessions:

* `ParticipantAlreadyExists`
* `MissingOwner`
* `InvalidStateTransition`

Diese Fehler beschreiben fachliche Zustände bzw. Regelverletzungen.

Technische Fehler, beispielsweise:

* Storage nicht verfügbar
* Datei nicht lesbar
* Netzwerkfehler
* Kommunikationsfehler

sind **nicht Bestandteil des aktuellen Core-Fehlermodells**.

Sie gehören zu den jeweiligen technischen Grenzen und werden bei der späteren Umsetzung der Kommunikationsgrenze separat behandelt.

---

# Bewusste Abgrenzungen

Die folgenden Fähigkeiten sind derzeit **nicht Teil dieses fachlichen API-Surfaces**:

* RecorderApplication
* RecorderWorkflow
* CaptureProvider
* RecordingArtifact
* ArtifactRegistry
* ArtifactCoordinator
* RecordingArtifactProcessor
* PersistenceProvider
* FilesystemPersistenceProvider
* ArtifactRecoveryService

Diese Komponenten implementieren technische Abläufe und Grenzen.

Sie werden nicht automatisch zu API-Ressourcen, nur weil ihre Typen oder Methoden in Rust `pub` sind.

---

# Recording Artifacts

Recording Artifacts werden derzeit nicht als Teil der fachlichen Production-Session-API definiert.

Sie gehören zur technischen Recorder-Architektur.

Die Abgrenzung folgt insbesondere:

* ADR-039 Recording Architecture and Capture Boundary
* ADR-040 Recorder Workflow and Capture Lifecycle Coordination
* ADR-042 Recording Artifact Model and Lifecycle Boundary
* ADR-051 Recording Artifact Processing Boundary
* ADR-053 Artifact Recovery and Consistency Boundary

Eine spätere API für Assets bzw. Recording Artifacts kann auf diesen technischen Grundlagen aufgebaut werden.

Sie wird jedoch erst definiert, wenn der entsprechende fachliche Produktionsworkflow konkret benötigt wird.

---

# Persistence

Persistence ist keine fachliche API-Ressource.

Die Persistenz wird durch technische Grenzen abstrahiert.

Der aktuelle Recorder unterstützt:

* `PersistenceProvider`
* `InMemoryPersistenceProvider`
* `FilesystemPersistenceProvider`

Die Wahl des konkreten Persistence Providers bleibt außerhalb der fachlichen API.

Bezug:

* ADR-044 Persistence Provider Interface
* ADR-048 Artifact Registry and Persistence Coordination
* ADR-052 Local Filesystem Persistence Provider

---

# API und interne Implementierung

Die aktuelle Implementierung besitzt bereits öffentliche Rust-Typen und Methoden.

Diese bilden die Grundlage für die fachliche API-Betrachtung, sind aber nicht identisch mit dem API-Vertrag.

Die externe API soll insbesondere nicht von folgenden Eigenschaften abhängig werden:

* Rust-Modulstruktur
* konkreten Struct-Feldern
* Generic Types
* konkreten Providern
* internen Koordinationsobjekten

Die API beschreibt stattdessen fachliche Operationen und deren Ergebnisse.

---

# Aktueller API-Surface

Der derzeit definierte fachliche API-Surface umfasst:

```text
Production Session
│
├── Create
├── Get
├── Start
├── Complete
│
├── Participants
│   ├── Add
│   └── List
│
├── Recordings
│   ├── Add
│   └── List
│
└── Activity History
    └── List
```

Dieser Surface ist bewusst kleiner als die derzeit vorhandene technische Implementierung.

---

# Noch nicht festgelegt

Dieses Dokument legt bewusst noch nicht fest:

* konkrete Transportprotokolle
* URL-Strukturen
* HTTP-Methoden
* JSON-Schemata
* Authentication-Protokolle
* Authorization-Protokolle
* Netzwerkarchitektur
* öffentliche Entwickler-API
* Versionierungsmechanismen auf Transportebene

Diese Entscheidungen erfolgen erst bei konkretem Bedarf.

---

# Beziehung zu Architekturentscheidungen

Der API-Surface basiert insbesondere auf:

* ADR-028 API Design Principles
* ADR-031 Identity, Authentication and User Roles
* ADR-034 Implementation Architecture
* ADR-035 Domain Lifecycle and State Transition Management
* ADR-038 Core Implementation Structure and Module Organization
* ADR-039 Recording Architecture and Capture Boundary
* ADR-040 Recorder Workflow and Capture Lifecycle Coordination
* ADR-042 Recording Artifact Model and Lifecycle Boundary
* ADR-044 Persistence Provider Interface
* ADR-048 Artifact Registry and Persistence Coordination
* ADR-051 Recording Artifact Processing Boundary
* ADR-052 Local Filesystem Persistence Provider
* ADR-053 Artifact Recovery and Consistency Boundary

---

# Status

## Draft

Dieses Dokument beschreibt den ersten aus dem bestehenden Core abgeleiteten fachlichen API-Surface.

Es ist noch kein implementierter API-Vertrag.

Änderungen am fachlichen API-Surface werden nachvollziehbar dokumentiert und bei architektonisch relevanten Änderungen über ADRs abgesichert.

---

# English Version ([Deutsche Version oben](#deutsche-version))

---

# Purpose

This document describes the domain capabilities of NC-PoRe that may be exposed through an API.

It does **not** define a concrete communication mechanism.

In particular, this document does not select:

* HTTP
* REST
* WebSocket
* gRPC
* JSON
* concrete network architecture

The document describes the domain API surface.

The concrete technical communication boundary will be selected when a concrete client or another external system requires it.

---

# Principle

The API describes **system capabilities**.

It does not describe:

* internal Rust data structures
* concrete module boundaries
* internal implementation details
* persistence providers
* audio or hardware implementations
* internal workflow coordination

The API therefore forms a stable domain boundary between NC-PoRe and its environment.

References:

* ADR-028 API Design Principles
* ADR-034 Implementation Architecture

---

# Production Session

The Production Session is the central domain entity of NC-PoRe.

The API therefore needs to support the essential operations on a Production Session.

---

## Create Production Session

### Purpose

Create a new Production Session.

### Input

* Production Identifier

### Result

* created Production Session
* initial status `Created`

### Domain properties

A newly created session initially contains:

* no participants
* no recordings
* an Activity History containing `SessionCreated`

---

## Get Production Session

### Purpose

Retrieve an existing Production Session.

### Input

* Production Identifier

### Result

The domain-relevant properties of the Production Session:

* identifier
* status
* participants / participations
* recordings
* Activity History

### Errors

* session not found

---

## Start Production Session

### Purpose

Start a Production Session.

### Input

* Production Identifier

### Result

* updated Production Session
* status `Active`

### Preconditions

The session must be in status `Created`.

### Errors

* invalid state transition

### Activity

A successful start creates a `SessionStarted` Activity Event.

---

## Complete Production Session

### Purpose

Complete an active Production Session.

### Input

* Production Identifier

### Result

* updated Production Session
* status `Completed`

### Preconditions

* session is in status `Active`
* session has an owner

### Errors

* invalid state transition
* missing owner

### Activity

A successful completion creates a `SessionCompleted` Activity Event.

---

# Participants

Participants describe persons or other identities involved in a Production Session.

Responsibility within a Production Session is represented through a Participation and its Role.

---

## Add Participant

### Purpose

Assign a Participant to a Production Session.

### Input

* Production Session Identifier
* Participant Identifier
* Participant Role

### Result

* created Participation

### Domain rules

A Participant can only participate once within a Production Session.

### Errors

* participant already exists

---

## List Participants

### Purpose

Retrieve the Participants of a Production Session.

### Input

* Production Session Identifier

### Result

* list of Participants / Participations

---

# Participant Roles

The currently defined roles are:

* `Owner`
* `Producer`
* `Participant`
* `Guest`

The Role describes responsibility of a Participant **within a Production Session**.

The API must therefore keep Participant Identity separate from session-specific Role.

---

# Recordings

Recordings belong to a Production Session.

The current domain model defines the lifecycle:

```text
Prepared
    ↓
Recording
    ↓
Completed
```

---

## Add Recording

### Purpose

Associate a Recording with a Production Session.

### Input

* Production Session Identifier
* Recording

### Result

* Recording is associated with the Production Session

### Domain property

The Production Session owns the relationship between the Production Session and its Recordings.

---

## List Recordings

### Purpose

Retrieve the Recordings of a Production Session.

### Input

* Production Session Identifier

### Result

* list of Recordings

---

# Recording Lifecycle

The current Recording model supports the following domain state transitions:

```text
Prepared → Recording → Completed
```

The API can expose these states.

A dedicated external API operation for starting or completing a Recording is **not defined as an external API capability in this version**.

The current Core exposes these operations internally, but the concrete external recording workflow has not yet been defined.

This prevents internal domain methods from prematurely becoming a stable external contract.

---

# Activity History

The Production Session maintains a domain Activity History.

The current Core defines:

* `SessionCreated`
* `SessionStarted`
* `SessionCompleted`

---

## List Activity History

### Purpose

Retrieve the Activity History of a Production Session.

### Input

* Production Session Identifier

### Result

* chronological list of Activity Events

The Activity History is part of the domain representation of a session and is not merely an internal logging mechanism.

---

# Error Semantics

The domain API must distinguish domain errors from technical errors.

The current Core defines the following Production Session errors:

* `ParticipantAlreadyExists`
* `MissingOwner`
* `InvalidStateTransition`

These errors describe domain conditions or violated domain rules.

Technical errors such as:

* unavailable storage
* unreadable files
* network failures
* communication failures

are **not part of the current Core error model**.

They belong to the respective technical boundaries and will be addressed separately when the communication boundary is implemented.

---

# Explicit Scope Boundaries

The following capabilities are currently **not part of this domain API surface**:

* RecorderApplication
* RecorderWorkflow
* CaptureProvider
* RecordingArtifact
* ArtifactRegistry
* ArtifactCoordinator
* RecordingArtifactProcessor
* PersistenceProvider
* FilesystemPersistenceProvider
* ArtifactRecoveryService

These components implement technical workflows and boundaries.

They do not automatically become API resources merely because their Rust types or methods are public.

---

# Recording Artifacts

Recording Artifacts are currently not defined as part of the domain Production Session API.

They belong to the technical Recorder architecture.

The boundary is defined in particular by:

* ADR-039 Recording Architecture and Capture Boundary
* ADR-040 Recorder Workflow and Capture Lifecycle Coordination
* ADR-042 Recording Artifact Model and Lifecycle Boundary
* ADR-051 Recording Artifact Processing Boundary
* ADR-053 Artifact Recovery and Consistency Boundary

A future API for Assets or Recording Artifacts may build on these technical foundations.

It will be defined when the corresponding domain production workflow requires it.

---

# Persistence

Persistence is not a domain API resource.

Persistence is abstracted through technical boundaries.

The current Recorder supports:

* `PersistenceProvider`
* `InMemoryPersistenceProvider`
* `FilesystemPersistenceProvider`

The concrete Persistence Provider remains outside the domain API.

References:

* ADR-044 Persistence Provider Interface
* ADR-048 Artifact Registry and Persistence Coordination
* ADR-052 Local Filesystem Persistence Provider

---

# API and Internal Implementation

The current implementation already contains public Rust types and methods.

These form the basis for the API analysis but are not identical to the API contract.

The external API must not depend on:

* Rust module structure
* concrete struct fields
* generic types
* concrete providers
* internal coordination objects

The API instead describes domain operations and their results.

---

# Current API Surface

The currently defined domain API surface is:

```text
Production Session
│
├── Create
├── Get
├── Start
├── Complete
│
├── Participants
│   ├── Add
│   └── List
│
├── Recordings
│   ├── Add
│   └── List
│
└── Activity History
    └── List
```

This surface is deliberately smaller than the currently implemented technical system.

---

# Not Yet Defined

This document deliberately does not define:

* concrete transport protocols
* URL structures
* HTTP methods
* JSON schemas
* authentication protocols
* authorization protocols
* network architecture
* public developer API
* transport-level versioning mechanisms

These decisions will be made when concrete requirements exist.

---

# Relationship to Architecture Decisions

The API surface is primarily based on:

* ADR-028 API Design Principles
* ADR-031 Identity, Authentication and User Roles
* ADR-034 Implementation Architecture
* ADR-035 Domain Lifecycle and State Transition Management
* ADR-038 Core Implementation Structure and Module Organization
* ADR-039 Recording Architecture and Capture Boundary
* ADR-040 Recorder Workflow and Capture Lifecycle Coordination
* ADR-042 Recording Artifact Model and Lifecycle Boundary
* ADR-044 Persistence Provider Interface
* ADR-048 Artifact Registry and Persistence Coordination
* ADR-051 Recording Artifact Processing Boundary
* ADR-052 Local Filesystem Persistence Provider
* ADR-053 Artifact Recovery and Consistency Boundary

---

# Status

## Draft

This document describes the first domain API surface derived from the existing Core.

It is not yet an implemented API contract.

Changes to the domain API surface will be documented and architecturally relevant changes will be protected through ADRs.
