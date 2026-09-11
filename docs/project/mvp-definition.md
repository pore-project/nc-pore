# NC-PoRe MVP Definition

* Version: 1.1
* Date: 2026-09-11

---

# Deutsch ([English version below](#english-version))

---

# 1. Zweck dieses Dokuments

Dieses Dokument beschreibt die erste technische Ausbaustufe von NC-PoRe.

Das MVP (Minimum Viable Product) definiert einen kleinen, vollständigen Produktionsablauf, der die zentralen Architekturprinzipien von NC-PoRe technisch nachweist.

Das MVP ist kein vollständiges Produkt und kein Wegwerf-Prototyp. Die Umsetzung baut auf den verbindlichen Architekturentscheidungen des Projekts auf.

---

# 2. Ziel des MVP

Das MVP soll zeigen, dass der grundlegende Ansatz von NC-PoRe funktioniert:

> Lokale Aufnahme ermöglicht professionelle Produktion, ohne die Kontrolle über eigene Daten abzugeben.

Der Kernprozess ist:

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

# 4. MVP-Umfang

## 4.1 Client

Der Client muss ermöglichen:

* Erstellung oder Auswahl einer Production Session
* lokale Audioaufnahme
* lokale Speicherung von Aufnahmedaten
* Anzeige grundlegender Sessioninformationen
* kontrollierte Übergabe fertiger Aufnahmen an die zentrale Umgebung

Nicht erforderlich sind insbesondere:

* vollständige professionelle DAW-Funktionen
* komplexe Audioeffekte
* umfangreiche Bearbeitungsfunktionen

---

## 4.2 Production Session

Das MVP benötigt eine grundlegende Verwaltung von:

* Session
* Recording-Teilnehmern
* Aufnahmen
* Metadaten

Eine Session muss eindeutig identifizierbar sein.

---

## 4.3 Core

Der Core muss ermöglichen:

* Verwaltung von Sessions
* Verwaltung grundlegender Zustände
* Prüfung von Berechtigungen
* Bereitstellung klarer Schnittstellen für Clients

Der Core ist die fachliche Autorität.

---

## 4.4 Speicherung

Das MVP muss ermöglichen:

* lokale und kontrollierte Speicherung von Audiodaten
* Speicherung von Metadaten
* Wiederauffindbarkeit von Sessions und Assets
* nachvollziehbare Integritätsprüfung bei der Übergabe

Die Speicherung unterstützt das Grundprinzip:

> Daten bleiben unter Kontrolle der Nutzer.

---

## 4.5 Synchronisation

Das MVP muss einen kontrollierten Synchronisationsablauf ermöglichen:

* lokale Daten entstehen unabhängig von der Netzwerkqualität
* fertige Daten werden kontrolliert übertragen
* der zentrale Zustand kann nachvollzogen werden

Die konkrete technische Ausgestaltung der Synchronisation richtet sich nach den dafür getroffenen Architekturentscheidungen.

---

# 5. Bewusste Nicht-Ziele

Folgende Bereiche sind nicht Bestandteil dieses MVP:

## Vollständige professionelle Produktionsumgebung

Nicht enthalten:

* Mehrspur-Mixing
* komplexe Nachbearbeitung
* integrierte Effekte
* vollständiger Studio-Workflow

---

## Weitere Medien- und Kommunikationsfunktionen

Nicht enthalten:

* Videoaufnahme
* Live-Streaming
* automatische Veröffentlichung

Diese Punkte sind hier keine Zieldefinition des MVP.

---

## Skalierung

Nicht Bestandteil der MVP-Definition sind insbesondere:

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
* die Architekturprinzipien sichtbar umgesetzt sind

Der Erfolg wird nicht an der Anzahl von Funktionen gemessen.

---

# 7. Technische Risiken

Das MVP soll insbesondere folgende Risiken früh sichtbar machen:

* zuverlässige lokale Audioaufnahme
* lokale Speicherung und Wiederherstellung
* Synchronisation und kontrollierte Übergabe
* Datenmodell der Production Session
* Zusammenspiel zwischen Client und Core

---

# 8. Grundsatz

Das MVP ist ein klar abgegrenzter funktionierender Ausschnitt von NC-PoRe.

Es soll zeigen:

> Die Architektur funktioniert.

Es definiert nicht die vollständige Produktentwicklung und enthält keine vollständige Liste möglicher späterer Funktionen.

NC-PoRe wird anhand realer Anforderungen und technischer Erkenntnisse weiterentwickelt.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# 1. Purpose of this Document

This document describes the first technical implementation stage of NC-PoRE.

The MVP (Minimum Viable Product) defines a small, complete production workflow that technically demonstrates the central architectural principles of NC-PoRE.

The MVP is not a complete product and not throw-away prototype code. Its implementation is based on the project's binding architectural decisions.

---

# 2. Goal of the MVP

The MVP should demonstrate that the fundamental approach of NC-PoRe works:

> Local recording enables professional production without giving up control over personal data.

The core workflow is:

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

The MVP follows the existing architectural principles:

* Local Recording First
* Production Session as the central domain entity
* separate audio tracks
* open data formats
* self-hostable infrastructure
* clear separation of Client and Core
* traceable data flows

---

# 4. MVP Scope

## 4.1 Client

The client must enable:

* creation or selection of a Production Session
* local audio recording
* local storage of recording data
* display of basic session information
* controlled handoff of finalized recordings to the central environment

The MVP does not require in particular:

* complete professional DAW functionality
* complex audio effects
* extensive editing features

---

## 4.2 Production Session

The MVP requires basic management of:

* session
* recording participants
* recordings
* metadata

A session must have a unique identity.

---

## 4.3 Core

The Core must enable:

* session management
* basic state management
* permission checks
* clear interfaces for clients

The Core is the domain authority.

---

## 4.4 Storage

The MVP must enable:

* local and controlled storage of audio data
* storage of metadata
* retrieval of sessions and assets
* traceable integrity verification during handoff

Storage supports the core idea:

> Data remains under user control.

---

## 4.5 Synchronization

The MVP must provide a controlled synchronization workflow:

* local data is created independently of network quality
* finalized data is transferred in a controlled manner
* central state can be traced

The concrete technical implementation of synchronization follows the corresponding architectural decisions.

---

# 5. Explicit Non-Goals

The following areas are not part of this MVP:

## Complete Professional Production Environment

Not included:

* multi-track mixing
* complex post-production
* integrated effects
* complete studio workflow

---

## Additional Media and Communication Functions

Not included:

* video recording
* live streaming
* automatic publishing

These items are not part of the MVP definition.

---

## Scaling

The MVP definition does not cover in particular:

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
* architectural principles are visibly implemented

Success is not measured by the number of features.

---

# 7. Technical Risks

The MVP should make the following risks visible early:

* reliable local audio recording
* local storage and recovery
* synchronization and controlled handoff
* Production Session data model
* interaction between Client and Core

---

# 8. Principle

The MVP is a clearly bounded working part of NC-PoRe.

It should demonstrate:

> The architecture works.

It does not define the complete product development and does not provide a complete list of possible later features.

NC-PoRe evolves according to real requirements and technical findings.
