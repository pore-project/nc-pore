# Deutsch ([English version below](#english-version))

# ADR-007: Open Formats and Interoperability

## Status

Accepted

## Date

2026-07-22

---

# Kontext

NC-PoRe ist eine Open-Source-Podcast-Produktionsplattform.

Audio-Produktionen können über viele Jahre bestehen bleiben.
Daher müssen erzeugte Daten auch langfristig nutzbar sein.

Proprietäre Formate oder geschlossene Workflows können
Benutzer langfristig an einzelne Anwendungen binden.

Dies widerspricht den Grundprinzipien von NC-PoRe.

---

# Entscheidung

NC-PoRe verwendet bevorzugt offene und dokumentierte
Datenformate.

Erzeugte Daten sollen mit freien und etablierten
Produktionswerkzeugen verarbeitet werden können.

---

# Audioformate

Primäres Masterformat:
WAV
PCM
48 kHz
24 Bit
Mono

Begründung:

- verlustfrei
- weit verbreitet
- langfristig lesbar
- unterstützt durch professionelle DAWs

---

# Metadaten

Zusätzliche Informationen werden in offenen Formaten
gespeichert.

Beispiel:
metadata.json

Enthalten können sein:

- Sessioninformationen
- Teilnehmer
- Zeitstempel
- Synchronisationsdaten
- Einwilligungsinformationen
- technische Parameter

---

# Produktionsintegration

NC-PoRe soll die Zusammenarbeit mit externen Werkzeugen
ermöglichen.

Beispiele:

- Audacity
- Ardour
- weitere DAWs

Die erzeugten Dateien sollen ohne proprietäre Konvertierung
weiterverarbeitet werden können.

---

# Exportprinzip

NC-PoRe erzeugt Produktionsdaten.

NC-PoRe zwingt Benutzer nicht in einen bestimmten
Bearbeitungsworkflow.

---

# Konsequenzen

## Positive Auswirkungen

- langfristige Datenverfügbarkeit
- freie Werkzeugwahl
- Unterstützung der FOSS-Community
- einfache Migration

---

## Negative Auswirkungen

- weniger Kontrolle über den kompletten Workflow
- mehr Aufwand für Kompatibilität
- zusätzliche Tests verschiedener Werkzeuge

---

# Betrachtete Alternativen

## Proprietäres Projektformat als Hauptspeicher

Verworfen.

Gründe:

- Abhängigkeit vom Anbieter
- erschwerte Archivierung
- nicht vereinbar mit FOSS-Grundsätzen

---

# Hinweise

NC-PoRe soll ein Werkzeug sein, das Menschen unterstützt.

Die erzeugten Daten gehören den Benutzern.

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-007: Open Formats and Interoperability

## Status

Accepted

## Date

2026-07-22

---

# Context

NC-PoRe is an open-source podcast production platform.

Audio productions may remain in use for many years.
Therefore, generated data must remain usable over the long term.

Proprietary formats or closed workflows can bind users to individual applications over the long term.

This contradicts the fundamental principles of NC-PoRe.

---

# Decision

NC-PoRe preferably uses open and documented data formats.

Generated data should be processable with free and established production tools.

---

# Audio Formats

Primary master format:
WAV
PCM
48 kHz
24 bit
Mono

Rationale:

- lossless
- widely used
- readable over the long term
- supported by professional DAWs

---

# Metadata

Additional information is stored in open formats.

Example:
metadata.json

May contain:

- session information
- participants
- timestamps
- synchronization data
- consent information
- technical parameters

---

# Production Integration

NC-PoRe should enable collaboration with external tools.

Examples:

- Audacity
- Ardour
- other DAWs

Generated files should be processable further without proprietary conversion.

---

# Export Principle

NC-PoRe generates production data.

NC-PoRe does not force users into a specific editing workflow.

---

# Consequences

## Positive Effects

- long-term data availability
- freedom of tool choice
- support for the FOSS community
- simple migration

---

## Negative Effects

- less control over the complete workflow
- additional compatibility effort
- additional testing of different tools

---

# Alternatives Considered

## Proprietary Project Format as Primary Storage

Rejected.

Reasons:

- dependency on the provider
- more difficult archiving
- incompatible with FOSS principles

---

# Notes

NC-PoRe should be a tool that supports people.

The generated data belongs to the users.
