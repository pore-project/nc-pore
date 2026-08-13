# Deutsch ([English version below](#english-version))

# ADR-010: Core Data Model

## Status

Accepted

## Date

2026-07-22

---

# Kontext

NC-PoRe verwaltet nicht nur Audiodateien, sondern komplette Produktionsabläufe.

Eine Podcastproduktion besteht aus mehreren Ebenen:

- Personen
- Projekte
- Episoden
- Aufnahmesessions
- Teilnehmern
- Audiospuren
- Audiodateien
- Exporten

Diese Beziehungen müssen eindeutig modelliert werden, damit Verwaltung, Rechte und Produktion zuverlässig funktionieren.

---

# Entscheidung

NC-PoRe verwendet ein hierarchisches Datenmodell.

Die zentrale Struktur lautet:

```text
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

# Kernentitäten

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

```text
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

```text
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

```text
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

# Beziehungsprinzipien

NC-PoRe folgt diesen Beziehungen:

```text
Ein User kann mehrere Projekte besitzen.

Ein Project kann mehrere Episoden enthalten.

Eine Episode kann mehrere Sessions enthalten.

Eine Session enthält mehrere Participants.

Ein Participant besitzt eine oder mehrere Tracks.

Ein Track besteht aus mehreren Chunks.
```

---

# Konsequenzen

## Positive Auswirkungen

- klare Datenstruktur
- einfache Erweiterbarkeit
- Grundlage für APIs
- geeignet für Datenbanken
- unterstützt Rollenmodell

## Negative Auswirkungen

- mehr Verwaltungsaufwand
- komplexeres Datenmodell als einfache Dateisammlung

---

# Betrachtete Alternativen

## Nur Dateistruktur ohne Metadaten

Verworfen.

Grund:

Professionelle Verwaltung und Zusammenarbeit wären nicht möglich.

---

## Flaches Modell

Beispiel:

```text
Episode/
  host.wav
  guest.wav
```

Verworfen.

Grund:

Nicht ausreichend für Sessions, Rechte und langfristige Verwaltung.

---

# Hinweise

Das Datenmodell bildet die Grundlage für:

- Nextcloud-App
- Datenbankstruktur
- API
- Exportfunktionen
- Produktionsworkflow

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-010: Core Data Model

## Status

Accepted

## Date

2026-07-22

---

# Context

NC-PoRe manages not only audio files, but complete production workflows.

A podcast production consists of several levels:

- people
- projects
- episodes
- recording sessions
- participants
- audio tracks
- audio files
- exports

These relationships must be modeled unambiguously so that administration, permissions, and production work reliably.

---

# Decision

NC-PoRe uses a hierarchical data model.

The central structure is:

```text
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

A person with access to NC-PoRe.

Attributes:

- user ID
- name
- login information
- roles
- permissions

---

## Project

A podcast or production project.

Examples:

- own podcast
- client production
- series format

Attributes:

- project ID
- name
- description
- owner
- members

---

## Episode

A single production within a project.

Example:

```text
Project:
Soundtrack of Life

Episode:
Episode 42
```

Attributes:

- episode ID
- title
- status
- publication information

---

## Session

A specific recording session.

An episode may have multiple sessions.

Examples:

- test recording
- main recording
- additional recording

Attributes:

- session ID
- date
- participants
- status
- consent information

---

## Participant

A person participating in a session.

May be:

- registered user
- guest

Attributes:

- participant ID
- role within the session
- consent status
- assigned track

---

## Track

A single audio track belonging to a participant.

Example:

```text
Host.wav
Guest.wav
CoHost.wav
```

Attributes:

- track ID
- participant
- format
- sample rate
- synchronization data

---

## Chunk

A partial recording within a track.

Example:

```text
Host

chunk_0001.wav
chunk_0002.wav
chunk_0003.wav
```

Attributes:

- chunk ID
- track ID
- sequence
- start sample
- duration
- checksum

---

## Export

A generated production package.

Examples:

- WAV package
- Audacity project
- Ardour session

Attributes:

- export ID
- format
- creation date
- destination

---

# Relationship Principles

NC-PoRe follows these relationships:

```text
A User can own multiple Projects.

A Project can contain multiple Episodes.

An Episode can contain multiple Sessions.

A Session contains multiple Participants.

A Participant owns one or more Tracks.

A Track consists of multiple Chunks.
```

---

# Consequences

## Positive Effects

- clear data structure
- easy extensibility
- foundation for APIs
- suitable for databases
- supports the role model

## Negative Effects

- more administrative effort
- more complex data model than a simple collection of files

---

# Alternatives Considered

## File Structure Without Metadata

Rejected.

Reason:

Professional administration and collaboration would not be possible.

---

## Flat Model

Example:

```text
Episode/
  host.wav
  guest.wav
```

Rejected.

Reason:

Not sufficient for sessions, permissions, and long-term administration.

---

# Notes

The data model provides the foundation for:

- Nextcloud app
- database structure
- API
- export functions
- production workflow
