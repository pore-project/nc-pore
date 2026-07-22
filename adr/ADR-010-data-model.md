# ADR-010: Core Data Model

## Status

Accepted

## Date

2026-07-22

---

# Context

NC-PoRe verwaltet nicht nur Audiodateien, sondern komplette
Produktionsabläufe.

Eine Podcastproduktion besteht aus mehreren Ebenen:

- Personen
- Projekte
- Episoden
- Aufnahmesessions
- Teilnehmern
- Audiospuren
- Audiodateien
- Exporten

Diese Beziehungen müssen eindeutig modelliert werden,
damit Verwaltung, Rechte und Produktion zuverlässig
funktionieren.

---

# Decision

NC-PoRe verwendet ein hierarchisches Datenmodell.

Die zentrale Struktur lautet:

```
User

Project
 |
 +-- Episode
      |
      +-- Session
            |
            +-- Participant
                  |
                  +-- Track
                        |
                        +-- Chunk

            |
            +-- Export
```

---

# Core Entities

## User

Eine Person mit Zugang zu NC-PoRe.

Attribute:

- Benutzer-ID
- Name
- Login-Information
- Rollen
- Berechtigungen

---

## Project

Ein Podcast- oder Produktionsprojekt.

Beispiele:

- eigener Podcast
- Kundenproduktion
- Serienformat

Attribute:

- Projekt-ID
- Name
- Beschreibung
- Besitzer
- Mitglieder

---

## Episode

Eine einzelne Produktion innerhalb eines Projekts.

Beispiel:

```
Projekt:
Soundtrack of Life

Episode:
Folge 42
```

Attribute:

- Episoden-ID
- Titel
- Status
- Veröffentlichungsinformationen

---

## Session

Eine konkrete Aufnahmesitzung.

Eine Episode kann mehrere Sessions besitzen.

Beispiel:

- Testaufnahme
- Hauptaufnahme
- Nachaufnahme

Attribute:

- Session-ID
- Datum
- Teilnehmer
- Status
- Consent-Informationen

---

## Participant

Eine Person, die an einer Session teilnimmt.

Kann sein:

- registrierter Benutzer
- Gast

Attribute:

- Teilnehmer-ID
- Rolle innerhalb der Session
- Einwilligungsstatus
- zugeordnete Spur

---

## Track

Eine einzelne Audiospur eines Teilnehmers.

Beispiel:

```
Host.wav
Gast.wav
CoHost.wav
```

Attribute:

- Track-ID
- Teilnehmer
- Format
- Sample-Rate
- Synchronisationsdaten

---

## Chunk

Eine Teilaufnahme innerhalb einer Spur.

Beispiel:

```
Host

chunk_0001.wav
chunk_0002.wav
chunk_0003.wav
```

Attribute:

- Chunk-ID
- Track-ID
- Reihenfolge
- Startsample
- Dauer
- Prüfsumme

---

## Export

Ein erzeugtes Produktionspaket.

Beispiele:

- WAV-Paket
- Audacity-Projekt
- Ardour-Session

Attribute:

- Export-ID
- Format
- Erstellungsdatum
- Ziel

---

# Relationship Principles

NC-PoRe folgt diesen Beziehungen:

```
Ein User kann mehrere Projekte besitzen.

Ein Project kann mehrere Episoden enthalten.

Eine Episode kann mehrere Sessions enthalten.

Eine Session enthält mehrere Participants.

Ein Participant besitzt eine oder mehrere Tracks.

Ein Track besteht aus mehreren Chunks.
```

---

# Consequences

## Positive Auswirkungen

- klare Datenstruktur
- einfache Erweiterbarkeit
- Grundlage für APIs
- geeignet für Datenbanken
- unterstützt Rollenmodell

---

## Negative Auswirkungen

- mehr Verwaltungsaufwand
- komplexeres Datenmodell als einfache Dateisammlung

---

# Alternatives considered

## Nur Dateistruktur ohne Metadaten

Verworfen.

Grund:

Professionelle Verwaltung und Zusammenarbeit
wären nicht möglich.

---

## Flaches Modell

Beispiel:

```
Episode/
  host.wav
  guest.wav
```

Verworfen.

Grund:

Nicht ausreichend für Sessions, Rechte und
langfristige Verwaltung.

---

# Notes

Das Datenmodell bildet die Grundlage für:

- Nextcloud-App
- Datenbankstruktur
- API
- Exportfunktionen
- Produktionsworkflow
```