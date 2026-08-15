# NC-PoRe Roadmap

- Version: 1.2
- Date: 2026-08-14

---

# Deutsch (English version below)

## Einleitung

Diese Roadmap beschreibt die langfristige Entwicklung von NC-PoRe.

Sie dient als Orientierung und beschreibt die geplante Richtung des Projekts.  
Sie ist keine starre Verpflichtung und kann durch technische Erkenntnisse, Nutzeranforderungen oder neue Entwicklungen angepasst werden.

Die zentrale Idee:

> NC-PoRe entwickelt nicht nur einen Recorder. NC-PoRe entwickelt eine offene Plattform für Medien-Sessions.

---

# Entwicklungsprinzip

NC-PoRe wird schrittweise entwickelt.

Jede Version soll einen konkreten Nutzen bieten:

- für Anwender
- für Entwickler
- für die Community

Neue Funktionen werden nicht nur nach technischer Machbarkeit bewertet, sondern nach ihrem tatsächlichen Mehrwert.

Dabei gilt:

> Komplexität soll innerhalb des Systems gelöst werden und nicht beim Anwender entstehen.

---

# Version 1.x – Die Session-Basis

## Ziel

Aufbau der kleinsten sinnvollen Version der zukünftigen NC-PoRe-Plattform.

V1 ist kein isolierter Recorder, sondern die Grundlage für verteilte Medien-Sessions.

---

## Schwerpunkte

- Session-basierte Architektur
- stabile Audioaufnahme
- konfigurierbare Aufnahmeparameter mit einem vorgeschlagenen Standardprofil gemäß ADR-002
- Teilnehmer- und Geräteverwaltung
- Metadatenverwaltung
- lokale Speicherung
- erste Exportmöglichkeiten
- modulare Architektur
- klare Provider-Schnittstellen
- umfangreiche Dokumentation

---

## Plattformstrategie

Die Architektur berücksichtigt von Anfang an:

- Linux
- Windows
- macOS
- iOS
- Android

Nicht jede Plattform muss sofort denselben Funktionsumfang besitzen.

Entscheidend ist:

Die Plattformen sind Teil des Designs und keine spätere Erweiterung.

---

## Integrationen

V1 schafft die Grundlagen für:

- lokale Clients
- mobile Clients
- entfernte Teilnehmer
- zukünftige Conference Provider

Beispiele:

- Nextcloud Talk
- BigBlueButton
- Jitsi
- weitere Systeme

---

# Version 2.x – Kollaborative Sessions

## Ziel

NC-PoRe ermöglicht echte verteilte Zusammenarbeit.

---

## Schwerpunkte

- vollständige mobile Teilnahme
- Synchronisation mehrerer Teilnehmer
- verteilte Aufnahme-Szenarien
- Integration erster Conference Provider
- Nextcloud Talk Integration
- verbesserte Session-Verwaltung

---

## Beispiel-Szenario

Eine Session kann bestehen aus:

- Host auf macOS
- Teilnehmer auf Linux
- Teilnehmer auf Windows
- Gast über iOS
- Gast über Android

Alle arbeiten gemeinsam an einer Session.

---

# Version 3.x – Offene Plattform

## Ziel

NC-PoRe wird unabhängig von einzelnen Plattformen und Diensten.

---

## Schwerpunkte

- weitere Cloud-Anbieter
- unabhängige Storage Provider
- WebDAV-Unterstützung
- offene APIs
- Plugin-System
- Community-Erweiterungen

---

## Vision

NC-PoRe ist nicht nur ein Nextcloud-Werkzeug.

NC-PoRe ist eine offene Medienplattform.

---

# Version 4.x – Erweiterte Medien

## Ziel

Erweiterung von Audio-Sessions zu vollständigen Medien-Sessions.

---

## Schwerpunkte

- Videoaufnahme
- Bildschirmaufnahme
- kombinierte Audio-/Video-Sessions
- professionelle Produktionsworkflows
- erweiterte Exportmöglichkeiten

Die Architektur aus V1 ermöglicht diese Erweiterungen ohne grundlegende Neuentwicklung.

---

# Version 5.x – Professionelle Distributed Production

## Ziel

NC-PoRe ermöglicht professionelle verteilte Medienproduktion.

---

## Schwerpunkte

- mehrere Recording Nodes
- intelligente Synchronisation
- große verteilte Produktionen
- komplexe Teilnehmerstrukturen
- professionelle Workflows

---

## Beispiel

Eine Produktion mit:

- mehreren Hosts
- verschiedenen Betriebssystemen
- mobilen Teilnehmern
- unterschiedlichen Aufnahmequellen

wird als eine gemeinsame Session verwaltet.

---

# Eigenständiges Podcast-Hosting

## Produktidee

Neben NC-PoRe soll langfristig ein eigenständiges Podcast-Hosting-Produkt entstehen.

Das Podcast-Hosting ist kein Bestandteil von NC-PoRe, sondern ein eigenständiges Produkt mit eigener Produktgrenze, eigener Entwicklung und eigenständigem Betrieb.

---

## Produktgrenzen

Beide Produkte sollen unabhängig voneinander funktionieren:

- NC-PoRe muss vollständig ohne das Podcast-Hosting betrieben werden können.
- Das Podcast-Hosting muss vollständig ohne NC-PoRe betrieben werden können.
- Keines der beiden Produkte setzt die Installation oder Existenz des jeweils anderen voraus.
- Beide Produkte können unabhängig entwickelt, versioniert und betrieben werden.

NC-PoRe bleibt dabei auf die Produktion und Verarbeitung von Medien-Sessions ausgerichtet.

Das Podcast-Hosting konzentriert sich auf die Bereitstellung und Distribution von Podcast-Inhalten.

---

## Zusammenspiel

Zwischen beiden Produkten ist eine **vorgesehene Integration über definierte Schnittstellen** vorgesehen.

Diese Integration soll:

- die Übergabe von Podcast-Inhalten und zugehörigen Metadaten ermöglichen,
- die jeweiligen Produktgrenzen erhalten,
- unabhängig von einer gemeinsamen Codebasis funktionieren,
- auch eine Integration mit anderen Produktions- bzw. Hosting-Systemen ermöglichen.

Die Schnittstellen sollen deshalb fachlich möglichst neutral gestaltet werden.

NC-PoRe soll nicht auf dieses eine Hosting-Produkt festgelegt sein.

Ebenso soll das Podcast-Hosting Inhalte auch aus anderen Produktionssystemen aufnehmen können.

Die konkrete API-, Protokoll- und Integrationsarchitektur wird in einem späteren eigenständigen Architekturvorhaben festgelegt.

---

## Roadmap-Einordnung

Das Podcast-Hosting ist ein langfristiges eigenständiges Produktvorhaben.

Es ist **nicht Bestandteil des aktuellen NC-PoRe-Implementierungsumfangs** und soll nicht zu einer technischen Abhängigkeit zwischen den beiden Produkten führen.

---

# Langfristige Vision

NC-PoRe soll Menschen ermöglichen, hochwertige Medieninhalte einfach, offen und plattformübergreifend zu erstellen.

Mögliche zukünftige Entwicklungen:

- KI-gestützte Unterstützung
- automatische Transkription
- Übersetzungen
- intelligente Zusammenfassungen
- Assistenzfunktionen
- professionelle Produktionsumgebungen
- Community-basierte Erweiterungen

Technologie bleibt dabei ein Werkzeug.

Menschen und ihre Inhalte stehen im Mittelpunkt.

---

# English

## Introduction

This roadmap describes the long-term development direction of NC-PoRe.

It provides guidance and may change due to technical insights, user requirements or new developments.

The central idea:

> NC-PoRe does not only build a recorder. NC-PoRe builds an open platform for media sessions.

---

# Development Principle

NC-PoRe is developed step by step.

Each version should provide concrete value:

- for users
- for developers
- for the community

Features are evaluated not only by technical feasibility, but by their actual benefit.

The guiding principle:

> Complexity should be solved inside the system, not transferred to the user.

---

# Version 1.x – Session Foundation

## Goal

Building the smallest meaningful version of the future NC-PoRe platform.

V1 is not an isolated recorder, but the foundation for distributed media sessions.

---

## Focus

- session-based architecture
- stable audio recording
- configurable recording parameters with a suggested default profile according to ADR-002
- participant and device management
- metadata management
- local storage
- first export capabilities
- modular architecture
- clear provider interfaces
- extensive documentation

---

## Platform Strategy

The architecture considers from the beginning:

- Linux
- Windows
- macOS
- iOS
- Android

Not every platform needs identical functionality immediately.

The important principle:

Platforms are part of the design, not later additions.

---

# Version 2.x – Collaborative Sessions

## Goal

NC-PoRe enables real distributed collaboration.

---

## Focus

- full mobile participation
- synchronization of multiple participants
- distributed recording scenarios
- first conference provider integrations
- Nextcloud Talk integration
- improved session management

---

# Version 3.x – Open Platform

## Goal

NC-PoRe becomes independent from individual platforms and services.

---

## Focus

- additional cloud providers
- independent storage providers
- WebDAV support
- open APIs
- plugin system
- community extensions

---

# Version 4.x – Extended Media

## Goal

Expanding audio sessions into complete media sessions.

---

## Focus

- video recording
- screen capture
- combined audio/video sessions
- professional production workflows
- advanced exports

---

# Version 5.x – Professional Distributed Production

## Goal

NC-PoRe enables professional distributed media production.

---

## Focus

- multiple recording nodes
- intelligent synchronization
- large distributed productions
- complex participant structures
- professional workflows

---

# Independent Podcast Hosting

## Product Idea

In the long term, a separate podcast hosting product shall be developed alongside NC-PoRe.

The podcast hosting product is not part of NC-PoRe, but an independent product with its own product boundary, development and operation.

---

## Product Boundaries

Both products shall be usable independently:

- NC-PoRe must operate fully without the podcast hosting product.
- The podcast hosting product must operate fully without NC-PoRe.
- Neither product requires the installation or existence of the other.
- Both products can be developed, versioned and operated independently.

NC-PoRe remains focused on the production and processing of media sessions.

The podcast hosting product focuses on the hosting and distribution of podcast content.

---

## Integration

A **planned integration through defined interfaces** shall enable cooperation between the two products.

This integration shall:

- enable the transfer of podcast content and associated metadata,
- preserve the respective product boundaries,
- work independently of a shared codebase,
- also allow integration with other production and hosting systems.

The interfaces should therefore be designed to be as domain-neutral as reasonably possible.

NC-PoRe should not be tied to this particular hosting product.

Likewise, the podcast hosting product should be able to receive content from other production systems.

The concrete API, protocol and integration architecture will be defined in a separate future architecture initiative.

---

## Roadmap Position

Podcast hosting is a long-term independent product initiative.

It is **not part of the current NC-PoRe implementation scope** and shall not create a technical dependency between the two products.

---

# Long-Term Vision

NC-PoRe enables people to create high-quality media content easily, openly and across platforms.

Possible future developments:

- AI-assisted workflows
- automatic transcription
- translations
- intelligent summaries
- assistance features
- professional production environments
- community-driven extensions

Technology remains a tool.

People and their content remain the focus.
