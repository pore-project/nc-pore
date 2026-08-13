# Deutsch ([English version below](#english-version))

# ADR-020: Metadata Data Model

## Status

Accepted

## Date

2026-07-23

---

# Context

NC-PoRe behandelt eine Aufnahme nicht nur als
Audiodatei, sondern als verwaltbare Einheit.

ADR-019 definiert die Recording Session als
zentrale Einheit einer Aufnahme.

Eine Session benötigt zusätzliche Informationen,
um nachvollziehbar gespeichert, verwaltet und
später verarbeitet werden zu können.

Diese Informationen werden als Metadaten geführt.

---

# Decision

NC-PoRe verwendet ein separates Metadata Model.

Metadaten werden unabhängig von den eigentlichen
Audiodaten behandelt.

Das Metadata Model beschreibt technische,
organisatorische und optionale beschreibende
Informationen einer Aufnahme.

---

# Metadata Categories

Metadaten werden in mehrere Bereiche unterteilt.

---

# Technical Metadata

Technische Informationen über die Aufnahme.

Beispiele:

- Audioformat
- Sample Rate
- Anzahl der Kanäle
- Bit-Tiefe
- Aufnahmedauer
- verwendete Recorder-Version

Diese Informationen werden möglichst automatisch
erzeugt.

---

# Session Metadata

Informationen zur Verwaltung der Aufnahme.

Beispiele:

- Session ID
- Erstellungszeitpunkt
- Startzeit
- Endzeit
- Status

Diese Informationen werden durch das
Session Management verwaltet.

---

# User Metadata

Informationen, die durch Benutzer oder
Anwendungen ergänzt werden können.

Beispiele:

- Titel
- Beschreibung
- Tags
- Notizen

Diese Informationen sind optional.

---

# System Metadata

Informationen über die technische Umgebung.

Beispiele:

- Betriebssystem
- verwendetes Gerät
- Anwendungsversion

Diese Informationen unterstützen Diagnose
und Support.

---

# Data Model Concept

Die grundsätzliche Struktur:

```text
RecordingSession

    |
    |
    +-- Metadata

          |
          +-- Technical Metadata

          +-- Session Metadata

          +-- User Metadata

          +-- System Metadata
```

---

# Decision Principles

Das Metadata Model folgt diesen Prinzipien:

- klare Trennung von Audio und Beschreibung
- Erweiterbarkeit ohne Änderung vorhandener Daten
- optionale Felder ermöglichen zukünftige Funktionen
- maschinenlesbare Speicherung

---

# Alternatives Considered

## Metadata Only Inside Audio Files

Metadaten werden ausschließlich in Audiodateien
gespeichert.

Verworfen wegen:

- Abhängigkeit von Dateiformaten
- eingeschränkter Erweiterbarkeit
- schlechter Trennung von Daten und Beschreibung

---

## No Separate Metadata Model

Alle Informationen werden direkt in einzelnen
Komponenten verwaltet.

Verworfen wegen:

- fehlender Übersichtlichkeit
- schwieriger Synchronisation
- hoher Kopplung

---

# Consequences

## Positive Consequences

- klare Datenstruktur
- bessere Suche und Verwaltung
- Grundlage für Nextcloud-Integration
- einfache Erweiterbarkeit

---

## Negative Consequences

- zusätzliche Datenhaltung
- Modell muss langfristig gepflegt werden

---

# Future Considerations

Spätere Entscheidungen müssen definieren:

- konkrete Rust-Strukturen
- Serialisierungsformat
- Datenbank- oder Dateispeicherung
- Synchronisationsmodell
- Datenschutzaspekte

---

# Final Principle

Metadaten machen aus einer Audiodatei eine
nachvollziehbare und verwaltbare Aufnahme.

NC-PoRe behandelt Informationen über eine Aufnahme
als gleichwertigen Bestandteil der Architektur.

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-020: Metadata Data Model

## Status

Accepted

## Date

2026-07-23

---

# Context

NC-PoRe treats a recording not only as an audio file, but as a manageable unit.

ADR-019 defines the Recording Session as the central unit of a recording.

A session requires additional information so that it can be stored, managed, and processed later in a traceable manner.

This information is maintained as metadata.

---

# Decision

NC-PoRe uses a separate Metadata Model.

Metadata is handled independently of the actual audio data.

The Metadata Model describes technical, organizational, and optional descriptive information about a recording.

---

# Metadata Categories

Metadata is divided into several areas.

---

# Technical Metadata

Technical information about the recording.

Examples:

- audio format
- sample rate
- number of channels
- bit depth
- recording duration
- Recorder version used

This information is generated automatically wherever possible.

---

# Session Metadata

Information used to manage the recording.

Examples:

- session ID
- creation time
- start time
- end time
- status

This information is managed by Session Management.

---

# User Metadata

Information that may be added by users or applications.

Examples:

- title
- description
- tags
- notes

This information is optional.

---

# System Metadata

Information about the technical environment.

Examples:

- operating system
- device used
- application version

This information supports diagnosis and support.

---

# Data Model Concept

The basic structure:

```text
RecordingSession

    |
    |
    +-- Metadata

          |
          +-- Technical Metadata

          +-- Session Metadata

          +-- User Metadata

          +-- System Metadata
```

---

# Decision Principles

The Metadata Model follows these principles:

- clear separation of audio and description
- extensibility without changing existing data
- optional fields allow future functions
- machine-readable storage

---

# Alternatives Considered

## Metadata Only Inside Audio Files

Metadata is stored exclusively in audio files.

Rejected because of:

- dependency on file formats
- limited extensibility
- poor separation of data and description

---

## No Separate Metadata Model

All information is managed directly by individual components.

Rejected because of:

- lack of clarity
- difficult synchronization
- high coupling

---

# Consequences

## Positive Consequences

- clear data structure
- better search and management
- foundation for Nextcloud integration
- simple extensibility

---

## Negative Consequences

- additional data storage
- model requires long-term maintenance

---

# Future Considerations

Later decisions must define:

- concrete Rust structures
- serialization format
- database or file storage
- synchronization model
- data protection aspects

---

# Final Principle

Metadata turns an audio file into a traceable and manageable recording.

NC-PoRe treats information about a recording as an equal part of the architecture.
