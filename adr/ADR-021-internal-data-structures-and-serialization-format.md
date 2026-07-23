# ADR-021: Internal Data Structures and Serialization Format

## Status

Accepted

## Date

2026-07-23

---

# Context

Die bisherigen Architekturentscheidungen definieren:

* die Recorder-Struktur
* den Datenfluss
* das Konzept einer Recording Session
* das Metadata Model

Für die Implementierung benötigt NC-PoRe nun konkrete
interne Datenstrukturen.

Die Datenmodelle müssen:

* in Rust abbildbar sein
* langfristig erweiterbar bleiben
* zwischen Komponenten austauschbar sein
* für Speicherung und Synchronisation geeignet sein

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

* weit verbreitet
* einfach zu testen
* gut mit anderen Systemen austauschbar
* geeignet für spätere Synchronisation

---

# Alternatives Considered

## Binary Formats

Beispiele:

* MessagePack
* eigene Binärformate

Verworfen als erste Lösung wegen:

* schlechter Lesbarkeit
* höherer Komplexität
* schwieriger Diagnose

---

## Database First

Direkte Speicherung aller Daten in einer Datenbank.

Verworfen als erste Lösung wegen:

* unnötiger Komplexität
* erschwerter Portabilität
* fehlendem Bedarf im frühen Entwicklungsstadium

---

# Consequences

## Positive Consequences

* klare Trennung zwischen Logik und Speicherung
* einfache Tests
* nachvollziehbare Daten
* gute Grundlage für Synchronisation

---

## Negative Consequences

* zusätzliche Modellierung notwendig
* JSON kann bei sehr großen Datenmengen später ersetzt werden müssen

---

# Future Considerations

Spätere Entscheidungen müssen definieren:

* konkrete Rust-Implementierung
* Versionsmanagement der Datenformate
* Migration alter Daten
* Verschlüsselung
* Synchronisation mit Nextcloud

---

# Final Principle

Interne Datenstrukturen bilden die Verbindung zwischen
Architektur und Implementierung.

NC-PoRe verwendet klare Modelle statt impliziter
Datenannahmen.
