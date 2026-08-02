# Local Recording Persistence Foundation

* Date: 2026-08-01
* Milestone Type: Persistence Foundation
* Status: Completed

---

# Deutsch ([English version below](#english-version))

---

# Zweck

Dieser Milestone dokumentiert die technische Grundlage für die lokale Persistenz von Recording Artifacts in NC-PoRe.

Nach der Definition des Recording Artifact Modells und seiner technischen Grenzen wurde die nächste notwendige Schicht umgesetzt:

die zuverlässige Verwaltung und Speicherung technischer Aufnahmeartefakte.

Ziel dieses Schrittes war nicht die Einführung einer konkreten Speichertechnologie, sondern die Definition einer austauschbaren Persistenzarchitektur.

---

# Erreichte Ergebnisse

## Local Recording Persistence Boundary

Definiert:

- technische Grenze zwischen Recording Artifacts und lokaler Speicherung
- Trennung zwischen Artifact-Erzeugung und Persistenz
- unabhängige technische Schnittstelle zur Speicherung

Die Erstellung eines Recording Artifacts und seine Speicherung bleiben getrennte Verantwortlichkeiten.

---

## Persistence Provider Interface

Eingeführt:

- definierter Provider-Vertrag für Persistenzoperationen
- Abstraktion konkreter Speicherimplementierungen
- austauschbare Provider-Architektur

Das Recorder-System kennt keine konkrete Speichertechnologie.

---

## In-Memory Persistence Provider

Implementiert:

- Referenzimplementierung für Entwicklung und Tests
- Validierung der Persistence Boundary
- technische Grundlage für zukünftige Storage Provider

Die In-Memory-Implementierung ist keine endgültige Speicherlösung.

---

## Persistence Operations

Die erste technische Schnittstelle unterstützt:

- Speichern von Recording Artifacts
- Laden von Recording Artifacts
- Auflisten gespeicherter Artifacts
- Entfernen von Artifacts

Weitere Operationen werden erst bei konkretem Bedarf ergänzt.

---

# Architekturprinzipien

Mit diesem Milestone wurden folgende Prinzipien umgesetzt:

- Speicherung bleibt von der Recorder-Logik getrennt
- konkrete Storage-Technologien bleiben austauschbar
- Persistenz wird über technische Grenzen abstrahiert
- Tests können ohne externe Speicherabhängigkeiten durchgeführt werden
- zukünftige Storage-Implementierungen können integriert werden

---

# Validierung

Implementiert und geprüft:

```text
recorder tests: 13 passed
```

Die Tests prüfen unter anderem:

- Speicherung von Artifacts
- Laden von Artifacts
- Auflisten von Artifacts
- Entfernen von Artifacts

---

# Relevante ADRs

- ADR-041 Local Recording Artifact and Storage Boundary
- ADR-042 Recording Artifact Model and Lifecycle Boundary
- ADR-043 Local Recording Persistence Boundary
- ADR-044 Persistence Provider Interface

---

# Bedeutung für die weitere Entwicklung

Mit diesem Milestone ist die technische Kette bis zur lokalen Persistenz vorbereitet:

```text
Recorder Workflow

↓

Recording Artifact

↓

Persistence Provider Interface

↓

Persistence Provider Implementation

↓

Local Storage
```

Konkrete Storage-Technologien und Synchronisationsmechanismen bleiben nachfolgende Entscheidungen.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# Purpose

This milestone documents the technical foundation for local persistence of Recording Artifacts in NC-PoRe.

After defining the Recording Artifact model and its technical boundaries, the next required layer was implemented:

reliable management and persistence of technical recording artifacts.

The goal was not introducing a specific storage technology, but defining an interchangeable persistence architecture.

---

# Achieved Results

## Local Recording Persistence Boundary

Defined:

- technical boundary between Recording Artifacts and local storage
- separation between artifact creation and persistence
- independent technical interface for storage handling

Creation of a Recording Artifact and persistence remain separate responsibilities.

---

## Persistence Provider Interface

Introduced:

- defined provider contract for persistence operations
- abstraction of concrete storage implementations
- replaceable provider architecture

The Recorder system does not know any concrete storage technology.

---

## In-Memory Persistence Provider

Implemented:

- reference implementation for development and testing
- validation of the Persistence Boundary
- technical foundation for future storage providers

The In-Memory implementation is not a final storage solution.

---

## Persistence Operations

The initial technical interface supports:

- storing Recording Artifacts
- loading Recording Artifacts
- listing stored Artifacts
- removing Artifacts

Additional operations will only be introduced when concrete requirements exist.

---

# Architecture Principles

This milestone implements the following principles:

- storage remains separated from Recorder logic
- concrete storage technologies remain replaceable
- persistence is abstracted through technical boundaries
- tests can run without external storage dependencies
- future storage implementations can be integrated

---

# Validation

Implemented and verified:

```text
recorder tests: 13 passed
```

The tests verify among other things:

- storing Artifacts
- loading Artifacts
- listing Artifacts
- removing Artifacts

---

# Related ADRs

- ADR-041 Local Recording Artifact and Storage Boundary
- ADR-042 Recording Artifact Model and Lifecycle Boundary
- ADR-043 Local Recording Persistence Boundary
- ADR-044 Persistence Provider Interface

---

# Importance for Further Development

With this milestone, the technical chain up to local persistence is prepared:

```text
Recorder Workflow

↓

Recording Artifact

↓

Persistence Provider Interface

↓

Persistence Provider Implementation

↓

Local Storage
```

Concrete storage technologies and synchronization mechanisms remain future decisions.
