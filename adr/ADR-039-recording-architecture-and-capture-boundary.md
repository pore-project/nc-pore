# ADR-039: Recording Architecture and Capture Boundary

* Status: Accepted
* Date: 2026-08-08
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

## Kontext

NC-PoRe wurde mit dem Ziel entwickelt, verteilte Podcast-Produktionen mit lokaler Aufnahme zu ermöglichen.

Die bisherigen Architekturentscheidungen definieren lokale Aufnahme, die Production Session als zentrale fachliche Einheit, den Core als Domain Authority und die Trennung fachlicher und technischer Verantwortlichkeiten.

Mit der technischen Umsetzung müssen insbesondere folgende Konzepte getrennt bleiben:

* fachliches Recording
* technische Audioaufnahme
* Recording Artifact
* lokale Verwaltung und Persistenz des Artifacts
* Verarbeitung und Synchronisation

Ein Recording ist nicht mit einer Audiodatei gleichzusetzen.

---

# Entscheidung

NC-PoRe trennt das fachliche Recording-Modell von der technischen Capture-Implementierung.

Der Core beschreibt das fachliche Recording. Die Capture-Schicht erzeugt daraus technische Aufnahmedaten. Diese werden als Recording Artifact behandelt und über definierte technische Grenzen an Registry, Persistence und weitere Verarbeitungskomponenten übergeben.

```text
Domain

Production Session
        |
        v
Recording
        |
        v
Capture Boundary
        |
        v
Recording Artifact
        |
        +--------------------+
        |                    |
        v                    v
Artifact Registry      Persistence
        |
        v
Artifact Processing
```

Die konkrete technische Implementierung kann diese Grenzen unterschiedlich realisieren. Die fachlichen Verantwortlichkeiten bleiben davon unabhängig.

---

# Recording als fachliches Produktionsobjekt

Ein `Recording` ist Bestandteil des fachlichen Produktionsmodells.

Es beschreibt insbesondere:

* seine Zugehörigkeit zu einer Production Session
* fachliche Zustände
* fachliche Beziehungen zu Participants
* relevante fachliche Metadaten
* seine fachliche Identität und seinen Lebenszyklus

Der Core ist für die fachliche Integrität dieses Modells verantwortlich.

---

# Verantwortung des Core

Der Core ist verantwortlich für die fachliche Bedeutung eines Recordings, insbesondere für:

* Recording-Lebenszyklus
* Beziehung zwischen Recording und Production Session
* Beziehung zwischen Recording und Participant
* fachliche Validierung
* erlaubte fachliche Zustandsübergänge
* fachliche Recording-Metadaten
* fachliche Ereignisse

Der Core entscheidet nicht, wie Audiodaten technisch erzeugt oder gespeichert werden.

---

# Nicht-Verantwortung des Core

Der Core enthält keine konkrete technische Audioaufnahme.

Dazu gehören insbesondere:

* Audio-Hardware-Ansteuerung
* Mikrofonzugriffe
* plattformspezifische Audio-APIs
* konkrete Audio-Backends
* Capture-Puffer
* Echtzeit-Audioverarbeitung
* konkrete Dateischreibvorgänge
* Filesystem-Operationen
* technische Artifact-Verwaltung

Diese Verantwortlichkeiten liegen außerhalb des Domain-Modells.

---

# Capture als technische Operation

Die Capture-Komponente erfasst Audiodaten von einer technischen Audioquelle.

Sie ist verantwortlich für:

* Initialisierung des Audio-Captures
* Erfassung des Audio-Streams
* technische Pufferung
* Übergabe erfasster Audiodaten an den Recording Workflow
* technische Behandlung von Capture-Fehlern

Sie entscheidet nicht über Production-Session-Regeln, Benutzerrollen, fachliche Berechtigungen oder fachliche Recording-Zustände.

---

# Capture Boundary

Die Grenze zwischen fachlichem Recording und technischer Audioaufnahme wird durch eine definierte Capture Boundary gebildet.

```text
Core

Recording Lifecycle
        |
        v
Capture Boundary
        |
        v
Capture Implementation
        |
        v
Audio Backend
        |
        v
Audio Source
```

Der Core ist damit nicht von einem konkreten Audio-Backend abhängig.

---

# Recording Artifact

Das Ergebnis der technischen Aufnahme wird als `Recording Artifact` behandelt.

Ein Artifact repräsentiert die tatsächlich erzeugten technischen Aufnahmedaten und kann beispielsweise technische Dateiinformationen, Aufnahmeeigenschaften und die Zuordnung zu Session und Participant tragen.

Die genaue technische Verwaltung von Artifacts ist nicht Bestandteil dieser ADR.

---

# Trennung von Recording und Artifact

```text
Recording

= fachliches Produktionsobjekt

Recording Artifact

= technische Repräsentation
  tatsächlich erzeugter Aufnahmedaten
```

Ein Recording kann fachlich existieren, ohne dass bereits ein fertiges Artifact vorhanden ist.

Umgekehrt definiert ein Artifact nicht selbst die fachliche Bedeutung eines Recordings.

Die Verbindung erfolgt über definierte fachliche beziehungsweise technische Referenzen.

---

# Local Recording Principle

Die Aufnahme folgt dem grundlegenden Prinzip aus ADR-001 und ADR-029:

```text
Lokal aufnehmen

↓

Aufnahmedaten lokal sichern

↓

Aufnahme fachlich abschließen

↓

Artifact kontrolliert weiterverarbeiten

↓

Synchronisieren
```

Eine laufende Audioaufnahme darf nicht von einer stabilen Netzwerkverbindung abhängig sein.

Ein Ausfall der Netzwerkverbindung während des Captures darf die lokale Audioaufnahme nicht automatisch beenden.

---

# Track Model

NC-PoRe verwendet getrennte Audiospuren als Grundlage der Aufnahmearchitektur.

Ein Track ist ein technisches Aufnahmeergebnis und nicht selbst das fachliche Recording.

Die fachliche Zuordnung zu Production Session und Participants erfolgt im Domain-Modell.

---

# Verantwortungsgrenze im Recording Workflow

Konzeptionell:

```text
Production Session
        |
        v
Recording Domain Object
        |
        v
Recording Workflow
        |
        v
Capture Boundary
        |
        v
Capture Implementation
        |
        v
Recording Artifact
        |
        v
Artifact Registry
        |
        v
Persistence
        |
        v
Artifact Processing
```

Nicht jede Implementierung muss diese Schritte als separate Softwarekomponenten ausführen. Die Verantwortlichkeiten müssen jedoch getrennt bleiben.

---

# Fehler- und Ausfallverhalten

Fehler in einer technischen Komponente dürfen nicht automatisch die fachliche Bedeutung des Recordings zerstören.

Ein fehlgeschlagener Upload bedeutet beispielsweise nicht automatisch, dass das lokale Recording verloren ist.

Recovery- und Konsistenzregeln werden durch die jeweils zuständigen technischen Grenzen definiert.

---

# Interface Boundary

Die Kommunikation zwischen Core und Capture erfolgt über definierte Schnittstellen.

```text
Core

Recording Operation
        |
        v
Capture Boundary
        |
        v
Capture Provider
        |
        v
Audio Backend
```

Der Core kennt keine konkrete Audio-Technologie. Der Capture Provider kennt keine fachliche Implementierung der Production Session.

---

# Technology Independence

Konkrete Audio-Technologien sind nicht Bestandteil dieser Entscheidung.

Technische Entscheidungen dürfen die hier definierten Verantwortungsgrenzen nicht aufheben. Insbesondere darf eine konkrete Technologie keine technischen Audioabhängigkeiten in das fachliche Recording-Modell einführen.

---

# Konsequenzen

## Positive Auswirkungen

* fachliche Recording-Logik bleibt unabhängig von Audio-Technologie
* Capture-Implementierungen können ausgetauscht werden
* Core-Tests benötigen keine Audio-Hardware
* technische Audiofehler bleiben außerhalb der Domain
* Registry und Persistence bleiben unabhängig von Capture
* lokale Aufnahme bleibt unabhängig von Netzwerkverfügbarkeit
* Verantwortungsgrenzen bleiben nachvollziehbar

## Negative Auswirkungen

* zusätzliche Schnittstellen müssen definiert werden
* Recording Workflow und Capture müssen koordiniert werden
* Artifact-Erzeugung benötigt eigene technische Regeln
* Fehlerzustände müssen über mehrere technische Grenzen hinweg behandelt werden

Diese Nachteile werden bewusst akzeptiert.

---

# Betrachtete Alternativen

## Audio Recording Inside the Core

Nicht gewählt. Dies würde technische Audioabhängigkeiten in die fachliche Domäne einführen.

## Client-Owned Recording Without Domain Model

Nicht gewählt. Dies würde fachliche Recording-Zustände und Session-Beziehungen in technische Clients verlagern.

## Capture Provider Owns Artifact Persistence

Nicht gewählt. Capture und Persistence würden dadurch unnötig gekoppelt.

## Recording as Audio File

Nicht gewählt. Eine Audiodatei ist ein technisches Ergebnis und nicht die vollständige fachliche Repräsentation eines Recordings.

---

# Beziehung zu bestehender Architektur

Diese Entscheidung konkretisiert insbesondere die in ADR-001, ADR-002, ADR-015, ADR-018, ADR-019, ADR-029, ADR-033, ADR-034, ADR-035 und ADR-038 definierten Grenzen.

Sie bildet die Grenze zwischen fachlichem Recording und technischer Aufnahme explizit ab.

---

# Status

Diese Entscheidung definiert die grundlegende Architekturgrenze zwischen dem fachlichen Recording innerhalb des NC-PoRe Core und der technischen Audioaufnahme.

---

# English Version ([Deutsche Version oben](#deutsch))

## Context

NC-PoRe was designed to enable distributed podcast production with local recording.

Previous architecture decisions define local recording, the Production Session as the central domain entity, the Core as Domain Authority and the separation of domain and technical responsibilities.

The implementation must keep the following concepts separate:

* domain Recording
* technical audio capture
* Recording Artifact
* local Artifact management and persistence
* processing and synchronization

A Recording is not the same thing as an audio file.

---

# Decision

NC-PoRe separates the domain Recording model from technical capture implementation.

The Core defines the domain Recording. The Capture layer produces technical recording data. This data is represented as a Recording Artifact and passed through defined technical boundaries to registry, persistence and processing components.

```text
Domain

Production Session
        |
        v
Recording
        |
        v
Capture Boundary
        |
        v
Recording Artifact
        |
        +--------------------+
        |                    |
        v                    v
Artifact Registry      Persistence
        |
        v
Artifact Processing
```

Concrete implementations may realize these boundaries differently. The domain responsibilities remain independent.

---

# Recording as a Domain Production Object

A `Recording` is part of the domain production model.

It describes in particular:

* its Production Session association
* domain states
* domain relationships to Participants
* relevant domain metadata
* its domain identity and lifecycle

The Core is responsible for the integrity of this model.

---

# Core Responsibility

The Core is responsible for the domain meaning of a Recording, including:

* Recording lifecycle
* relationship between Recording and Production Session
* relationship between Recording and Participant
* domain validation
* permitted domain state transitions
* domain Recording metadata
* domain events

The Core does not decide how audio data is technically produced or stored.

---

# Core Non-Responsibility

The Core does not contain concrete technical audio capture.

This includes:

* audio hardware control
* microphone access
* platform-specific audio APIs
* concrete audio backends
* capture buffers
* real-time audio processing
* concrete file writes
* filesystem operations
* technical Artifact management

These responsibilities remain outside the domain model.

---

# Capture as a Technical Operation

The Capture component records audio data from a technical audio source.

It is responsible for:

* initializing capture
* capturing the audio stream
* technical buffering
* passing captured data to the Recording workflow
* handling technical capture failures

It does not decide Production Session rules, user roles, domain permissions or domain Recording states.

---

# Capture Boundary

The boundary between domain Recording and technical audio capture is defined by a Capture Boundary.

```text
Core

Recording Lifecycle
        |
        v
Capture Boundary
        |
        v
Capture Implementation
        |
        v
Audio Backend
        |
        v
Audio Source
```

The Core is therefore independent from a concrete audio backend.

---

# Recording Artifact

The result of technical capture is treated as a `Recording Artifact`.

An Artifact represents the technical recording data actually produced and may carry technical file information, recording properties and associations with the session and participant.

The concrete technical management of Artifacts is outside the scope of this ADR.

---

# Separation of Recording and Artifact

```text
Recording

= domain production object

Recording Artifact

= technical representation
  of actually produced recording data
```

A Recording may exist in the domain before a finished Artifact exists.

Conversely, an Artifact does not itself define the domain meaning of a Recording.

The relationship is established through defined domain or technical references.

---

# Local Recording Principle

Recording follows the fundamental principle established by ADR-001 and ADR-029:

```text
Record locally

↓

Secure recording data locally

↓

Complete the recording in the domain

↓

Process the Artifact in a controlled way

↓

Synchronize
```

An active audio recording must not depend on a stable network connection.

A network failure during capture must not automatically terminate the local recording.

---

# Track Model

NC-PoRe uses separate audio tracks as a foundation of the recording architecture.

A track is a technical recording result, not the domain Recording itself.

The domain association with the Production Session and Participants is defined by the domain model.

---

# Responsibility Boundaries in the Recording Workflow

Conceptually:

```text
Production Session
        |
        v
Recording Domain Object
        |
        v
Recording Workflow
        |
        v
Capture Boundary
        |
        v
Capture Implementation
        |
        v
Recording Artifact
        |
        v
Artifact Registry
        |
        v
Persistence
        |
        v
Artifact Processing
```

Not every implementation must realize these steps as separate software components. The responsibilities must nevertheless remain separate.

---

# Failure Handling

Failure in a technical component must not automatically destroy the domain meaning of the Recording.

For example, a failed upload does not automatically mean that the local Recording is lost.

Recovery and consistency rules are defined by the responsible technical boundaries.

---

# Interface Boundary

Communication between Core and Capture uses defined interfaces.

```text
Core

Recording Operation
        |
        v
Capture Boundary
        |
        v
Capture Provider
        |
        v
Audio Backend
```

The Core knows no concrete audio technology. The Capture Provider knows no domain implementation of the Production Session.

---

# Technology Independence

Concrete audio technologies are outside the scope of this decision.

Technical decisions must preserve the responsibility boundaries defined here. In particular, a concrete technology must not introduce technical audio dependencies into the domain Recording model.

---

# Consequences

## Positive Consequences

* domain Recording logic remains independent from audio technology
* Capture implementations can be replaced
* Core tests do not require audio hardware
* technical audio failures remain outside the domain
* Registry and Persistence remain independent from Capture
* local recording remains independent from network availability
* responsibility boundaries remain traceable

## Negative Consequences

* additional interfaces must be defined
* Recording Workflow and Capture must be coordinated
* Artifact creation requires technical rules
* failures must be handled across multiple technical boundaries

These disadvantages are consciously accepted.

---

# Alternatives Considered

## Audio Recording Inside the Core

Rejected. This would introduce technical audio dependencies into the domain.

## Client-Owned Recording Without a Domain Model

Rejected. This would move domain Recording states and session relationships into technical clients.

## Capture Provider Owns Artifact Persistence

Rejected. This would couple Capture and Persistence unnecessarily.

## Recording as Audio File

Rejected. An audio file is a technical result and not the complete domain representation of a Recording.

---

# Relationship to Existing Architecture

This decision concretizes the boundaries established in particular by ADR-001, ADR-002, ADR-015, ADR-018, ADR-019, ADR-029, ADR-033, ADR-034, ADR-035 and ADR-038.

It makes the boundary between domain Recording and technical capture explicit.

---

# Status

This decision defines the fundamental architecture boundary between the domain Recording within the NC-PoRe Core and technical audio capture.
