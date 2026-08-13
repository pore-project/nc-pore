# Deutsch ([English version below](#english-version))

# ADR-009: Track Synchronisation

## Status

Accepted

## Date

2026-07-22

---

# Kontext

NC-PoRe erstellt für jeden Teilnehmer eine eigene Audiospur.

Da die Spuren unabhängig voneinander lokal aufgenommen werden, muss eine spätere gemeinsame Verarbeitung die zeitliche Zuordnung zuverlässig ermöglichen.

Eine einfache Dateireihenfolge reicht nicht aus.

---

# Entscheidung

NC-PoRe verwendet eine sample-basierte interne Zeitbasis für die Synchronisation einzelner Audiospuren.

Jede Aufnahme enthält Metadaten zur zeitlichen Einordnung.

---

# Synchronisationsdaten

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
```

---

# Chunk-Synchronisation

Chunks behalten ihre Position innerhalb der Spur.

Beispiel:

Track Host

chunk_0001
samples 0-14399999

chunk_0002
samples 14400000-28799999

---

# Optionale Synchronisationsmarker

NC-PoRe kann zusätzliche Synchronisationsmarker erzeugen.

Mögliche Nutzung:

- manuelle DAW-Ausrichtung
- automatische Spurkorrektur
- Fehleranalyse

---

# Konsequenzen

## Positive Auswirkungen

- präzise Mehrspurproduktion
- robuste Verarbeitung langer Sessions
- geeignet für professionelle DAWs

## Negative Auswirkungen

- höhere technische Komplexität
- Synchronisationslogik erforderlich

---

# Betrachtete Alternativen

## Nur Startzeit verwenden

Verworfen.

Grund:

Nicht ausreichend präzise für professionelle Mehrspurproduktion.

---

# Hinweise

Zeitliche Genauigkeit ist ein Grundbestandteil der Produktionsqualität von NC-PoRe.

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-009: Track Synchronisation

## Status

Accepted

## Date

2026-07-22

---

# Context

NC-PoRe creates a separate audio track for each participant.

Since the tracks are recorded independently and locally, later joint processing must be able to establish their temporal relationship reliably.

A simple file order is not sufficient.

---

# Decision

NC-PoRe uses a sample-based internal time base for synchronizing individual audio tracks.

Each recording contains metadata for its temporal positioning.

---

# Synchronization Data

Each track contains:

- session ID
- participant ID
- recording start
- sample rate
- sample offset
- chunk order

Example:

```json
{
  "session": "episode-042",
  "track": "host",
  "sample_rate": 48000,
  "start_offset": 0
}
```

---

# Chunk Synchronization

Chunks retain their position within the track.

Example:

Track Host

chunk_0001
samples 0-14399999

chunk_0002
samples 14400000-28799999

---

# Optional Synchronization Markers

NC-PoRe may generate additional synchronization markers.

Possible uses:

- manual DAW alignment
- automatic track correction
- error analysis

---

# Consequences

## Positive Effects

- precise multitrack production
- robust processing of long sessions
- suitable for professional DAWs

## Negative Effects

- higher technical complexity
- synchronization logic required

---

# Alternatives Considered

## Use Start Time Only

Rejected.

Reason:

Not sufficiently precise for professional multitrack production.

---

# Notes

Temporal accuracy is a fundamental part of NC-PoRe's production quality.
