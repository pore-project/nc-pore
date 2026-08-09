# ADR-045 Local Artifact Management

* Status: Accepted
* Date: 2026-08-01
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

---

# Kontext

Mit ADR-042 wurde das Recording Artifact als eigenständiges technisches Modell mit eigenem Lifecycle eingeführt.

Mit ADR-043 wurde die Local Recording Persistence Boundary definiert.

Mit ADR-044 wurde das Persistence Provider Interface als technische Schnittstelle innerhalb dieser Boundary eingeführt.

Die aktuelle Architektur trennt damit:

```text
Recording Artifact

↓

Persistence Provider Interface

↓

Persistence Provider Implementation

↓

Local Storage Backend
```

Bisher ist jedoch noch nicht definiert, welche Verantwortung die lokale Artefaktverwaltung innerhalb dieser Struktur übernimmt.

Die Persistenz eines Artifacts bedeutet nicht nur das Speichern von Daten.

Eine lokale Artefaktverwaltung muss zusätzlich sicherstellen:

* eindeutige technische Identität
* Verwaltung vorhandener Artefakte
* Zuordnung persistierter Daten
* kontrollierten Zugriff auf lokale Artefakte

Diese Verantwortung soll nicht in das Recording Artifact Modell und nicht in den Recorder Workflow verschoben werden.

---

# Entscheidung

NC-PoRe führt eine **Local Artifact Management Layer** ein.

Diese Schicht verwaltet lokale Recording Artifacts innerhalb der Persistence Provider Implementierung.

Die Architektur lautet:

```text
Recorder Workflow

↓

Persistence Provider Interface

↓

Local Artifact Management

↓

Storage Backend
```

Der Recorder Workflow bleibt weiterhin unabhängig von konkreten Speichermechanismen.

---

# Architectural Principle

Local Artifact Management trennt:

* fachliche Erzeugung eines Recording Artifacts
* technische Verwaltung eines gespeicherten Artifacts
* konkrete Speicherung der Daten

Das Recording Artifact beschreibt:

**Was technisch aufgenommen wurde.**

Die Local Artifact Management Layer beschreibt:

**Wie dieses Artifact lokal verwaltet wird.**

---

# Responsibilities

## Local Artifact Management

Verantwortlich für:

* Verwaltung lokaler Recording Artifacts
* Zuordnung zwischen Artifact und Persistenzdaten
* Verwaltung technischer Metadaten
* Vorbereitung zukünftiger Storage- und Synchronisationsmechanismen

Nicht verantwortlich für:

* Audioaufnahme
* fachliche Produktionslogik
* Recording Lifecycle Entscheidungen
* Benutzerinteraktion

---

## Recording Artifact Model

Das Recording Artifact bleibt verantwortlich für:

* technische Identität
* eigenen Lifecycle
* Beschreibung des technischen Aufnahmeergebnisses

Es übernimmt nicht:

* Dateiverwaltung
* Speicherzugriff
* Persistenzlogik

---

## Persistence Provider

Der Persistence Provider bleibt verantwortlich für:

* Bereitstellung der Persistenzoperationen
* Zugriff auf gespeicherte Artefakte
* technische Abstraktion des Backends

Er enthält keine fachliche Interpretation der Artefakte.

---

# Initial Scope

Die erste Implementierung beschränkt sich auf:

```text
create local artifact reference

↓

store artifact metadata

↓

retrieve artifact metadata

↓

remove artifact metadata
```

Weitere Funktionen werden erst eingeführt, wenn konkrete Anforderungen entstehen.

---

# In-Memory Implementation

Die bestehende In-Memory Persistence Implementation wird erweitert, um die Local Artifact Management Layer abzubilden.

Sie bleibt:

* Referenzimplementierung
* Testgrundlage
* Entwicklungsumgebung

Sie ist weiterhin keine endgültige Storage-Lösung.

---

# Technology Independence

Diese ADR definiert keine konkrete Speichertechnologie.

Nicht Bestandteil dieser Entscheidung:

* Dateisystemstruktur
* Datenbankmodell
* Verschlüsselung
* Cloud Storage
* Synchronisationsmechanismen

Diese Entscheidungen erfolgen später durch eigene ADRs.

---

# Consequences

## Positive Consequences

* klare Trennung zwischen Artifact und Speicherung
* Vorbereitung für reale Storage Implementierungen
* bessere Erweiterbarkeit
* geringere Kopplung zwischen Workflow und Infrastruktur
* Grundlage für spätere Synchronisation

---

## Negative Consequences

* zusätzliche technische Schicht
* mehr Komponenten im Recorder-System
* höherer initialer Implementierungsaufwand

Diese Nachteile werden bewusst akzeptiert.

---

# Considered Alternatives

## Persistence Provider manages everything

Nicht gewählt.

Begründung:

Der Provider würde dadurch neben der technischen Speicherung auch Verwaltungslogik übernehmen.

Dies würde die Verantwortlichkeiten vermischen.

---

## Recording Artifact manages local storage

Nicht gewählt.

Begründung:

Das technische Modell würde Infrastrukturverantwortung übernehmen.

Dies widerspricht der bestehenden Architekturtrennung.

---

# Relationship to Existing Architecture

Diese Entscheidung erweitert:

* ADR-041 Local Recording Artifact and Storage Boundary
* ADR-042 Recording Artifact Model and Lifecycle Boundary
* ADR-043 Local Recording Persistence Boundary
* ADR-044 Persistence Provider Interface

Sie definiert die Verantwortlichkeit für lokale Artefaktverwaltung innerhalb der Persistence Architektur.

---

# Future Decisions

Spätere ADRs behandeln:

* konkrete Storage Provider
* Dateisystem-basierte Speicherung
* Datenbank-basierte Speicherung
* Artefaktversionierung
* Synchronisationsstatus
* Wiederherstellungsmechanismen

---

# Status

Diese Entscheidung definiert Local Artifact Management als technische Verwaltungsschicht zwischen Persistence Provider und Storage Backend.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# Context

ADR-042 introduced the Recording Artifact as an independent technical model with its own lifecycle.

ADR-043 defined the Local Recording Persistence Boundary.

ADR-044 introduced the Persistence Provider Interface as the technical contract inside this boundary.

The current architecture separates:

```text
Recording Artifact

↓

Persistence Provider Interface

↓

Persistence Provider Implementation

↓

Local Storage Backend
```

However, responsibility for managing locally persisted artifacts has not yet been defined.

Persistence is not only storing data.

Local artifact management also requires:

* technical identity handling
* management of existing artifacts
* mapping persisted data
* controlled access to local artifacts

This responsibility must not be moved into the Recording Artifact model or the Recorder Workflow.

---

# Decision

NC-PoRe introduces a **Local Artifact Management Layer**.

This layer manages local Recording Artifacts inside the Persistence Provider implementation.

The architecture becomes:

```text
Recorder Workflow

↓

Persistence Provider Interface

↓

Local Artifact Management

↓

Storage Backend
```

The Recorder Workflow remains independent from concrete storage mechanisms.

---

# Architectural Principle

Local Artifact Management separates:

* domain creation of a Recording Artifact
* technical management of a persisted Artifact
* concrete storage implementation

The Recording Artifact describes:

**What was technically recorded.**

The Local Artifact Management Layer describes:

**How this Artifact is managed locally.**

---

# Responsibilities

## Local Artifact Management

Responsible for:

* managing local Recording Artifacts
* mapping artifacts to persistence data
* managing technical metadata
* preparing future storage and synchronization mechanisms

Not responsible for:

* audio recording
* production domain logic
* Recording Lifecycle decisions
* user interaction

---

## Recording Artifact Model

The Recording Artifact remains responsible for:

* technical identity
* own lifecycle
* description of the technical recording result

It does not manage:

* file handling
* storage access
* persistence logic

---

## Persistence Provider

The Persistence Provider remains responsible for:

* providing persistence operations
* accessing stored artifacts
* abstracting technical backend details

It does not contain domain interpretation.

---

# Initial Scope

The first implementation is limited to:

```text
create local artifact reference

↓

store artifact metadata

↓

retrieve artifact metadata

↓

remove artifact metadata
```

Additional functions are introduced only when concrete requirements exist.

---

# Status

This decision defines Local Artifact Management as the technical management layer between Persistence Provider and Storage Backend.
