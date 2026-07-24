# NC-PoRe Project Status

* Version: 1.1
* Date: 2026-07-24

---

# Deutsch ([English version below](#english-version))

---

# Project Phase

Current phase:

## Foundation and Architecture Definition Completed

NC-PoRe hat die grundlegende Konzeptions- und Architekturphase abgeschlossen.

Der Fokus lag auf:

* Architekturentscheidungen
* Anforderungen
* Datenmodell und Session-Konzept
* Projektstruktur
* FOSS-Grundlagen
* Verteilung und Zusammenarbeit
* Sicherheits- und Rollenmodellen

Die grundlegenden Architekturentscheidungen sind dokumentiert.

Die nächste Phase ist die technische Umsetzungsvorbereitung.

---

# Project Vision

NC-PoRe ist eine selbsthostbare Open-Source-Plattform
für professionelle Podcast-Aufnahmen und Produktion.

Zentrales Prinzip:

> Meine Daten gehören mir.

Audioaufnahmen werden lokal erzeugt und erst anschließend
zum eigenen Server übertragen.

NC-PoRe ermöglicht verteilte Zusammenarbeit, ohne die Kontrolle
über eigene Daten und Produktionsabläufe abzugeben.

---

# Completed

## Project Setup

Completed:

* GitHub Repository erstellt
* AGPL-3.0 Lizenz gewählt
* Dokumentationsstruktur eingerichtet
* ADR-Struktur etabliert

---

## Vision and Requirements

Completed:

* Projektvision dokumentiert
* funktionale Anforderungen definiert
* zentrale Benutzergruppen und Nutzungsszenarien beschrieben

---

# Architecture Decisions

Die Architekturgrundlagen wurden durch folgende ADRs definiert:

## Early Architecture Foundation

### ADR-001

Local Recording

Grundentscheidung für lokale Aufnahme.

### ADR-002

Audio Format and Track Concept

Getrennte hochwertige Monospuren als Produktionsbasis.

### ADR-003

Local Chunk Storage

Chunk-basierte lokale Speicherung.

### ADR-004

Upload After Recording

Upload erst nach Abschluss der Aufnahme.

### ADR-005

Consent and Recording Transparency

Transparente Aufnahme und dokumentierte Zustimmung.

### ADR-006

Role-Based Access Control

Rollenmodell für unterschiedliche Nutzergruppen.

### ADR-007

Open Formats and Interoperability

Offene Formate und freie Werkzeugwahl.

### ADR-008

Client Architecture

Modulare Recorder-Architektur mit professionellen
und vereinfachten Clients.

---

# Extended Architecture Foundation

### ADR-022

Modular Architecture

Definition einer modularen und erweiterbaren Systemstruktur.

### ADR-023

Internationalization and Localization Strategy

Strategie für Mehrsprachigkeit und Lokalisierung.

### ADR-024

Client Architecture and Platform Strategy

Strategie für plattformübergreifende Clients.

### ADR-025

Product Identity and Naming

Definition der Produktidentität und Namensstrategie.

### ADR-026

Session Data and Storage Architecture

Definition von Production Sessions und Storage-Strukturen.

### ADR-027

Core Architecture and Module Boundaries

Definition der Core-Verantwortung und Modulgrenzen.

### ADR-028

API Design Principles

Grundsätze für Kommunikation zwischen Systemkomponenten.

### ADR-029

Distributed Recording Architecture

Definition des Local Recording First Prinzips.

### ADR-030

Synchronization Strategy for Distributed Recordings

Trennung von Control Synchronization und Media Synchronization.

### ADR-031

Identity, Authentication and User Roles

Definition von Identität, Authentifizierung und Rollenmodell.

### ADR-032

Auditability and Activity History

Definition von nachvollziehbarer Produktionshistorie.

---

# Current Architecture Principles

NC-PoRe folgt diesen Grundsätzen:

* lokale Aufnahme
* keine Audioabhängigkeit vom Netzwerk
* offene Formate
* getrennte Audiospuren
* transparente Zustimmung
* rollenbasierte Rechte
* selbsthostbare Infrastruktur
* Erweiterbarkeit
* Production Session als zentrale fachliche Einheit
* Core als Autorität für Geschäftslogik
* API- und Event-basierte Kommunikation
* Local Recording First
* Trennung von Control Synchronization und Media Synchronization
* Identität getrennt von Rollen und Berechtigungen
* Activity History als Produktionsgedächtnis
* Zusammenarbeit steht im Mittelpunkt

---

# Current Architecture Model

NC-PoRe basiert auf folgenden zentralen Konzepten:

```text
Production Session

        |
        |
        +-- Participants
        |
        +-- Roles
        |
        +-- Recordings
        |
        +-- Assets
        |
        +-- Synchronization Metadata
        |
        +-- Activity History
```

Die Production Session bildet die fachliche Klammer
für eine gemeinsame Produktion.

---

# Technical Direction

Aktuelle technische Richtung:

## Clients

* plattformspezifische Clients
* lokale Aufnahme
* lokale Verarbeitung
* Synchronisation mit zentraler Umgebung

## Core

* zentrale Geschäftslogik
* Verwaltung fachlicher Zustände
* Berechtigungsprüfung
* Session-Management

## Storage

* Nextcloud-basierte Speicherung
* selbsthostbare Infrastruktur
* offene Datenhaltung

## Communication

* API-basierte Kommunikation
* Event-orientierte Architektur
* getrennte Steuerungs- und Mediendaten

---

# Next Steps

## Implementation Planning

Geplante nächste Schritte:

* technische Kernarchitektur
* Repository-Struktur für den Core
* Technologieentscheidungen
* erste Modulstruktur
* erstes vertikales MVP
* technischer Prototyp

---

## Future Architecture Decisions

Mögliche nächste ADRs:

* Implementation Architecture
* Database and Persistence Strategy
* Client-Core Communication
* Deployment Architecture
* Build and Release Strategy
* Testing Strategy

---

# Current Status Summary

NC-PoRe verfügt über:

* definierte Vision
* dokumentierte Anforderungen
* grundlegende Architektur
* 32 dokumentierte Architekturentscheidungen
* definiertes Session-Modell
* verteilte Recording-Strategie
* Synchronisationsstrategie
* Identitäts- und Rollenmodell
* nachvollziehbare Produktionshistorie

Die Architekturphase wurde bewusst abgeschlossen.

Die Implementierungsphase wurde noch nicht begonnen.

Der nächste Schritt ist die technische Umsetzungsvorbereitung.

---

# Current Transition

NC-PoRe befindet sich am Übergang von der Architekturdefinition
zur technischen Umsetzungsvorbereitung.

Die nächsten Arbeiten konzentrieren sich auf:

* technische Projektgrundlage
* Repository- und Modulstruktur
* Technologieentscheidungen
* Entwicklungsumgebung
* erste technische Prototypen

Die eigentliche Implementierung produktiver Funktionen wurde bewusst
noch nicht begonnen.

---

# Milestone

## Architecture Foundation Complete

Date:

2026-07-24

NC-PoRe verfügt nun über eine belastbare Grundlage für die nächste Entwicklungsphase.

Die Architektur beschreibt nicht nur Softwarekomponenten,
sondern die Zusammenarbeit von Menschen, Geräten und Produktionsprozessen.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# Project Phase

Current phase:

## Foundation and Architecture Definition Completed

NC-PoRe has completed the fundamental concept and architecture definition phase.

The focus was on:

* architecture decisions
* requirements
* data model and session concept
* project structure
* FOSS foundations
* distribution and collaboration
* security and role models

The fundamental architecture decisions are documented.

The next phase is technical implementation preparation.

---

# Project Vision

NC-PoRe is a self-hostable open-source platform
for professional podcast recording and production.

Central principle:

> My data belongs to me.

Audio recordings are created locally and transferred
to the user's own server afterwards.

NC-PoRe enables distributed collaboration without giving up
control over personal data and production workflows.

---

# Completed

## Project Setup

Completed:

* GitHub repository created
* AGPL-3.0 license selected
* documentation structure established
* ADR structure established

---

## Vision and Requirements

Completed:

* project vision documented
* functional requirements defined
* user groups and usage scenarios described

---

# Architecture Decisions

The architectural foundation was defined through the following ADRs:

## Early Architecture Foundation

### ADR-001

Local Recording

Fundamental decision for local recording.

### ADR-002

Audio Format and Track Concept

High-quality separate mono tracks as production basis.

### ADR-003

Local Chunk Storage

Chunk-based local storage.

### ADR-004

Upload After Recording

Upload only after recording completion.

### ADR-005

Consent and Recording Transparency

Transparent recording and documented consent.

### ADR-006

Role-Based Access Control

Role model for different user groups.

### ADR-007

Open Formats and Interoperability

Open formats and free tool choice.

### ADR-008

Client Architecture

Modular recorder architecture with professional
and simplified clients.

---

# Extended Architecture Foundation

### ADR-022

Modular Architecture

Definition of a modular and extensible system structure.

### ADR-023

Internationalization and Localization Strategy

Strategy for multilingual support and localization.

### ADR-024

Client Architecture and Platform Strategy

Strategy for cross-platform clients.

### ADR-025

Product Identity and Naming

Definition of product identity and naming strategy.

### ADR-026

Session Data and Storage Architecture

Definition of Production Sessions and storage structures.

### ADR-027

Core Architecture and Module Boundaries

Definition of Core responsibilities and module boundaries.

### ADR-028

API Design Principles

Principles for communication between system components.

### ADR-029

Distributed Recording Architecture

Definition of the Local Recording First principle.

### ADR-030

Synchronization Strategy for Distributed Recordings

Separation of Control Synchronization and Media Synchronization.

### ADR-031

Identity, Authentication and User Roles

Definition of identity, authentication and role model.

### ADR-032

Auditability and Activity History

Definition of traceable production history.

---

# Current Status Summary

NC-PoRe has:

* a defined vision
* documented requirements
* a fundamental architecture
* 32 documented architecture decisions
* a defined session model
* a distributed recording strategy
* a synchronization strategy
* an identity and role model
* a traceable production history

The architecture phase has been deliberately completed.

The implementation phase has not started yet.

The next step is technical implementation preparation.

---

# Current Transition

NC-PoRe is transitioning from architecture definition
to technical implementation preparation.

The next activities focus on:

* technical project foundation
* repository and module structure
* technology decisions
* development environment
* first technical prototypes

The actual implementation of production features has deliberately
not started yet.

---

# Milestone

## Architecture Foundation Complete

Date:

2026-07-24

NC-PoRe now has a solid foundation for the next development phase.

The architecture describes not only software components,
but also the collaboration of people, devices and production processes.
