<a id="deutsch"></a>

# Deutsch

# ADR-026: Session Data and Storage Architecture

* Status: Accepted
* Date: 2026-07-23
* Decision Type: Architecture

---

## Kontext

Eine Podcast-Produktion besteht nicht nur aus einzelnen Dateien. Aus Sicht der Benutzer existiert eine zusammenhängende Produktion mit Teilnehmern, Aufnahmen und Produktionsdaten. NC-PoRe benötigt daher ein Datenmodell, das diesen Zusammenhang abbildet.

## Entscheidung

Die zentrale fachliche Einheit von NC-PoRe ist die **Production Session**.

Eine Production Session umfasst die relevanten Informationen und Medien einer gemeinsamen Produktion. Medien und Dateien werden als Bestandteile einer Session betrachtet und nicht als eigenständige Primärobjekte.

NC-PoRe verwaltet Medien als Assets und erhält den direkten Zugriff auf die zugrunde liegenden Mediendaten für externe Werkzeuge. Der Benutzer bleibt Eigentümer seiner Produktionsdaten.

## Session-Modell

```text
Production Session
├── Session Metadata
├── Participants
├── Media Streams
├── Events
├── Notes
├── Assets
└── Exports
```

## Core-Modell und Speicherung

NC-PoRe trennt zwischen dem fachlichen Datenmodell und der technischen Speicherung.

```text
NC-PoRe Core
      |
Session Data Model
      |
Storage Provider Layer
      |
Storage Provider
```

Der konkrete Speicherort ist eine technische Implementierung. Das Domain-Modell darf nicht von der internen Struktur eines bestimmten Storage-Systems abhängen.

## Verhältnis zu Nextcloud

Nextcloud ist die Integrationsplattform der ersten Version. Die Nutzung von Nextcloud für Speicherung, Synchronisation, Zugriff und Zusammenarbeit darf das fachliche NC-PoRe-Datenmodell nicht bestimmen.

## Zugriff auf Mediendateien

Die zugrunde liegenden Mediendaten bleiben zugänglich. Externe Werkzeuge können diese Daten weiterverarbeiten. NC-PoRe ersetzt nicht die vom Benutzer gewählten Produktionswerkzeuge.

## Storage Provider Prinzip

Die Speicherung wird über klar definierte Schnittstellen abstrahiert. Dadurch können konkrete Storage-Implementierungen ausgetauscht werden, ohne das fachliche Datenmodell an eine Infrastrukturtechnologie zu koppeln.

## Grundprinzipien

### 1. Session First

Die Session ist die zentrale Einheit der Benutzer- und Fachdaten.

### 2. Assets statt isolierter Dateien

Produktionsdaten werden als Bestandteile einer Session verwaltet, während die zugrunde liegenden Dateien zugänglich bleiben.

### 3. Speicherort ist technische Umsetzung

Der Benutzer arbeitet mit der Produktion; der konkrete Speichermechanismus bleibt eine technische Angelegenheit.

### 4. Keine proprietäre Bindung

NC-PoRe organisiert Produktionsdaten, ohne Benutzer an ein proprietäres Datenformat für die weitere Verarbeitung zu binden.

## Konsequenzen

### Vorteile

* klare Trennung zwischen Fachmodell und Technologie
* austauschbare Storage-Implementierungen
* natürliche Abbildung des Produktions-Workflows
* Zugriff auf eigene Produktionsdaten bleibt erhalten

### Nachteile

* zusätzliche Abstraktion
* höherer Entwicklungsaufwand
* zusätzliche Architekturdisziplin

## Nicht-Ziele

Diese Entscheidung legt keine konkreten zukünftigen Storage-Anbieter, Medienarten oder externen Integrationen fest.

## Leitgedanke

NC-PoRe speichert nicht einfach Dateien. NC-PoRe verwaltet Produktionen.

---

<a id="english-version"></a>

# English Version

# ADR-026: Session Data and Storage Architecture

* Status: Accepted
* Date: 2026-07-23
* Decision Type: Architecture

---

## Context

A podcast production is more than a collection of individual files. From the user's perspective there is a connected production containing participants, recordings and production data. NC-PoRe therefore requires a data model representing this relationship.

## Decision

The central domain entity of NC-PoRe is the **Production Session**.

A Production Session contains the relevant information and media belonging to a shared production. Media and files are treated as parts of a session rather than independent primary objects.

NC-PoRe manages media as assets while preserving access to the underlying media data for external tools. Users remain owners of their production data.

## Session Model

```text
Production Session
├── Session Metadata
├── Participants
├── Media Streams
├── Events
├── Notes
├── Assets
└── Exports
```

## Core Model and Storage

NC-PoRe separates the domain data model from technical storage.

```text
NC-PoRe Core
      |
Session Data Model
      |
Storage Provider Layer
      |
Storage Provider
```

The concrete storage location is a technical implementation detail. The domain model must not depend on the internal structure of a particular storage system.

## Relationship to Nextcloud

Nextcloud is the integration platform of the first version. Its use for storage, synchronization, access and collaboration must not define the NC-PoRe domain model.

## Media Access

Underlying media data remains accessible. External tools can process that data. NC-PoRe does not replace the production tools chosen by the user.

## Storage Provider Principle

Storage is abstracted through clearly defined interfaces. Concrete storage implementations can therefore be changed without coupling the domain model to infrastructure technology.

## Principles

### 1. Session First

The session is the central unit of user and domain data.

### 2. Assets instead of isolated files

Production data is managed as part of a session while the underlying files remain accessible.

### 3. Storage location is an implementation detail

Users work with the production; the concrete storage mechanism remains a technical concern.

### 4. No proprietary lock-in

NC-PoRe organizes production data without locking users into a proprietary format for further processing.

## Consequences

### Benefits

* clear separation between domain model and technology
* replaceable storage implementations
* natural representation of the production workflow
* continued access to users' own production data

### Costs

* additional abstraction
* higher development effort
* additional architectural discipline

## Non-Goals

This decision does not define specific future storage providers, media types or external integrations.

## Guiding Principle

NC-PoRe does not simply store files. NC-PoRe manages productions.
