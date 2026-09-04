# ADR-041 Local Recording Artifact and Storage Boundary

* Status: Proposed
* Date: 2026-07-31
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

---

# Kontext

NC-PoRe trennt zwischen fachlicher Produktionslogik und technischen
Aufnahmeprozessen.

Die bisherigen Architekturentscheidungen definieren:

* die Production Session als zentrale fachliche Einheit
* den Core als Autorität für Geschäftslogik
* die Trennung zwischen Domain und technischen Komponenten
* lokale Aufnahme als zentrales Prinzip
* die Capture Boundary zwischen Recorder Workflow und Audio-Technologie

Mit ADR-039 wurde festgelegt:

* Audio Capture ist eine technische Operation.
* Der Core enthält keine Audio-Implementierung.
* Recorder-Komponenten kapseln technische Aufnahmeprozesse.

Mit ADR-040 wurde zusätzlich definiert:

* Der Recorder Workflow koordiniert lokale Aufnahmeabläufe.
* RecordingSession beschreibt den lokalen Aufnahmeprozess.

Damit entsteht die nächste Architekturfrage:

Wie werden die während einer lokalen Aufnahme entstehenden technischen
Ergebnisse behandelt?

Während einer Aufnahme entstehen technische Artefakte:

* Audiodaten
* einzelne Tracks
* Aufnahmemetadaten
* lokale Statusinformationen

Diese Artefakte gehören nicht zur fachlichen Domain.

Ihre Verwaltung benötigt jedoch eine klare technische Grenze.

---

# Entscheidung

NC-PoRe führt eine klare Trennung zwischen:

* fachlichem Recording-Modell
* lokalem Recording Artifact
* technischer Speicherung

ein.

Ein **Recording Artifact** ist das technische Ergebnis einer lokalen
Recorder-Aufnahme.

Der Recorder ist verantwortlich für:

* Erzeugung lokaler Artefakte
* Verwaltung des lokalen Aufnahmezustands
* Zuordnung technischer Daten zu einer RecordingSession
* Vorbereitung für spätere Verarbeitung oder Synchronisation

Der Core bleibt unabhängig von diesen technischen Artefakten.

---

# Architectural Principle

Ein Recording Artifact ist ein technisches Produktionsartefakt.

Es ist nicht identisch mit dem fachlichen Recording-Objekt.

```text
Core Domain

Production Session

        |
        |
        v

Recording Entity


Recorder Application

        |
        |
        v

Recording Session

        |
        |
        v

Recording Artifact

        |
        |
        v

Local Storage
```

Die Domain beschreibt:

* dass ein Recording existiert
* welchen fachlichen Zustand es besitzt
* zu welcher Production Session es gehört

Der Recorder beschreibt:

* welche technischen Daten lokal entstanden sind
* wie diese Daten verwaltet werden
* wie sie für weitere Verarbeitung vorbereitet werden

---

# Recorder Storage Responsibility

Die Recorder-Schicht ist verantwortlich für:

* lokale Speicherung während und nach der Aufnahme
* Verwaltung technischer Aufnahmeartefakte
* Verwaltung lokaler Metadaten
* Sicherstellung der lokalen Verfügbarkeit

Die Recorder-Schicht entscheidet nicht über:

* Production Session Regeln
* Benutzerrechte
* fachliche Produktionszustände

---

# Local Recording Principle

Die lokale Speicherung folgt weiterhin dem Grundsatz:

```text
Lokal aufnehmen

↓

Lokale Artefakte erzeugen

↓

Daten sichern

↓

Später synchronisieren
```

Eine laufende Aufnahme darf nicht von einer Netzwerkverbindung
abhängig sein.

Lokale Speicherung ist daher ein Bestandteil der Recorder-Architektur.

---

# Artifact Contents

Ein Recording Artifact kann technische Informationen enthalten:

* Audiodaten
* einzelne Audiospuren
* technische Metadaten
* Aufnahmeinformationen
* lokale Zustandsinformationen

Die konkrete Struktur wird durch spätere Entscheidungen definiert.

Diese Entscheidung legt bewusst nicht fest:

* konkrete Dateiformate
* konkrete Container
* konkrete Verzeichnisstrukturen
* konkrete Datenbanken

---

# Storage Boundary

Die Storage-Komponente stellt eine technische Grenze dar.

```text
Recorder Workflow

        |
        |
        v

Recording Artifact Interface

        |
        |
        v

Storage Implementation

        |
        |
        v

Filesystem / Database / Object Storage
```

Der Workflow kennt keine konkrete Speichertechnologie.

Die Storage-Schicht kann später unterschiedliche technische
Implementierungen verwenden.

---

# Synchronization Boundary

Lokale Speicherung und Synchronisation werden getrennt behandelt.

Ein lokales Recording Artifact kann später synchronisiert werden.

Die Synchronisationslogik entscheidet jedoch:

* welche Daten übertragen werden
* wann Daten übertragen werden
* wie Konflikte behandelt werden

Diese Fragen werden durch spätere Architekturentscheidungen behandelt.

---

# Technology Independence

Diese Entscheidung trifft keine Auswahl für:

* Dateisysteme
* Datenbanken
* Cloud Storage
* Containerformate
* Audioformate

Technische Entscheidungen erfolgen separat, sobald ein konkreter Bedarf
besteht.

---

# Consequences

## Positive Consequences

* lokale Aufnahme bleibt unabhängig vom Netzwerk
* Core bleibt frei von technischen Speicherabhängigkeiten
* Storage-Implementierungen können ausgetauscht werden
* Synchronisation kann unabhängig entwickelt werden
* technische Artefakte bleiben klar von Domainobjekten getrennt

---

## Negative Consequences

* zusätzliche Abstraktionsschicht notwendig
* technische Implementierung benötigt zusätzliche Schnittstellen
* vollständiger Workflow entsteht erst über mehrere Architekturentscheidungen

Diese Nachteile werden bewusst akzeptiert.

Die klare Trennung verbessert langfristig Wartbarkeit,
Testbarkeit und Erweiterbarkeit.

---

# Alternatives Considered

## Recording Files Directly Inside Core

Nicht gewählt.

Begründung:

Dies würde technische Speicher- und Dateikonzepte in die fachliche
Domain einführen.

---

## Client-Specific Storage Without Shared Boundary

Nicht gewählt.

Begründung:

Ohne gemeinsame Storage Boundary würden unterschiedliche Clients
unterschiedliche Datenmodelle und Abläufe entwickeln.

---

# Relationship to Existing Architecture

Diese Entscheidung erweitert:

* ADR-001 Local Recording
* ADR-002 Audio Format and Track Concept
* ADR-003 Local Chunk Storage
* ADR-007 Open Formats and Interoperability
* ADR-018 Recorder Data Flow and Processing Pipeline
* ADR-029 Distributed Recording Architecture
* ADR-036 Persistence Boundary and Storage Strategy
* ADR-039 Recording Architecture and Capture Boundary
* ADR-040 Recorder Workflow and Capture Lifecycle Coordination

Sie definiert die Grenze zwischen lokalem Aufnahmeprozess und
technischer Speicherung.

---

# Future Considerations

Weitere Entscheidungen behandeln:

* konkrete Storage Implementation
* Chunk Storage
* Track Storage
* Metadata Format
* Synchronisation
* Export

Diese Entscheidungen erfolgen erst bei konkretem technischem Bedarf.

---

# Status

Diese Entscheidung definiert die grundlegende Storage Boundary für
lokale Recording Artefakte innerhalb von NC-PoRe.

Die konkrete technische Speicherung wird durch spätere Entscheidungen
festgelegt.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# Context

NC-PoRe separates production domain logic from technical recording
processes.

Previous architecture decisions define:

* Production Session as the central domain entity
* Core as authority for domain logic
* separation between domain and technical components
* local recording as a central principle
* capture boundary between recorder workflow and audio technology

ADR-039 established:

* Audio capture is a technical operation.
* The Core does not contain audio implementation details.
* Recorder components encapsulate technical recording processes.

ADR-040 established:

* Recorder Workflow coordinates local recording processes.
* RecordingSession represents the local recording workflow.

The next architectural question is:

How are technical results created during local recording handled?

During recording, technical artifacts are created:

* audio data
* tracks
* recording metadata
* local status information

These artifacts do not belong to the domain model.

They require a clearly defined technical boundary.

---

# Decision

NC-PoRe separates:

* Recording domain model
* local Recording Artifact
* technical storage

A Recording Artifact is the technical result of a local recorder
operation.

The Recorder is responsible for:

* creating local artifacts
* managing local recording state
* associating technical data with RecordingSession
* preparing data for later processing or synchronization

The Core remains independent from these technical artifacts.

---

# Status

This decision defines the storage boundary for local recording artifacts
within NC-PoRe.

Concrete storage technologies will be defined through later decisions.
