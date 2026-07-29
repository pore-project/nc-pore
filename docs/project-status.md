# NC-PoRe Project Status

* Version: 1.3
* Date: 2026-07-29

---

# Deutsch ([English version below](#english-version))

---

# Project Phase

Current phase:

## Architecture Foundation Completed — Implementation Architecture Defined

NC-PoRe hat die grundlegende Konzeptions- und Architekturphase abgeschlossen.

Die technische Implementierungsarchitektur wurde definiert.

Der Fokus lag auf:

* Architekturentscheidungen
* Anforderungen
* Datenmodell und Session-Konzept
* Projektstruktur
* FOSS-Grundlagen
* Verteilung und Zusammenarbeit
* Sicherheits- und Rollenmodellen
* technischer Struktur der Implementierung

Die grundlegenden Architekturentscheidungen und Implementierungsprinzipien sind dokumentiert.

Die nächste Phase ist die technische Umsetzung.

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

### ADR-009

Track Synchronisation

Definition der Synchronisation zwischen getrennten Audiospuren.

### ADR-010

Core Data Model

Definition des zentralen Datenmodells.

### ADR-011

Security Model

Definition der grundlegenden Sicherheitsarchitektur.

### ADR-012

Export Architecture

Definition der Exportstruktur und Exportverantwortlichkeiten.

### ADR-013

Technology Stack

Grundsätze zur Auswahl des Technologie-Stacks.

### ADR-014

Development Environment and Toolchain

Definition der Entwicklungsumgebung und Werkzeugkette.

### ADR-015

Initial Architecture of the NC-PoRe Recorder Client

Definition der grundlegenden Architektur des Recorder Clients.

### ADR-016

Audio Layer Technology Selection

Auswahlprinzipien für die Audio-Schicht.

### ADR-017

Audio Backend Library Selection

Auswahlprinzipien für Audio-Backend-Bibliotheken.

### ADR-018

Recorder Data Flow and Processing Pipeline

Definition des Datenflusses und der Verarbeitungspipeline
des Recorders.

### ADR-019

Recording Session Data Model

Definition des Recording Session Datenmodells.

### ADR-020

Metadata Data Model

Definition des Metadatenmodells.

### ADR-021

Internal Data Structures and Serialization Format

Definition interner Datenstrukturen und
Serialisierungsformate.

---

# Extended Architecture Foundation

### ADR-022

Modular Architecture and Provider Design

Definition einer modularen und erweiterbaren Systemstruktur
sowie Provider-Grenzen.

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

### ADR-033

Core Architecture

Definition des Core als fachliche Autorität
und Festlegung der Verantwortlichkeiten.

### ADR-034

Implementation Architecture

Definition der grundlegenden Prinzipien
für die technische Umsetzung.

### ADR-035

Domain Lifecycle and State Transition Management

Definition des Musters für fachliche Lebenszyklen
und kontrollierte Zustandsübergänge innerhalb des Core.

### ADR-036

Development Workflow and Source of Truth

Definition des nachvollziehbaren Entwicklungsprozesses
und der Repository-Struktur als technische Quelle der Wahrheit.

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
* Domain Authority liegt ausschließlich beim Core
* Implementierungsarchitektur folgt der fachlichen Architektur
* technische Details bleiben von der Domäne getrennt
* fachliche Lebenszyklen werden explizit modelliert
* Entwicklungsprozesse folgen einem zustandsorientierten Workflow
* Repository-Inhalt ist die technische Quelle der Wahrheit

---

# Current Architecture Model

NC-PoRe basiert auf folgenden zentralen Konzepten:

```
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

## Technical Implementation

Geplante nächste Schritte:

* technische Projektgrundlage
* Repository-Struktur definieren
* Modulstruktur umsetzen
* Technologieentscheidungen treffen
* Entwicklungsumgebung vorbereiten
* erstes vertikales MVP entwickeln
* technische Prototypen erstellen
* Persistenz- und Speicherstrategie
* technische Umsetzung der Core-Struktur

---

## Future Architecture Decisions

Mögliche nächste ADRs:

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
* 36 dokumentierte Architekturentscheidungen
* definiertes Session-Modell
* verteilte Recording-Strategie
* Synchronisationsstrategie
* Identitäts- und Rollenmodell
* nachvollziehbare Produktionshistorie
* definierte Implementierungsarchitektur
* definiertes Domain Lifecycle Modell
* definierten Entwicklungsworkflow
* klare technische Architekturprinzipien

Die Architekturphase wurde bewusst abgeschlossen.

Die Implementierungsphase beginnt auf Basis der definierten Architektur.

Der nächste Schritt ist die technische Umsetzung.

---

# Current Transition

NC-PoRe befindet sich am Übergang von der Architekturphase
zur technischen Umsetzung.

Die Architektur beschreibt nun sowohl die fachlichen Grundlagen
als auch die Prinzipien für deren technische Realisierung.

Die nächsten Arbeiten konzentrieren sich auf:

* technische Projektgrundlage
* Repository- und Modulstruktur
* Entwicklungsumgebung
* konkrete Technologieentscheidungen
* erste technische Prototypen
* Persistenz- und Speicherstrategie
* technische Umsetzung der Core-Struktur

Die Entwicklung produktiver Funktionen beginnt schrittweise
auf Basis der definierten Architektur.

---

# Milestone

## Architecture Foundation Complete

Date:

2026-07-24

NC-PoRe verfügt nun über eine belastbare Grundlage
für die nächste Entwicklungsphase.

Die Architektur beschreibt nicht nur Softwarekomponenten,
sondern die Zusammenarbeit von Menschen, Geräten
und Produktionsprozessen.

Die technische Umsetzung beginnt auf Basis dieser Grundlage.

# English Version ([Deutsche Version oben](#deutsch))

---

# Project Phase

Current phase:

## Architecture Foundation Completed — Implementation Architecture Defined

NC-PoRe has completed the fundamental concept
and architecture definition phase.

The implementation architecture has been defined.

The focus was on:

* architecture decisions
* requirements
* data model and session concept
* project structure
* FOSS foundations
* distribution and collaboration
* security and role models
* technical implementation structure

The fundamental architecture decisions and implementation principles
are documented.

The next phase is technical implementation.

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

* GitHub Repository created
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

### ADR-033

Core Architecture

Definition of the Core as domain authority
and responsibility boundaries.

### ADR-034

Implementation Architecture

Definition of fundamental principles
for technical implementation.

### ADR-035

Domain Lifecycle and State Transition Management

Definition of domain lifecycle handling
and state transition responsibilities.

### ADR-036

Development Workflow and Source of Truth

Definition of the development workflow,
verification process and repository source of truth.

---

# Current Architecture Principles

NC-PoRe follows these principles:

* local recording
* no dependency of audio production on network availability
* open formats
* separate audio tracks
* transparent consent
* role-based permissions
* self-hostable infrastructure
* extensibility
* Production Session as central domain entity
* Core as authority for business logic
* API- and event-based communication
* Local Recording First
* separation of Control Synchronization and Media Synchronization
* identity separated from roles and permissions
* Activity History as production memory
* collaboration as a central principle
* Domain Authority exclusively located in the Core
* implementation architecture follows domain architecture
* technical details remain separated from the domain
* domain lifecycles are explicitly modeled
* development follows a state-oriented workflow
* repository content is the technical source of truth

---

# Current Architecture Model

NC-PoRe is based on the following central concepts:

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

The Production Session forms the domain framework
for a shared production.

---

# Technical Direction

Current technical direction:

## Clients

* platform-specific clients
* local recording
* local processing
* synchronization with central environment

## Core

* central business logic
* management of domain states
* permission validation
* session management

## Storage

* Nextcloud-based storage
* self-hostable infrastructure
* open data handling

## Communication

* API-based communication
* event-oriented architecture
* separated control and media data

---

# Next Steps

## Technical Implementation

Planned next steps:

* technical project foundation
* definition of repository structure
* implementation of module structure
* technology decisions
* preparation of development environment
* first vertical MVP
* technical prototypes

---

## Future Architecture Decisions

Possible next ADRs:

* Database and Persistence Strategy
* Client-Core Communication
* Deployment Architecture
* Build and Release Strategy
* Testing Strategy

---

# Current Status Summary

NC-PoRe has:

* a defined vision
* documented requirements
* a fundamental architecture
* 36 documented architecture decisions
* a defined session model
* a distributed recording strategy
* a synchronization strategy
* an identity and role model
* a traceable production history
* a defined implementation architecture
* a defined domain lifecycle model
* a defined development workflow
* clear technical architecture principles

The architecture phase has been deliberately completed.

The implementation phase starts based on the defined architecture.

The next step is technical implementation.

---

# Current Transition

NC-PoRe is transitioning from architecture definition
to technical implementation.

The architecture now describes both domain foundations
and principles for their technical realization.

The next activities focus on:

* technical project foundation
* repository and module structure
* development environment
* concrete technology decisions
* first technical prototypes

The development of production features starts step by step
based on the defined architecture.

---

# Milestone

## Architecture Foundation Complete

Date:

2026-07-24

NC-PoRe now has a solid foundation
for the next development phase.

The architecture describes not only software components,
but also collaboration between people, devices
and production processes.

