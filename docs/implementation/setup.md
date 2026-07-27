# NC-PoRe Technical Foundation

* Version: 1.0
* Date: 2026-07-24

---

# Deutsch ([English version below](#english-version))

---

# 1. Zweck dieses Dokuments

Dieses Dokument beschreibt die technische Grundlage,
die vor Beginn der eigentlichen Implementierung definiert werden soll.

Es verbindet die vorhandenen Architekturentscheidungen mit den
notwendigen technischen Vorbereitungsschritten.

Dieses Dokument ersetzt keine Architecture Decision Records (ADRs).

Technische Entscheidungen mit langfristiger Auswirkung werden weiterhin
über ADRs dokumentiert.

Ziel dieses Dokuments ist:

* technische Orientierung
* Strukturierung der nächsten Schritte
* Sichtbarkeit offener Entscheidungen
* gemeinsame Grundlage für die Implementierungsphase

---

# 2. Technische Grundidee

NC-PoRe basiert auf einer verteilten Architektur.

Die zentrale Idee:

> Aufnahme und Verarbeitung entstehen lokal.
> Verwaltung und Zusammenarbeit erfolgen über eine zentrale Umgebung.

Das System verbindet daher:

* lokale Clients
* zentrale Geschäftslogik
* selbsthostbare Speicherung
* kontrollierte Synchronisation

---

# 3. Technische Hauptkomponenten

NC-PoRe wird in folgende technische Bereiche gegliedert:

---

## 3.1 Client

Der Client ist für lokale Produktionsaufgaben verantwortlich.

Verantwortlichkeiten:

* lokale Audioaufnahme
* lokale Verarbeitung
* lokale Zwischenspeicherung
* Verwaltung lokaler Zustände
* Synchronisation mit der zentralen Umgebung

Grundprinzip:

Die Aufnahme ist nicht von einer permanenten Netzwerkverbindung abhängig.

---

## 3.2 Core

Der Core bildet die zentrale fachliche Autorität.

Verantwortlichkeiten:

* Verwaltung von Production Sessions
* fachliche Geschäftslogik
* Verwaltung von Zuständen
* Rollen und Berechtigungen
* Validierung von Aktionen

Der Core entscheidet über fachliche Regeln.

Clients stellen Funktionen bereit, sind aber nicht die alleinige Quelle
für fachliche Wahrheit.

---

## 3.3 Storage

Der Storage-Bereich verwaltet dauerhaft gespeicherte Informationen.

Verantwortlichkeiten:

* Audio Assets
* Produktionsdaten
* Metadaten
* Session-bezogene Informationen
* langfristige Speicherung

Grundprinzip:

Daten bleiben unter Kontrolle der Nutzer.

---

## 3.4 Communication Layer

Der Communication Layer verbindet die Systembestandteile.

Verantwortlichkeiten:

* API-Kommunikation
* Event-basierte Kommunikation
* Synchronisationsinformationen
* technische Schnittstellen

Steuerungsinformationen und Mediendaten werden getrennt behandelt.

---

# 4. Bereits getroffene technische Richtungen

Folgende technische Richtungen sind bereits durch Architekturentscheidungen festgelegt:

* lokale Aufnahme als Grundprinzip
* keine Abhängigkeit von permanenter Netzwerkverbindung
* offene Audioformate
* getrennte Audiospuren
* selbsthostbare Infrastruktur
* Nextcloud-basierte Speicherung
* API-basierte Kommunikation
* Event-orientierte Kommunikation
* Trennung von Control Synchronization und Media Synchronization

---

# 5. Technische Prinzipien

Technische Entscheidungen werden nach folgenden Kriterien getroffen:

## Wartbarkeit

Die Lösung soll langfristig verständlich und betreibbar bleiben.

## Offenheit

Standards und offene Schnittstellen werden bevorzugt.

## Einfachheit

Komplexität wird nur eingeführt, wenn sie einen konkreten Nutzen bringt.

## Erweiterbarkeit

Die Architektur soll zukünftige Entwicklungen ermöglichen.

## Nachvollziehbarkeit

Entscheidungen sollen dokumentiert und begründet werden.

---

# 6. Offene technische Entscheidungen

Vor Beginn der Implementierung müssen unter anderem folgende Bereiche geklärt werden:

## Programmiersprachen

Zu definieren:

* Client-Technologien
* Core-Technologie
* gemeinsame Bibliotheken

---

## Frameworks und Laufzeitumgebung

Zu definieren:

* verwendete Frameworks
* Plattformintegration
* Entwicklungswerkzeuge

---

## Repository-Struktur

Zu definieren:

* Monorepository oder mehrere Repositories
* Modulstruktur
* Abhängigkeiten
* Build-Struktur

---

## Datenhaltung

Zu definieren:

* Datenbankstrategie
* lokale Speicherung
* zentrale Speicherung
* Migrationen
* Backup-Strategie

---

## Build und Release

Zu definieren:

* Build-System
* Paketierung
* Versionierung
* Release-Prozess

---

## Testing

Zu definieren:

* Teststrategie
* automatisierte Tests
* Integrationsprüfungen
* Qualitätssicherung

---

# 7. Erste Implementierungsstrategie

Die Implementierung soll schrittweise erfolgen.

Bevorzugter Ansatz:

## Vertikales MVP

Nicht einzelne technische Schichten isoliert bauen,
sondern einen kleinen vollständigen Produktionsablauf ermöglichen.

Beispiel:

* Client kann aufnehmen
* Session kann erstellt werden
* Daten können gespeichert werden
* zentrale Verwaltung funktioniert

Danach erfolgt schrittweise Erweiterung.

---

# 8. Grundsatz

NC-PoRe wird technisch nicht durch maximale Komplexität definiert.

Die technische Qualität entsteht durch:

* klare Architektur
* bewusste Entscheidungen
* verständliche Komponenten
* saubere Dokumentation

Technik dient dem Produktionsprozess.

Nicht der Produktionsprozess der Technik.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# 1. Purpose of this Document

This document describes the technical foundation
that should be defined before the actual implementation begins.

It connects the existing architecture decisions with the
necessary technical preparation steps.

This document does not replace Architecture Decision Records (ADRs).

Technical decisions with long-term impact continue to be documented
through ADRs.

The purpose of this document is:

* technical orientation
* structuring the next steps
* visibility of open decisions
* a shared foundation for the implementation phase

---

# 2. Technical Concept

NC-PoRe is based on a distributed architecture.

The central idea:

> Recording and processing happen locally.
> Management and collaboration happen through a central environment.

The system therefore connects:

* local clients
* central business logic
* self-hostable storage
* controlled synchronization

---

# 3. Main Technical Components

NC-PoRe is structured into the following technical areas:

---

## 3.1 Client

The client is responsible for local production tasks.

Responsibilities:

* local audio recording
* local processing
* local temporary storage
* management of local states
* synchronization with the central environment

Core principle:

Recording does not depend on a permanent network connection.

---

## 3.2 Core

The Core represents the central domain authority.

Responsibilities:

* management of Production Sessions
* domain business logic
* state management
* roles and permissions
* validation of actions

The Core defines domain rules.

Clients provide functionality but are not the sole source
of domain truth.

---

## 3.3 Storage

The storage area manages permanently stored information.

Responsibilities:

* audio assets
* production data
* metadata
* session-related information
* long-term storage

Core principle:

Data remains under user control.

---

## 3.4 Communication Layer

The Communication Layer connects the system components.

Responsibilities:

* API communication
* event-based communication
* synchronization information
* technical interfaces

Control information and media data are treated separately.

---

# 4. Established Technical Directions

The following technical directions have already been established through
architecture decisions:

* local recording as a core principle
* no dependency on permanent network connection
* open audio formats
* separate audio tracks
* self-hostable infrastructure
* Nextcloud-based storage
* API-based communication
* event-oriented communication
* separation of Control Synchronization and Media Synchronization

---

# 5. Technical Principles

Technical decisions are evaluated according to the following criteria:

## Maintainability

Solutions should remain understandable and operable long term.

## Openness

Standards and open interfaces are preferred.

## Simplicity

Complexity is introduced only when it provides concrete value.

## Extensibility

The architecture should enable future development.

## Traceability

Decisions should be documented and justified.

---

# 6. Open Technical Decisions

Before implementation begins, the following areas need clarification:

## Programming Languages

To be defined:

* client technologies
* core technology
* shared libraries

---

## Frameworks and Runtime Environment

To be defined:

* frameworks
* platform integration
* development tools

---

## Repository Structure

To be defined:

* monorepository or multiple repositories
* module structure
* dependencies
* build structure

---

## Data Management

To be defined:

* database strategy
* local storage
* central storage
* migrations
* backup strategy

---

## Build and Release

To be defined:

* build system
* packaging
* versioning
* release process

---

## Testing

To be defined:

* testing strategy
* automated tests
* integration tests
* quality assurance

---

# 7. Initial Implementation Strategy

Implementation should happen incrementally.

Preferred approach:

## Vertical MVP

Instead of building isolated technical layers,
create a small complete production workflow.

Example:

* client can record
* session can be created
* data can be stored
* central management works

Afterwards, functionality is expanded step by step.

---

# 8. Principle

NC-PoRe is not defined by maximum technical complexity.

Technical quality comes from:

* clear architecture
* conscious decisions
* understandable components
* clean documentation

Technology serves the production process.

Not the other way around.
