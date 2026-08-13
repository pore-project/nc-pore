# Deutsch ([English version below](#english-version))

# ADR-019: Recording Session Data Model

## Status

Accepted

## Date

2026-07-23

---

# Context

Der NC-PoRe Recorder benötigt eine definierte
Datenstruktur zur Beschreibung einer Aufnahme.

Eine Aufnahme besteht nicht nur aus Audiodaten.

Zusätzliche Informationen sind notwendig, um eine
Aufnahme nachvollziehbar zu verwalten, zu speichern
und später weiterzuverarbeiten.

Das Datenmodell muss sowohl einfache lokale
Aufnahmen als auch zukünftige Erweiterungen
unterstützen.

---

# Decision

NC-PoRe führt das Konzept einer
`Recording Session` ein.

Eine Recording Session beschreibt eine logische
Aufnahmeeinheit.

Die Session verbindet:

- Audiodaten
- technische Aufnahmeinformationen
- Metadaten
- Statusinformationen

Die Audio-Daten selbst werden getrennt von der
Session-Beschreibung behandelt.

---

# Recording Session Concept

Eine Session besitzt einen eindeutigen Lebenszyklus:

```text
Created

  ↓

Recording

  ↓

Stopped

  ↓

Stored

  ↓

Exported
```

Der aktuelle Zustand einer Session wird durch das
Session Management verwaltet.

---

# Initial Data Model

Eine Recording Session enthält mindestens:

```text
RecordingSession

- session_id
- created_at
- started_at
- stopped_at
- status
- audio_reference
- metadata
```

---

# Data Responsibilities

## Session ID

Verantwortlich für:

- eindeutige Identifikation einer Aufnahme
- Zuordnung von Dateien und Metadaten

---

## Timestamps

Verantwortlich für:

- zeitliche Einordnung
- spätere Sortierung
- Synchronisation

---

## Status

Beschreibt den aktuellen Zustand der Aufnahme.

Beispiele:

- created
- recording
- stopped
- stored
- failed

---

## Audio Reference

Verweist auf die gespeicherten Audiodaten.

Die Session enthält nicht zwingend die Audiodaten
selbst.

---

## Metadata

Enthält zusätzliche Informationen:

- technische Parameter
- Benutzerinformationen
- optionale Beschreibung

---

# Alternatives Considered

## Audio File as Primary Object

Die Audiodatei selbst ist die vollständige
Repräsentation einer Aufnahme.

Verworfen wegen:

- fehlender Erweiterbarkeit
- schwieriger Verwaltung zusätzlicher Informationen
- schlechter Grundlage für Synchronisation

---

## Metadata Embedded Only in Audio Files

Alle Informationen werden ausschließlich in
Audiodateien gespeichert.

Verworfen wegen:

- Abhängigkeit von Dateiformaten
- erschwerter Verarbeitung
- schlechter Trennung von Verantwortlichkeiten

---

# Consequences

## Positive Consequences

- klare Repräsentation einer Aufnahme
- bessere Erweiterbarkeit
- einfache Verwaltung mehrerer Aufnahmezustände
- Grundlage für spätere Synchronisation

---

## Negative Consequences

- zusätzliche Datenstruktur
- mehr Verwaltungslogik

---

# Future Considerations

Spätere Entscheidungen müssen definieren:

- konkretes Rust-Datenmodell
- Speicherung des Session-Modells
- Serialisierungsformat
- Synchronisation mit Nextcloud
- Umgang mit unterbrochenen Aufnahmen

---

# Final Principle

Eine Aufnahme ist mehr als eine Audiodatei.

NC-PoRe behandelt jede Aufnahme als verwaltbare,
nachvollziehbare und erweiterbare Einheit.

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-019: Recording Session Data Model

## Status

Accepted

## Date

2026-07-23

---

# Context

The NC-PoRe Recorder requires a defined data structure to describe a recording.

A recording consists of more than audio data alone.

Additional information is required to manage, store, and process a recording in a traceable manner.

The data model must support both simple local recordings and future extensions.

---

# Decision

NC-PoRe introduces the concept of a `Recording Session`.

A Recording Session describes a logical recording unit.

The session connects:

- audio data
- technical recording information
- metadata
- status information

The audio data itself is handled separately from the session description.

---

# Recording Session Concept

A session has a defined lifecycle:

```text
Created

  ↓

Recording

  ↓

Stopped

  ↓

Stored

  ↓

Exported
```

The current state of a session is managed by Session Management.

---

# Initial Data Model

A Recording Session contains at least:

```text
RecordingSession

- session_id
- created_at
- started_at
- stopped_at
- status
- audio_reference
- metadata
```

---

# Data Responsibilities

## Session ID

Responsible for:

- uniquely identifying a recording
- associating files and metadata

---

## Timestamps

Responsible for:

- temporal positioning
- later sorting
- synchronization

---

## Status

Describes the current state of the recording.

Examples:

- created
- recording
- stopped
- stored
- failed

---

## Audio Reference

References the stored audio data.

The session does not necessarily contain the audio data itself.

---

## Metadata

Contains additional information:

- technical parameters
- user information
- optional description

---

# Alternatives Considered

## Audio File as Primary Object

The audio file itself is the complete representation of a recording.

Rejected because of:

- lack of extensibility
- difficult management of additional information
- poor foundation for synchronization

---

## Metadata Embedded Only in Audio Files

All information is stored exclusively in audio files.

Rejected because of:

- dependency on file formats
- more difficult processing
- poor separation of responsibilities

---

# Consequences

## Positive Consequences

- clear representation of a recording
- better extensibility
- simple management of multiple recording states
- foundation for later synchronization

---

## Negative Consequences

- additional data structure
- more management logic

---

# Future Considerations

Later decisions must define:

- concrete Rust data model
- storage of the session model
- serialization format
- synchronization with Nextcloud
- handling of interrupted recordings

---

# Final Principle

A recording is more than an audio file.

NC-PoRe treats every recording as a manageable, traceable, and extensible unit.
