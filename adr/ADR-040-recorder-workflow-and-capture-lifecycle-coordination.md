# ADR-040 Recorder Workflow and Capture Lifecycle Coordination

* Status: Proposed
* Date: 2026-07-31
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

---

# Kontext

NC-PoRe trennt seit ADR-039 das fachliche Recording-Modell
von der technischen Audio-Capture-Implementierung.

Der Recorder besitzt eine lokale Anwendungsschicht, die den
Aufnahmeablauf eines Clients steuert.

Aktuell existieren:

* Production Session im Core
* Recording als fachliches Produktionsobjekt
* RecordingSession als lokaler Recorder-Zustand
* CaptureProvider als technische Capture-Grenze

Damit entsteht die Frage, welche Komponente für die
Koordination des lokalen Aufnahmeablaufs verantwortlich ist.

Die Architektur muss unterscheiden zwischen:

* fachlichen Zuständen einer Produktion
* lokalem Aufnahmeablauf
* technischer Audioaufnahme

Eine direkte Vermischung dieser Verantwortlichkeiten würde
die Trennung aus ADR-038 und ADR-039 verletzen.

---

# Entscheidung

NC-PoRe verwendet eine lokale Recorder Workflow-Schicht zur
Koordination des Aufnahmeablaufs.

Die Recorder Workflow-Schicht verbindet:

* lokale RecordingSession
* CaptureProvider
* technische Speicherkomponenten

Sie ist jedoch nicht Teil des Core.

Grundstruktur:

```text
Core

Production Session
Recording Entity

        |
        |
        v

Recorder Workflow

        |
        +----------------+
        |                |
        v                v

RecordingSession    CaptureProvider

                         |
                         v

                  Audio Backend
```

---

# Responsibility Separation

## Core

Der Core ist verantwortlich für:

* fachliche Recording-Zustände
* Produktionsregeln
* Beziehungen zwischen Produktionsobjekten

Der Core kennt nicht:

* lokale Hardware
* Mikrofone
* Audio-Backends
* lokale Dateien

---

## Recorder Workflow

Der Recorder Workflow ist verantwortlich für:

* Starten eines lokalen Aufnahmeablaufs
* Koordination von Session und Capture
* Behandlung lokaler technischer Fehler
* Übergabe technischer Ergebnisse

Der Recorder Workflow entscheidet nicht über:

* fachliche Produktionsregeln
* Benutzerrollen
* Domain Lifecycle

---

## Capture Provider

Der Capture Provider ist verantwortlich für:

* Audioquellen
* Datenaufnahme
* technische Audioverarbeitung

Der Capture Provider kennt nicht:

* Production Sessions
* Teilnehmerrollen
* Produktionsregeln

---

# Lifecycle Coordination

Der lokale Ablauf folgt diesem Modell:

```text
Recording Session erzeugen

↓

Capture vorbereiten

↓

Capture starten

↓

Aufnahme aktiv

↓

Capture stoppen

↓

Lokale Daten sichern

↓

Synchronisation vorbereiten
```

Die technische Capture-Lebensdauer wird durch den
Recorder Workflow gesteuert.

---

# Error Handling Boundary

Technische Fehler bleiben innerhalb der Recorder-Schicht.

Beispiele:

* Mikrofon nicht verfügbar
* Audio Backend Fehler
* lokaler Speicherfehler

Diese Fehler sind keine Domain-Regeln.

Der Recorder kann technische Zustände in fachlich
verständliche Ergebnisse übersetzen.

---

# Testing Consequences

Die Trennung ermöglicht:

* Tests des Core ohne Audio-Hardware
* Tests des Recorder Workflow ohne echte Hardware
* Mock-Capture-Provider
* reproduzierbare lokale Abläufe

---

# Alternatives Considered

## Core Controls Audio Capture Directly

Nicht gewählt.

Begründung:

Dies würde technische Abhängigkeiten in die Domain
einführen und die Trennung zwischen Fachlogik und
technischer Umsetzung verletzen.

---

## Capture Provider Controls Entire Recording Workflow

Nicht gewählt.

Begründung:

Der Capture Provider kennt technische Audiooperationen,
aber nicht den lokalen Produktionsablauf.

---

## No Workflow Layer

Nicht gewählt.

Begründung:

Ohne Koordinationsschicht würden Verantwortlichkeiten
zwischen Session, Capture und Storage verteilt.

---

# Relationship to Existing Architecture

Diese Entscheidung erweitert:

* ADR-038 Core Implementation Structure and Module Organization
* ADR-039 Recording Architecture and Capture Boundary

und berücksichtigt:

* ADR-001 Local Recording
* ADR-015 Recorder Software Architecture
* ADR-018 Recorder Data Flow and Processing Pipeline
* ADR-029 Distributed Recording Architecture

---

# Future Considerations

Weitere Entscheidungen werden separat behandelt:

* konkrete CaptureProvider Implementierungen
* Audio Backend Auswahl
* lokale Chunk-Speicherung
* Synchronisationsmechanismen
* Exportabläufe

Diese Entscheidungen erfolgen erst bei konkretem
technischem Bedarf.

---

# Status

Diese Entscheidung definiert die Koordinationsgrenze
zwischen lokalem Recorder Workflow und technischer
Audioaufnahme.

Die konkrete Implementierung erfolgt innerhalb dieser
Architekturgrenzen.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# Context

Since ADR-039, NC-PoRe separates the domain Recording model
from the technical audio capture implementation.

The Recorder contains a local application layer that
coordinates the recording process of a client.

Existing components:

* Production Session in Core
* Recording as domain production object
* RecordingSession as local recorder state
* CaptureProvider as technical capture boundary

The remaining question is which component coordinates the
local recording workflow.

The architecture must distinguish between:

* domain production states
* local recording workflow
* technical audio capture

Mixing these responsibilities would violate the separation
defined by ADR-038 and ADR-039.

---

# Decision

NC-PoRe uses a local Recorder Workflow layer to coordinate
the recording process.

The Recorder Workflow connects:

* local RecordingSession
* CaptureProvider
* technical storage components

It is not part of the Core.

Structure:

```text
Core

Production Session
Recording Entity

        |
        v

Recorder Workflow

        |
        +----------------+
        |                |
        v                v

RecordingSession    CaptureProvider

                         |
                         v

                  Audio Backend
```

---

# Responsibility Separation

## Core

Responsible for:

* recording domain states
* production rules
* relationships between production objects

The Core does not know:

* local hardware
* microphones
* audio backends
* local files

---

## Recorder Workflow

Responsible for:

* starting local recording workflows
* coordinating session and capture
* handling local technical errors
* passing technical results onward

The Recorder Workflow does not decide:

* domain production rules
* user roles
* domain lifecycle

---

## Capture Provider

Responsible for:

* audio sources
* data capture
* technical audio processing

The Capture Provider does not know:

* Production Sessions
* participant roles
* production rules

---

# Lifecycle Coordination

The local workflow follows this model:

```text
Create Recording Session

↓

Prepare Capture

↓

Start Capture

↓

Recording Active

↓

Stop Capture

↓

Store Local Data

↓

Prepare Synchronization
```

The technical capture lifecycle is controlled by the
Recorder Workflow.

---

# Error Handling Boundary

Technical errors remain inside the Recorder layer.

Examples:

* unavailable microphone
* audio backend failure
* local storage failure

These errors are not domain rules.

The Recorder translates technical states into meaningful
workflow results.

---

# Testing Consequences

The separation enables:

* Core tests without audio hardware
* Recorder Workflow tests without real hardware
* mock Capture Providers
* reproducible local workflows

---

# Alternatives Considered

## Core Controls Audio Capture Directly

Rejected.

Reason:

This would introduce technical dependencies into the domain
and violate the separation between domain logic and technical
implementation.

---

## Capture Provider Controls Entire Recording Workflow

Rejected.

Reason:

The Capture Provider knows technical audio operations,
but not the local production workflow.

---

## No Workflow Layer

Rejected.

Reason:

Without a coordination layer, responsibilities would be
distributed between session, capture and storage.

---

# Relationship to Existing Architecture

This decision extends:

* ADR-038 Core Implementation Structure and Module Organization
* ADR-039 Recording Architecture and Capture Boundary

and considers:

* ADR-001 Local Recording
* ADR-015 Recorder Software Architecture
* ADR-018 Recorder Data Flow and Processing Pipeline
* ADR-029 Distributed Recording Architecture

---

# Future Considerations

Further decisions are handled separately:

* concrete CaptureProvider implementations
* audio backend selection
* local chunk storage
* synchronization mechanisms
* export workflows

These decisions are made only when concrete technical
requirements arise.

---

# Status

This decision defines the coordination boundary between
local Recorder Workflow and technical audio capture.

Concrete implementation follows these architectural
boundaries.
