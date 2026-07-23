# NC-PoRe Project Status

## Last Update

2026-07-23

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

# Milestone

## Architecture Foundation Complete

Date:

2026-07-23

NC-PoRe verfügt nun über eine belastbare Grundlage für die nächste Entwicklungsphase.

Die Architektur beschreibt nicht nur Softwarekomponenten,
sondern die Zusammenarbeit von Menschen, Geräten und Produktionsprozessen.
