# ADR-007: Open Formats and Interoperability

## Status

Accepted

## Date

2026-07-22

---

# Context

NC-PoRe ist eine Open-Source-Podcast-Produktionsplattform.

Audio-Produktionen können über viele Jahre bestehen bleiben.
Daher müssen erzeugte Daten auch langfristig nutzbar sein.

Proprietäre Formate oder geschlossene Workflows können
Benutzer langfristig an einzelne Anwendungen binden.

Dies widerspricht den Grundprinzipien von NC-PoRe.

---

# Decision

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

# Consequences

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

# Alternatives considered

## Proprietäres Projektformat als Hauptspeicher

Verworfen.

Gründe:

- Abhängigkeit vom Anbieter
- erschwerte Archivierung
- nicht vereinbar mit FOSS-Grundsätzen

---

# Notes

NC-PoRe soll ein Werkzeug sein, das Menschen unterstützt.

Die erzeugten Daten gehören den Benutzern.