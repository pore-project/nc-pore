# Deutsch ([English version below](#english-version))

# ADR-021: Internal Data Structures and Serialization Format

## Status

Accepted

## Date

2026-07-23

---

# Context

Die bisherigen Architekturentscheidungen definieren:

- die Recorder-Struktur
- den Datenfluss
- das Konzept einer Recording Session
- das Metadata Model

Für die Implementierung benötigt NC-PoRe nun konkrete
interne Datenstrukturen.

Die Datenmodelle müssen:

- in Rust abbildbar sein
- langfristig erweiterbar bleiben
- zwischen Komponenten austauschbar sein
- für Speicherung und Synchronisation geeignet sein

---

# Decision

NC-PoRe verwendet klar definierte interne Datenmodelle
für die zentralen Domänenobjekte.

Die Datenstrukturen werden zunächst als Rust-Strukturen
modelliert.

Die wichtigsten Objekte sind:

```text
RecordingSession

Metadata

AudioReference

SessionStatus
```

Die internen Modelle werden von der Audio-Hardware,
der Speicherung und dem Export getrennt gehalten.

---

# Core Data Structures

## RecordingSession

Die RecordingSession repräsentiert eine vollständige
Aufnahmeeinheit.

Konzeptionell:

```text
RecordingSession

- id
- status
- created_at
- started_at
- stopped_at
- audio_reference
- metadata
```

---

## SessionStatus

Der Status beschreibt den Lebenszyklus einer Aufnahme.

Beispiele:

```text
Created

Recording

Stopped

Stored

Exported

Failed
```

---

## AudioReference

Die AudioReference beschreibt die Verbindung zu den
eigentlichen Audiodaten.

Sie enthält nicht zwingend die Audiodaten selbst.

Beispiele:

```text
AudioReference

- file_path
- format
- duration
- size
```

---

## Metadata

Das Metadata Model enthält beschreibende Informationen
zur Aufnahme.

Siehe ADR-020.

---

# Serialization Format

NC-PoRe verwendet zunächst ein menschenlesbares
Serialisierungsformat.

Die bevorzugte erste Umsetzung ist:

```text
JSON
```

Begründung:

- weit verbreitet
- einfach zu testen
- gut mit anderen Systemen austauschbar
- geeignet für spätere Synchronisation

---

# Alternatives Considered

## Binary Formats

Beispiele:

- MessagePack
- eigene Binärformate

Verworfen als erste Lösung wegen:

- schlechter Lesbarkeit
- höherer Komplexität
- schwieriger Diagnose

---

## Database First

Direkte Speicherung aller Daten in einer Datenbank.

Verworfen als erste Lösung wegen:

- unnötiger Komplexität
- erschwerter Portabilität
- fehlendem Bedarf im frühen Entwicklungsstadium

---

# Consequences

## Positive Consequences

- klare Trennung zwischen Logik und Speicherung
- einfache Tests
- nachvollziehbare Daten
- gute Grundlage für Synchronisation

---

## Negative Consequences

- zusätzliche Modellierung notwendig
- JSON kann bei sehr großen Datenmengen später ersetzt werden müssen

---

# Future Considerations

Spätere Entscheidungen müssen definieren:

- konkrete Rust-Implementierung
- Versionsmanagement der Datenformate
- Migration alter Daten
- Verschlüsselung
- Synchronisation mit Nextcloud

---

# Final Principle

Interne Datenstrukturen bilden die Verbindung zwischen
Architektur und Implementierung.

NC-PoRe verwendet klare Modelle statt impliziter
Datenannahmen.

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-021: Internal Data Structures and Serialization Format

## Status

Accepted

## Date

2026-07-23

---

# Context

The previous architectural decisions define:

- the Recorder structure
- the data flow
- the concept of a Recording Session
- the Metadata Model

For implementation, NC-PoRe now requires concrete internal data structures.

The data models must:

- be representable in Rust
- remain extensible over the long term
- be exchangeable between components
- be suitable for storage and synchronization

---

# Decision

NC-PoRe uses clearly defined internal data models for the central domain objects.

The data structures are initially modeled as Rust structures.

The main objects are:

```text
RecordingSession

Metadata

AudioReference

SessionStatus
```

The internal models are kept separate from audio hardware, storage, and export.

---

# Core Data Structures

## RecordingSession

The RecordingSession represents a complete recording unit.

Conceptually:

```text
RecordingSession

- id
- status
- created_at
- started_at
- stopped_at
- audio_reference
- metadata
```

---

## SessionStatus

The status describes the lifecycle of a recording.

Examples:

```text
Created

Recording

Stopped

Stored

Exported

Failed
```

---

## AudioReference

The AudioReference describes the connection to the actual audio data.

It does not necessarily contain the audio data itself.

Examples:

```text
AudioReference

- file_path
- format
- duration
- size
```

---

## Metadata

The Metadata Model contains descriptive information about the recording.

See ADR-020.

---

# Serialization Format

NC-PoRe initially uses a human-readable serialization format.

The preferred initial implementation is:

```text
JSON
```

Rationale:

- widely used
- easy to test
- easy to exchange with other systems
- suitable for later synchronization

---

# Alternatives Considered

## Binary Formats

Examples:

- MessagePack
- custom binary formats

Rejected as the initial solution because of:

- poor readability
- higher complexity
- more difficult diagnosis

---

## Database First

Direct storage of all data in a database.

Rejected as the initial solution because of:

- unnecessary complexity
- reduced portability
- no need at the early development stage

---

# Consequences

## Positive Consequences

- clear separation between logic and storage
- simple testing
- traceable data
- good foundation for synchronization

---

## Negative Consequences

- additional modeling required
- JSON may need to be replaced later for very large data volumes

---

# Future Considerations

Later decisions must define:

- concrete Rust implementation
- version management of data formats
- migration of old data
- encryption
- synchronization with Nextcloud

---

# Final Principle

Internal data structures form the connection between architecture and implementation.

NC-PoRe uses explicit models instead of implicit data assumptions.
