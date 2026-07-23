# ADR-026: Session Data and Storage Architecture

* Status: Accepted
* Date: 2026-07-23
* Decision Type: Architecture

---

# Deutsch (English version below)

## Kontext

NC-PoRe wurde als **Nextcloud Podcast Production Environment** definiert.

Der zentrale Anwendungsfall ist die kollaborative Produktion von Podcasts mit mehreren Teilnehmern, unterschiedlichen Geräten und perspektivisch unterschiedlichen Medienformaten.

Eine Podcast-Produktion besteht jedoch nicht nur aus einzelnen Dateien.

Aus Sicht der Benutzer existiert eine zusammenhängende Produktion:

* eine bestimmte Folge
* ein bestimmtes Gespräch
* bestimmte Teilnehmer
* bestimmte Aufnahmen
* ein bestimmter Produktionsprozess

Daher ist eine reine Betrachtung von Audiodateien als primäre Einheit nicht ausreichend.

NC-PoRe benötigt ein Datenmodell, das den tatsächlichen Arbeitsprozess abbildet.

---

## Entscheidung

Die zentrale fachliche Einheit von NC-PoRe ist die:

**Production Session**

Eine Production Session umfasst alle relevanten Informationen und Medien, die zu einer gemeinsamen Produktion gehören.

Medien und Dateien werden als Bestandteile einer Session betrachtet und nicht als eigenständige Primärobjekte.

NC-PoRe verwaltet Medien als Assets, ermöglicht jedoch weiterhin den direkten Zugriff auf zugrunde liegende Mediendateien für externe Produktionswerkzeuge.

Der Benutzer bleibt Eigentümer seiner Produktionsdaten und wird nicht durch das System eingeschränkt.

---

## Session-Modell

Eine Production Session kann enthalten:

```text
Production Session

├── Session Metadata
│
├── Participants
│
├── Audio Streams
│
├── Video Streams (future)
│
├── Events
│
├── Notes
│
├── Assets
│
└── Exports
```

---

## Core-Modell und Speicherung

NC-PoRe trennt zwischen:

1. dem fachlichen Datenmodell
2. der technischen Speicherung

Das interne Modell von NC-PoRe definiert, was eine Session ist.

Der konkrete Speicherort ist eine technische Implementierung.

Die Architektur folgt diesem Prinzip:

```text
NC-PoRe Core

        |
        |
Session Data Model

        |
        |
Storage Provider Layer

        |
        |
--------------------------------
|              |               |
Nextcloud    Local Storage   Future Providers
```

---

## Verhältnis zu Nextcloud

Nextcloud ist die primäre Integrationsplattform der ersten Version.

In V1 werden Nextcloud-Funktionen genutzt für:

* Speicherung von Produktionsdaten
* Synchronisation
* Zugriff und Austausch
* Zusammenarbeit

Das Datenmodell von NC-PoRe wird jedoch nicht durch die interne Struktur von Nextcloud vorgegeben.

Nextcloud ist ein Provider.

NC-PoRe bleibt Eigentümer des fachlichen Modells.

---

## Zugriff auf Mediendateien und externe Werkzeuge

NC-PoRe abstrahiert die Verwaltung von Medien, ersetzt jedoch nicht automatisch bestehende professionelle Produktionswerkzeuge.

Die zugrunde liegenden Mediendateien bleiben zugänglich.

Ein typischer Workflow kann sein:

```text
NC-PoRe Production Session

        |
        |
    Audio Assets

        |
        |
-------------------------
|                       |
Audacity              Ardour

        |
        |
Final Export
```

Beispielsweise können:

* WAV-Dateien direkt geöffnet werden
* einzelne Spuren in externe Audio-Editoren übernommen werden
* fertige Exporte wieder als Assets in die Session zurückgeführt werden

Eine spätere Integration mit externen Werkzeugen ist möglich, aber nicht Bestandteil von V1.

---

## Storage Provider Prinzip

Die Speicherung wird über klar definierte Schnittstellen abstrahiert.

Dadurch bleiben zukünftige Erweiterungen möglich:

* lokale Speicherung
* weitere Cloudanbieter
* WebDAV-basierte Speicher
* S3-kompatible Speicher
* weitere zukünftige Provider

Die Benutzer sollen dabei möglichst keinen Unterschied erkennen.

---

## Grundprinzipien

### 1. Session First

Die Session ist die zentrale Einheit.

Nicht:

> "Hier ist eine Audiodatei."

Sondern:

> "Hier ist eine Podcast-Produktion mit Teilnehmern, Medien und Produktionsverlauf."

---

### 2. Assets statt Dateitypen

NC-PoRe denkt in Produktionsbestandteilen, nicht in einzelnen Dateiformaten.

Ein Asset kann beispielsweise sein:

* Audioaufnahme
* Videoaufnahme
* Coverbild
* Transkript
* Notizen
* Exportdatei
* Metadaten

Neue Asset-Typen sollen später ergänzt werden können.

---

### 3. Der Speicherort ist nicht die Benutzerperspektive

Der Benutzer denkt nicht in Speicherorten.

Der Benutzer denkt in Produktionen.

Daher gilt:

> Der Speicherort ist eine technische Entscheidung. Entscheidend ist, dass die Produktion sicher verfügbar, nachvollziehbar und nutzbar bleibt.

---

### 4. Keine Abhängigkeit durch Abstraktion

NC-PoRe organisiert Produktionsdaten, sperrt Benutzer jedoch nicht in ein proprietäres Format ein.

Die direkte Nutzung von Mediendateien mit etablierten Werkzeugen bleibt möglich.

---

## Konsequenzen

### Vorteile

* klare Trennung zwischen Fachmodell und Technologie
* Erweiterbarkeit für Video und weitere Medien
* bessere Unterstützung verteilter Produktion
* zukünftige Provider möglich
* natürliche Abbildung des Benutzer-Workflows
* freie Nutzung etablierter Produktionswerkzeuge

### Nachteile

* zusätzliche Abstraktion
* höherer anfänglicher Entwicklungsaufwand
* mehr Architekturdisziplin erforderlich

Diese Nachteile werden bewusst akzeptiert.

---

## Nicht-Ziele

Diese Entscheidung bedeutet nicht:

* dass bereits alle Medienformate unterstützt werden müssen
* dass sofort jede mögliche Cloud integriert wird
* dass die konkrete Datenbanktechnologie festgelegt wird
* dass Speicherimplementierungen bereits vollständig definiert werden
* dass NC-PoRe eigene Audio-Editoren ersetzen muss

Diese Entscheidungen folgen in späteren ADRs.

---

## Leitgedanke

NC-PoRe speichert nicht einfach Dateien.

NC-PoRe verwaltet Produktionen.

Eine Production Session verbindet Menschen, Medien und Arbeitsabläufe zu einer gemeinsamen Einheit.

Die Benutzer behalten jederzeit Zugriff auf ihre eigenen Produktionsdaten und können bestehende Werkzeuge weiterhin verwenden.

---

# English (Deutsche Version oben)

## Context

NC-PoRe has been defined as a **Nextcloud Podcast Production Environment**.

The main use case is collaborative podcast production with multiple participants, different devices and future support for different media formats.

A podcast production is not only a collection of individual files.

From the user's perspective, there is a connected production:

* a specific episode
* a specific conversation
* specific participants
* specific recordings
* a specific production workflow

Therefore, treating audio files as the primary unit is insufficient.

NC-PoRe requires a data model that represents the real production process.

---

## Decision

The central domain entity of NC-PoRe is the:

**Production Session**

A Production Session contains all relevant information and media belonging to one collaborative production.

Media and files are treated as parts of a session, not as independent primary objects.

NC-PoRe manages media as assets while preserving direct access to underlying media files for external production tools.

Users remain owners of their production data and are not restricted by the system.

---

## Session Model

A Production Session can contain:

```text
Production Session

├── Session Metadata
│
├── Participants
│
├── Audio Streams
│
├── Video Streams (future)
│
├── Events
│
├── Notes
│
├── Assets
│
└── Exports
```

---

## Core Model and Storage

NC-PoRe separates:

1. the domain data model
2. the technical storage implementation

The internal NC-PoRe model defines what a session is.

The actual storage location is a technical implementation detail.

Architecture:

```text
NC-PoRe Core

        |
        |
Session Data Model

        |
        |
Storage Provider Layer

        |
        |
--------------------------------
|              |               |
Nextcloud    Local Storage   Future Providers
```

---

## Relationship to Nextcloud

Nextcloud is the primary integration platform of version 1.

Version 1 uses Nextcloud capabilities for:

* production data storage
* synchronization
* access and sharing
* collaboration

However, the NC-PoRe data model is not defined by the internal structure of Nextcloud.

Nextcloud is a provider.

NC-PoRe owns the domain model.

---

## Media Access and External Tools

NC-PoRe abstracts media management but does not automatically replace existing professional production tools.

Underlying media files remain accessible.

A typical workflow can be:

```text
NC-PoRe Production Session

        |
        |
    Audio Assets

        |
        |
-------------------------
|                       |
Audacity              Ardour

        |
        |
Final Export
```

For example:

* WAV files can be opened directly
* individual tracks can be processed in external audio editors
* final exports can be returned to the session as assets

Future integration with external tools is possible, but not part of V1.

---

## Storage Provider Principle

Storage is abstracted through clearly defined interfaces.

This enables future extensions:

* local storage
* additional cloud providers
* WebDAV-based storage
* S3-compatible storage
* future providers

Users should ideally not notice these differences.

---

## Principles

### 1. Session First

The session is the central unit.

Not:

> "Here is an audio file."

But:

> "Here is a podcast production with participants, media and production history."

---

### 2. Assets instead of file types

NC-PoRe thinks in production elements, not individual file formats.

An asset can be:

* audio recording
* video recording
* cover image
* transcript
* notes
* export file
* metadata

New asset types should be addable later.

---

### 3. Storage location is not the user perspective

Users do not think in storage locations.

Users think in productions.

Therefore:

> Storage location is a technical decision. What matters is that the production remains secure, traceable and usable.

---

### 4. No Dependency Through Abstraction

NC-PoRe organizes production data without locking users into a proprietary format.

Direct use of media files with established tools remains possible.

---

## Consequences

### Benefits

* clear separation between domain model and technology
* extensibility for video and additional media
* better support for distributed production
* future providers possible
* natural representation of user workflows
* freedom to use established production tools

### Costs

* additional abstraction
* higher initial development effort
* requires architectural discipline

These costs are consciously accepted.

---

## Non-Goals

This decision does not mean:

* all media formats must already be supported
* every possible cloud provider must be integrated immediately
* the database technology is already decided
* storage implementations are already fully specified
* NC-PoRe must replace audio editors

These decisions will follow in later ADRs.

---

## Guiding Principle

NC-PoRe does not simply store files.

NC-PoRe manages productions.

A Production Session connects people, media and workflows into one shared unit.

Users always retain access to their own production data and can continue using established tools.
