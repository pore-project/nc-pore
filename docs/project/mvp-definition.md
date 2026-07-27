# NC-PoRe MVP Definition

* Version: 1.0
* Date: 2026-07-24

---

# Deutsch ([English version below](#english-version))

---

# 1. Zweck dieses Dokuments

Dieses Dokument beschreibt die erste technische Ausbaustufe
von NC-PoRe.

Das MVP (Minimum Viable Product) definiert den kleinsten vollständigen
Produktionsablauf, der die zentralen Architekturprinzipien von NC-PoRe
technisch nachweist.

Das MVP ist kein vollständiges Produkt.

Es ist jedoch auch kein Wegwerf-Prototyp.

Die Implementierung des MVP soll auf den langfristigen Architekturprinzipien
von NC-PoRe aufbauen.

---

# 2. Ziel des MVP

Das MVP soll beweisen, dass der grundlegende Ansatz von NC-PoRe funktioniert:

> Lokale Aufnahme ermöglicht professionelle Produktion,
> ohne die Kontrolle über eigene Daten abzugeben.

Das MVP konzentriert sich deshalb auf den Kernprozess:

```text
Aufnahme
   |
   v
Lokale Speicherung
   |
   v
Session-Verwaltung
   |
   v
Synchronisation
   |
   v
Zentrale Verwaltung
```

---

# 3. MVP-Grundprinzipien

Das MVP folgt den bestehenden Architekturprinzipien:

* Local Recording First
* Production Session als zentrale fachliche Einheit
* getrennte Audiospuren
* offene Datenformate
* selbsthostbare Infrastruktur
* klare Trennung von Client und Core
* nachvollziehbare Datenflüsse

---

# 4. MVP Umfang

## 4.1 Client

Der erste Client muss ermöglichen:

* Erstellung oder Auswahl einer Production Session
* lokale Audioaufnahme
* Speicherung lokaler Aufnahmedaten
* Anzeige grundlegender Sessioninformationen
* Übertragung von Daten an die zentrale Umgebung

Nicht erforderlich:

* vollständige professionelle DAW-Funktionen
* komplexe Audioeffekte
* umfangreiche Bearbeitungsfunktionen

---

## 4.2 Production Session

Das MVP benötigt eine grundlegende Verwaltung von:

* Session
* Teilnehmern
* Aufnahmen
* Metadaten

Eine Session muss eindeutig identifizierbar sein.

---

## 4.3 Core

Der Core muss ermöglichen:

* Verwaltung von Sessions
* Verwaltung grundlegender Zustände
* Prüfung einfacher Berechtigungen
* Bereitstellung von Schnittstellen für Clients

Der Core ist die fachliche Autorität.

---

## 4.4 Speicherung

Das MVP muss ermöglichen:

* Speicherung von Audiodaten
* Speicherung von Metadaten
* Wiederauffindbarkeit von Sessions und Assets

Die Speicherung muss die Grundidee unterstützen:

> Daten bleiben unter Kontrolle der Nutzer.

---

## 4.5 Synchronisation

Das MVP muss einen grundlegenden Synchronisationsablauf zeigen:

* lokale Daten entstehen unabhängig vom Netzwerk
* Daten werden kontrolliert übertragen
* zentrale Umgebung kennt den aktuellen Zustand

Die vollständige verteilte Synchronisationsarchitektur bleibt zukünftigen Ausbaustufen vorbehalten.

---

# 5. Bewusste Nicht-Ziele

Folgende Bereiche gehören nicht zum ersten MVP:

## Vollständige professionelle Produktionsumgebung

Nicht enthalten:

* Mehrspur-Mixing
* komplexe Nachbearbeitung
* integrierte Effekte
* vollständiger Studio-Workflow

---

## Maximale Plattformunterstützung

Nicht enthalten:

* sofortige Unterstützung aller Plattformen
* vollständige mobile Clients
* perfekte Integration jeder Hardware

---

## Vollständige Skalierung

Nicht enthalten:

* große Enterprise-Installationen
* globale Infrastruktur
* maximale Performanceoptimierung

---

# 6. Erfolgskriterien

Das MVP ist erfolgreich, wenn:

* eine Session erstellt werden kann
* lokale Aufnahme funktioniert
* Audiodaten erhalten bleiben
* Daten kontrolliert übertragen werden können
* eine zentrale Verwaltung möglich ist
* Architekturprinzipien sichtbar umgesetzt sind

Der Erfolg wird nicht an der Anzahl von Funktionen gemessen.

---

# 7. Technische Risiken, die das MVP prüfen soll

Das MVP soll insbesondere folgende Risiken früh sichtbar machen:

* Audioaufnahme über verschiedene Plattformen
* lokale Speicherung
* Synchronisation
* Datenmodell der Production Session
* Zusammenspiel zwischen Client und Core

---

# 8. Grundsatz

Das MVP ist der erste funktionierende Ausschnitt von NC-PoRe.

Es soll zeigen:

Die Architektur funktioniert.

Nicht:

Die gesamte Vision ist bereits umgesetzt.

NC-PoRe wird schrittweise erweitert.

Jeder Schritt soll eine stabile Grundlage für den nächsten schaffen.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# 1. Purpose of this Document

This document describes the first technical implementation stage
of NC-PoRe.

The MVP (Minimum Viable Product) defines the smallest complete
production workflow that technically proves the central architectural
principles of NC-PoRe.

The MVP is not a complete product.

However, it is also not throw-away prototype code.

The MVP implementation should be based on the long-term architectural
principles of NC-PoRe.

---

# 2. Goal of the MVP

The MVP should prove that the fundamental approach of NC-PoRe works:

> Local recording enables professional production
> without giving up control over personal data.

Therefore, the MVP focuses on the core workflow:

```text
Recording
   |
   v
Local Storage
   |
   v
Session Management
   |
   v
Synchronization
   |
   v
Central Management
```

---

# 3. MVP Principles

The MVP follows the existing architecture principles:

* Local Recording First
* Production Session as central domain entity
* separate audio tracks
* open data formats
* self-hostable infrastructure
* clear separation of Client and Core
* traceable data flows

---

# 4. MVP Scope

## 4.1 Client

The first client must enable:

* creation or selection of a Production Session
* local audio recording
* local storage of recording data
* display of basic session information
* transfer of data to the central environment

Not required:

* complete professional DAW functionality
* complex audio effects
* extensive editing features

---

## 4.2 Production Session

The MVP requires basic management of:

* session
* participants
* recordings
* metadata

A session must have a unique identity.

---

## 4.3 Core

The Core must enable:

* session management
* basic state management
* simple permission checks
* interfaces for clients

The Core is the domain authority.

---

## 4.4 Storage

The MVP must enable:

* storage of audio data
* storage of metadata
* retrieval of sessions and assets

Storage must support the core idea:

> Data remains under user control.

---

## 4.5 Synchronization

The MVP must demonstrate a basic synchronization workflow:

* local data is created independently from the network
* data is transferred in a controlled manner
* the central environment knows the current state

The complete distributed synchronization architecture remains for future stages.

---

# 5. Explicit Non-Goals

The following areas are not part of the first MVP:

## Complete Professional Production Environment

Not included:

* multi-track mixing
* complex post-production
* integrated effects
* complete studio workflow

---

## Maximum Platform Support

Not included:

* immediate support for all platforms
* complete mobile clients
* perfect integration with every hardware device

---

## Complete Scaling

Not included:

* large enterprise installations
* global infrastructure
* maximum performance optimization

---

# 6. Success Criteria

The MVP is successful when:

* a session can be created
* local recording works
* audio data is preserved
* data can be transferred in a controlled way
* central management is possible
* architecture principles are visibly implemented

Success is not measured by the number of features.

---

# 7. Technical Risks Addressed by the MVP

The MVP should reveal the following risks early:

* audio recording across platforms
* local storage
* synchronization
* Production Session data model
* interaction between Client and Core

---

# 8. Principle

The MVP is the first working part of NC-PoRe.

It should demonstrate:

The architecture works.

Not:

The complete vision has already been implemented.

NC-PoRe will be expanded step by step.

Each step should create a stable foundation for the next one.
