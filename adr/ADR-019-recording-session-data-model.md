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

* Audiodaten
* technische Aufnahmeinformationen
* Metadaten
* Statusinformationen

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

* eindeutige Identifikation einer Aufnahme
* Zuordnung von Dateien und Metadaten

---

## Timestamps

Verantwortlich für:

* zeitliche Einordnung
* spätere Sortierung
* Synchronisation

---

## Status

Beschreibt den aktuellen Zustand der Aufnahme.

Beispiele:

* created
* recording
* stopped
* stored
* failed

---

## Audio Reference

Verweist auf die gespeicherten Audiodaten.

Die Session enthält nicht zwingend die Audiodaten
selbst.

---

## Metadata

Enthält zusätzliche Informationen:

* technische Parameter
* Benutzerinformationen
* optionale Beschreibung

---

# Alternatives Considered

## Audio File as Primary Object

Die Audiodatei selbst ist die vollständige
Repräsentation einer Aufnahme.

Verworfen wegen:

* fehlender Erweiterbarkeit
* schwieriger Verwaltung zusätzlicher Informationen
* schlechter Grundlage für Synchronisation

---

## Metadata Embedded Only in Audio Files

Alle Informationen werden ausschließlich in
Audiodateien gespeichert.

Verworfen wegen:

* Abhängigkeit von Dateiformaten
* erschwerter Verarbeitung
* schlechter Trennung von Verantwortlichkeiten

---

# Consequences

## Positive Consequences

* klare Repräsentation einer Aufnahme
* bessere Erweiterbarkeit
* einfache Verwaltung mehrerer Aufnahmezustände
* Grundlage für spätere Synchronisation

---

## Negative Consequences

* zusätzliche Datenstruktur
* mehr Verwaltungslogik

---

# Future Considerations

Spätere Entscheidungen müssen definieren:

* konkretes Rust-Datenmodell
* Speicherung des Session-Modells
* Serialisierungsformat
* Synchronisation mit Nextcloud
* Umgang mit unterbrochenen Aufnahmen

---

# Final Principle

Eine Aufnahme ist mehr als eine Audiodatei.

NC-PoRe behandelt jede Aufnahme als verwaltbare,
nachvollziehbare und erweiterbare Einheit.
