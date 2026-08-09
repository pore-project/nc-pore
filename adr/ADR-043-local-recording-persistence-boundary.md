# ADR-043 Local Recording Persistence Boundary

* Status: Accepted
* Date: 2026-08-01
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

---

# Kontext

Mit ADR-039 wurde die Grenze zwischen fachlichem Recording-Modell und technischer Audioaufnahme definiert.

ADR-040 beschreibt die Koordination des lokalen Recorder-Workflows.

ADR-041 definiert die Trennung zwischen lokalen Aufnahmedaten und deren Speicherung.

ADR-042 führt das Recording Artifact als eigenständiges technisches Modell mit eigenem Lifecycle ein.

Damit existiert eine technische Einheit zwischen:

```text
Capture

↓

Recording Artifact

↓

Storage

↓

Synchronization

↓

Export
```

Es fehlt jedoch noch die Entscheidung, wie Recording Artifacts lokal dauerhaft erhalten werden.

Die lokale Persistenz muss unabhängig bleiben von:

* konkreten Speichertechnologien
* Dateiformaten
* Datenbanken
* Synchronisationsmechanismen

Die Persistenz ist eine technische Verantwortung des Recorders und darf nicht Teil der fachlichen Domänenlogik werden.

---

# Entscheidung

NC-PoRe führt eine **Local Recording Persistence Boundary** ein.

Recording Artifacts werden nicht direkt durch den Recorder Workflow gespeichert.

Stattdessen erfolgt die Speicherung über eine technische Persistenzgrenze.

Die Architektur lautet:

```text
Recording Artifact

↓

Persistence Boundary

↓

Local Persistence Provider

↓

Stored Local Artifact
```

Die konkrete Implementierung der lokalen Speicherung bleibt austauschbar.

---

# Architectural Principle

Die Persistence Boundary trennt:

* das technische Aufnahmeergebnis
* die Verwaltung dieses Ergebnisses
* die konkrete Speicherung

Ein Recording Artifact beschreibt, **was** gespeichert werden soll.

Die Persistence Boundary beschreibt, **wie** dieses Artefakt lokal erhalten wird.

---

# Verantwortlichkeiten

## Recorder Workflow

Der Recorder Workflow ist verantwortlich für:

* Erzeugung von Recording Artifacts
* Steuerung des Aufnahmeablaufs
* Übergabe von Artifacts an die Persistenzschicht

Der Recorder Workflow ist nicht verantwortlich für:

* Speicherorte
* Dateisystemdetails
* konkrete Speichertechnologien

---

## Recording Artifact

Das Recording Artifact ist verantwortlich für:

* technische Identität
* technische Metadaten
* Lifecycle-Zustand
* Referenz zur Recording Session

Das Recording Artifact ist nicht verantwortlich für:

* Speicherung
* Wiederherstellung
* Infrastrukturzugriff

---

## Persistence Layer

Der Persistence Layer ist verantwortlich für:

* lokale Speicherung von Recording Artifacts
* Laden gespeicherter Artifacts
* technische Verwaltung persistierter Daten

Der Persistence Layer ist nicht verantwortlich für:

* fachliche Produktionsregeln
* Session Lifecycle
* Workflow-Entscheidungen

---

# Lifecycle Integration

Die lokale Persistenz erweitert den technischen Artifact Lifecycle.

Beispiel:

```text
Created

↓

Capturing

↓

Available

↓

Stored

↓

Synchronized

↓

Archived
```

Der Zustand `Stored` beschreibt den technischen Persistenzzustand.

Er verändert nicht automatisch den fachlichen Zustand einer Production Session oder eines Recordings.

---

# Local-First Prinzip

Die Persistence Strategy folgt dem zentralen NC-PoRe Prinzip:

> Lokal aufnehmen. Danach synchronisieren.

Eine Netzwerkverbindung ist keine Voraussetzung für:

* Erstellung von Recording Artifacts
* lokale Speicherung
* Wiederaufnahme nach Unterbrechungen

---

# Technology Independence

Diese Entscheidung legt keine konkrete Speichertechnologie fest.

Nicht Bestandteil dieser ADR sind:

* SQLite
* Dateiystemstrukturen
* Datenbanken
* Cloud Storage
* Verschlüsselungsverfahren

Diese Entscheidungen werden durch spätere ADRs getroffen.

---

# Konsequenzen

## Positive Konsequenzen

* klare Trennung zwischen Artifact und Speicherung
* austauschbare Storage Implementierungen
* bessere Testbarkeit
* Vorbereitung für Synchronisation
* klare technische Verantwortlichkeiten

---

## Negative Konsequenzen

* zusätzliche technische Abstraktion
* zusätzliche Schnittstelle
* höherer Implementierungsaufwand

Diese Nachteile werden bewusst akzeptiert.

---

# Betrachtete Alternativen

## Direct Storage in Recorder Workflow

Nicht gewählt.

Begründung:

Der Workflow würde direkt von einer Speicherimplementierung abhängen.

Dies würde die technische Kopplung erhöhen.

---

## Storage-specific Recording Artifacts

Nicht gewählt.

Begründung:

Das technische Aufnahmeergebnis würde von einer konkreten Speichertechnologie abhängig werden.

Dies widerspricht den Architekturprinzipien von NC-PoRe.

---

# Beziehung zu bestehender Architektur

Diese Entscheidung erweitert:

* ADR-039 Recording Architecture and Capture Boundary
* ADR-040 Recorder Workflow and Capture Lifecycle Coordination
* ADR-041 Local Recording Artifact and Storage Boundary
* ADR-042 Recording Artifact Model and Lifecycle Boundary

Sie definiert die technische Grenze zwischen Recording Artifact und lokaler Speicherung.

---

# Zukünftige Entscheidungen

Spätere Entscheidungen werden unter anderem behandeln:

* konkrete Persistence Implementation
* lokale Verzeichnisstruktur
* Metadatenformat
* Wiederherstellungsmechanismen
* Synchronisationsvorbereitung

Diese Entscheidungen erfolgen unabhängig von dieser Architekturentscheidung.

---

# Status

Diese Entscheidung definiert die Local Recording Persistence Boundary als technische Grundlage für die dauerhafte lokale Verwaltung von Recording Artifacts.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# Context

ADR-039 defined the boundary between the domain Recording model and technical audio capture.

ADR-040 describes coordination of the local recorder workflow.

ADR-041 defines the separation between local recording data and storage.

ADR-042 introduced the Recording Artifact as an independent technical model with its own lifecycle.

This creates a technical unit between:

```
Capture

↓

Recording Artifact

↓

Storage

↓

Synchronization

↓

Export
```

However, one architectural decision is still missing:

How Recording Artifacts are persisted locally.

Local persistence must remain independent from:

* concrete storage technologies
* file formats
* databases
* synchronization mechanisms

Persistence is a technical responsibility of the recorder architecture and must not become part of domain logic.

---

# Decision

NC-PoRe introduces a **Local Recording Persistence Boundary**.

Recording Artifacts are not stored directly by the Recorder Workflow.

Instead, storage is handled through a dedicated technical persistence boundary.

The architecture is:

```
Recording Artifact

↓

Persistence Boundary

↓

Local Persistence Provider

↓

Stored Local Artifact
```

The concrete local storage implementation remains replaceable.

---

# Architectural Principle

The Persistence Boundary separates:

* the technical recording result
* management of that result
* concrete storage implementation

A Recording Artifact describes **what** needs to be stored.

The Persistence Boundary describes **how** the artifact is preserved locally.

---

# Responsibilities

## Recorder Workflow

The Recorder Workflow is responsible for:

* creating Recording Artifacts
* controlling the recording workflow
* passing Artifacts to the persistence layer

The Recorder Workflow is not responsible for:

* storage locations
* filesystem details
* concrete storage technologies

---

## Recording Artifact

The Recording Artifact is responsible for:

* technical identity
* technical metadata
* lifecycle state
* reference to the Recording Session

The Recording Artifact is not responsible for:

* storage
* restoration
* infrastructure access

---

## Persistence Layer

The Persistence Layer is responsible for:

* local storage of Recording Artifacts
* loading stored Artifacts
* technical management of persisted data

The Persistence Layer is not responsible for:

* production domain rules
* Session Lifecycle
* workflow decisions

---

# Lifecycle Integration

Local persistence extends the technical Artifact Lifecycle.

Example:

```
Created

↓

Capturing

↓

Available

↓

Stored

↓

Synchronized

↓

Archived
```

The state `Stored` describes the technical persistence state.

It does not automatically change the domain state of a Production Session or Recording.

---

# Local-First Principle

The Persistence Strategy follows the central NC-PoRe principle:

> Record locally. Synchronize afterwards.

A network connection is not required for:

* creation of Recording Artifacts
* local storage
* recovery after interruptions

---

# Technology Independence

This decision does not define a specific storage technology.

The following are explicitly not part of this ADR:

* SQLite
* filesystem structures
* databases
* cloud storage
* encryption mechanisms

These decisions will be defined through later ADRs.

---

# Consequences

## Positive Consequences

* clear separation between Artifact and storage
* replaceable storage implementations
* improved testability
* preparation for synchronization
* clear technical responsibilities

---

## Negative Consequences

* additional technical abstraction
* additional interface
* increased implementation effort

These disadvantages are consciously accepted.

---

# Alternatives Considered

## Direct Storage in Recorder Workflow

Rejected.

Reason:

The workflow would directly depend on a storage implementation.

This would increase technical coupling.

---

## Storage-specific Recording Artifacts

Rejected.

Reason:

The technical recording result would become dependent on a concrete storage technology.

This contradicts NC-PoRe architecture principles.

---

# Relationship to Existing Architecture

This decision extends:

* ADR-039 Recording Architecture and Capture Boundary
* ADR-040 Recorder Workflow and Capture Lifecycle Coordination
* ADR-041 Local Recording Artifact and Storage Boundary
* ADR-042 Recording Artifact Model and Lifecycle Boundary

It defines the technical boundary between Recording Artifact and local storage.

---

# Future Decisions

Future decisions will address topics including:

* concrete Persistence Implementation
* local directory structure
* metadata format
* recovery mechanisms
* synchronization preparation

These decisions will be made independently from this architecture decision.

---

# Status

This decision defines the Local Recording Persistence Boundary as the technical foundation for durable local management of Recording Artifacts.

