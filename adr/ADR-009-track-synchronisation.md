# ADR-009: Track Synchronisation

## Status

Accepted

## Date

2026-07-22

---

# Context

NC-PoRe erstellt für jeden Teilnehmer eine eigene
Audiospur.

Da die Spuren unabhängig voneinander lokal aufgenommen
werden, muss eine spätere gemeinsame Verarbeitung die
zeitliche Zuordnung zuverlässig ermöglichen.

Eine einfache Dateireihenfolge reicht nicht aus.

---

# Decision

NC-PoRe verwendet eine sample-basierte interne Zeitbasis
für die Synchronisation einzelner Audiospuren.

Jede Aufnahme enthält Metadaten zur zeitlichen Einordnung.

---

# Synchronisation Data

Jede Spur enthält:

- Session-ID
- Teilnehmer-ID
- Aufnahmestart
- Sample-Rate
- Sample-Offset
- Chunk-Reihenfolge

Beispiel:

```json
{
 "session": "episode-042",
 "track": "host",
 "sample_rate": 48000,
 "start_offset": 0
}

Chunk Synchronisation

Chunks behalten ihre Position innerhalb der Spur.

Beispiel:
Track Host

chunk_0001
samples 0-14399999

chunk_0002
samples 14400000-28799999

Optional Synchronisation Marker

NC-PoRe kann zusätzliche Synchronisationsmarker erzeugen.

Mögliche Nutzung:

manuelle DAW-Ausrichtung
automatische Spurkorrektur
Fehleranalyse
Consequences
Positive Auswirkungen
präzise Mehrspurproduktion
robuste Verarbeitung langer Sessions
geeignet für professionelle DAWs
Negative Auswirkungen
höhere technische Komplexität
Synchronisationslogik erforderlich
Alternatives considered
Nur Startzeit verwenden

Verworfen.

Grund:

Nicht ausreichend präzise für professionelle
Mehrspurproduktion.

Notes

Zeitliche Genauigkeit ist ein Grundbestandteil der
Produktionsqualität von NC-PoRe.