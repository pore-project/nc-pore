# NC-PoRe Implementation Plan

## Deutsche Version ([English version below](#english-version))

---

# Zweck

Dieses Dokument beschreibt den Weg von der abgeschlossenen Architekturgrundlage zur ersten nutzbaren Version von NC-PoRe.

Der Implementation Plan beschreibt:

- in welcher Reihenfolge technische Komponenten entstehen
- welche Ergebnisse einzelne Phasen liefern sollen
- welche Entscheidungen vor der Umsetzung getroffen werden müssen
- welche Abhängigkeiten zwischen Komponenten bestehen

Dieses Dokument ersetzt keine Architecture Decision Records (ADRs).

ADRs beantworten:

> Warum wurde eine Entscheidung getroffen?

Der Implementation Plan beantwortet:

> In welcher Reihenfolge werden diese Entscheidungen umgesetzt?

---

# Grundsätze der Umsetzung

Die Umsetzung von NC-PoRe folgt den bestehenden Projektprinzipien.

## Präzision vor Marketing

Technische Dokumentation beschreibt die Realität des Systems.

Sie soll:

- präzise Begriffe verwenden
- Entscheidungen nachvollziehbar machen
- Auswirkungen von Entscheidungen beschreiben
- zwischen vorhandenen Fähigkeiten und zukünftigen Zielen unterscheiden

Technische Dokumentation ist kein Marketingmaterial.

---

## Architektur zuerst, Code danach

Die Architekturgrundlagen wurden bewusst vor Beginn der Implementierung definiert.

Die Umsetzung folgt dieser Struktur:

```text
Architektur

↓

Implementation Plan

↓

Technische Entscheidungen

↓

Code
```

---

## Kleine vollständige Schritte

NC-PoRe wird bevorzugt über vollständige vertikale Schritte entwickelt.

Ein kleiner vollständiger Ablauf ist wertvoller als viele isolierte Komponenten.

Beispiel:

```text
Production Session erzeugen

↓

Session speichern

↓

Session über API verfügbar machen

↓

Session im Client anzeigen
```

---

## Verantwortlichkeiten bleiben getrennt

Die Architekturprinzipien bleiben während der Umsetzung erhalten:

- Der Core enthält die fachliche Logik.
- Clients kümmern sich um Benutzerinteraktion.
- APIs definieren Kommunikationsgrenzen.
- Storage übernimmt persistente Datenhaltung.

---

## Keine vorzeitige Komplexität

Komplexität wird erst eingeführt, wenn ein konkreter Bedarf besteht.

NC-PoRe vermeidet:

- Optimierung ohne Messung
- Abstraktionen ohne Nutzen
- Skalierungsmechanismen ohne Anforderung
- technische Lösungen ohne konkreten Anwendungsfall

---

# Phase 1: Technische Projektgrundlage

## Ziel

Eine reproduzierbare Entwicklungsumgebung schaffen.

## Ergebnis

- Repository-Struktur definiert
- Entwicklungsumgebung dokumentiert
- Build-Prozess eingerichtet
- automatische Prüfungen möglich
- Entwicklungsworkflow beschrieben

## Zu klärende Entscheidungen

- Programmiersprachen
- Buildsystem
- Workspace-Struktur
- Entwicklungswerkzeuge

---

# Phase 2: Core-Implementierung

## Ziel

Die fachliche Grundlage von NC-PoRe implementieren.

Der Core ist die erste ausführbare Umsetzung der Architektur.

## Ergebnis

- Production Session kann erzeugt werden
- zentrale Domänenobjekte existieren
- Geschäftsregeln sind testbar
- Core funktioniert unabhängig von Clients

## Schwerpunkt

- Production Session
- Participants
- Roles
- Recordings
- Assets
- Activity History

---

# Phase 3: API- und Kommunikationsgrundlage

## Ziel

Die fachlichen Fähigkeiten des bestehenden Core als stabile Kommunikationsgrenze beschreiben.

Die Phase definiert zunächst die API als architektonischen Vertrag.

Ein konkretes Kommunikationsprotokoll wird in dieser Phase noch nicht festgelegt.

## Ergebnis

- fachliche Fähigkeiten des Core sind als API-Fähigkeiten beschrieben
- API-Grenzen orientieren sich an den Domänenobjekten und Produktionsabläufen
- Eingaben und Ergebnisse der wesentlichen Operationen sind definiert
- API und interne Implementierung bleiben getrennt
- Anforderungen an Versionierung und Erweiterbarkeit sind dokumentiert
- spätere Kommunikationsprotokolle können auf diesem Vertrag aufbauen

## Schwerpunkt

- Domain-orientierte API
- Session-bezogene Operationen
- Teilnehmerverwaltung
- Recording-bezogene Operationen
- Asset-bezogene Operationen
- Zustandsänderungen
- Fehler- und Ergebnissemantik
- API-Versionierung
- Dokumentation

## Bewusste Abgrenzung

Diese Phase implementiert noch kein konkretes externes Kommunikationsprotokoll.

Insbesondere werden noch nicht ohne konkreten Bedarf festgelegt:

- REST
- WebSocket
- gRPC
- konkrete Netzwerkarchitektur
- vollständige Authentifizierungsinfrastruktur
- externe Entwickler-API

Die technische Umsetzung der Kommunikationsgrenze wird erst festgelegt, wenn ein konkreter Client oder ein anderes externes System diese Grenze benötigt.

## Bezug

- ADR-028 API Design Principles
- ADR-034 Implementation Architecture

Die bereits implementierten internen technischen Schnittstellen bleiben davon unberührt.

Sie bilden die technischen Grenzen innerhalb des Systems und sind nicht mit der späteren externen API gleichzusetzen.

---

# Phase 4: Erster Client

## Ziel

Die API- und Kommunikationsgrenze durch einen realen Client validieren.

Der erste Client muss nicht vollständig sein.

Er soll zeigen, dass:

- ein Client definierte Core-Fähigkeiten nutzen kann
- Sessions über die vorgesehene Kommunikationsgrenze verwaltet werden können
- die Trennung zwischen Client und Core erhalten bleibt
- interne Implementierungsdetails des Core nicht Teil des Clients werden

Die konkrete technische Form der Kommunikation wird dabei nur soweit festgelegt, wie sie für den ersten Client erforderlich ist.

---

# Phase 5: Lokale Aufnahme

## Ziel

Das zentrale NC-PoRe-Prinzip technisch umsetzen:

> Lokal aufnehmen. Danach synchronisieren.

## Ergebnis

- lokale Audioaufnahme
- lokale Speicherung
- Aufnahmemetadaten
- Vorbereitung für Synchronisation

## Grundsatz

Die Aufnahme darf nicht von einer aktiven Netzwerkverbindung abhängig sein.

---

# Phase 6: Synchronisation

## Ziel

Lokale Produktionsdaten mit der zentralen Umgebung verbinden.

## Ergebnis

- Assets können übertragen werden
- Synchronisationszustände sind nachvollziehbar
- Konflikte können behandelt werden

## Bezug

- ADR-029 Distributed Recording Architecture
- ADR-030 Synchronization Strategy

---

# Phase 7: Produktionsworkflow

## Ziel

Die einzelnen Komponenten zu einem vollständigen Arbeitsablauf verbinden.

## Ergebnis

Ein vollständiger Produktionsablauf:

```text
Production Session erstellen

↓

Teilnehmer verwalten

↓

Lokal aufnehmen

↓

Assets synchronisieren

↓

Produktionsstatus prüfen

↓

Ergebnis exportieren
```

---

# Phase 8: Erste nutzbare Version

## Ziel

Eine Version erstellen, die für reale Podcast-Produktion eingesetzt werden kann.

## Ergebnis

- stabiler Aufnahmeprozess
- zuverlässige Synchronisation
- nutzbare Clients
- dokumentierte Installation

---

# Entscheidungspunkte

Während der Umsetzung werden neue Entscheidungen als ADR dokumentiert, wenn sie:

- mehrere Komponenten betreffen
- langfristige Auswirkungen haben
- bestehende Architekturprinzipien verändern

---

# Was wir bewusst noch nicht tun

NC-PoRe vermeidet bewusst:

- vollständige Benutzeroberflächen vor stabiler Kernlogik
- Optimierung vor realer Nutzung
- technische Komplexität ohne Bedarf
- Ersatz bestehender Werkzeuge ohne Grund
- Lösungen für hypothetische Probleme
- Festlegung eines Kommunikationsprotokolls ohne konkreten Bedarf

---

# Aktueller Umsetzungsstatus

Status:

## Technische Umsetzung läuft

Die Architekturgrundlage ist abgeschlossen.

Die ersten fachlichen Core-Modelle sowie zentrale Recorder- und Persistenzkomponenten wurden implementiert und durch automatisierte Tests validiert.

Der Recorder unterstützt inzwischen:

- Recording-Lifecycle
- Capture Boundary
- Workflow Coordination
- Recording Artifacts
- Artifact Registry
- Artifact Processing
- Persistence Provider
- Filesystem Persistence
- Artifact Recovery

Die nächsten Schritte konzentrieren sich auf die Definition der API- und Kommunikationsgrundlage sowie auf weitere fachliche und technische Workflows.

---

# Beziehung zu anderen Dokumenten

```text
README.md

↓

project-status.md

↓

implementation-plan.md

↓

ADR-Dokumente

↓

Source Code
```

Die Dokumente haben unterschiedliche Aufgaben:

- README beschreibt das Projekt.
- project-status beschreibt den aktuellen Zustand.
- implementation-plan beschreibt den Weg zur Umsetzung.
- ADRs erklären Entscheidungen.
- Source Code implementiert die Ergebnisse.

---

# Leitgedanke

NC-PoRe wird nicht durch möglichst schnelles Schreiben von Code entwickelt.

NC-PoRe wird durch nachvollziehbare Entscheidungen entwickelt.

Diese Entscheidungen werden anschließend in zuverlässige Software umgesetzt.

---

# English Version ([Deutsche Version oben](#deutsche-version))

---

# Purpose

This document describes the path from the completed architecture foundation to the first usable version of NC-PoRe.

The Implementation Plan describes:

- the order in which technical components are created
- expected results of individual phases
- decisions required before implementation
- dependencies between components

This document does not replace Architecture Decision Records (ADRs).

ADRs answer:

> Why was a decision made?

The Implementation Plan answers:

> In which order are these decisions implemented?

---

# Implementation Principles

NC-PoRe implementation follows the established project principles.

## Precision over Marketing

Technical documentation describes the actual system.

It should:

- use precise terminology
- make decisions understandable
- describe consequences of decisions
- distinguish implemented capabilities from future goals

Technical documentation is not marketing material.

---

## Architecture First, Code Second

The architectural foundation was deliberately created before implementation started.

Implementation follows this structure:

```text
Architecture

↓

Implementation Plan

↓

Technical Decisions

↓

Code
```

---

## Small Complete Steps

NC-PoRe prefers complete vertical steps over isolated components.

A small complete workflow is more valuable than many isolated components.

Example:

```text
Create Production Session

↓

Store Session

↓

Expose Session through API

↓

Display Session in Client
```

---

## Keep Responsibilities Separate

The architectural principles remain valid during implementation:

- The Core contains domain logic.
- Clients handle user interaction.
- APIs define communication boundaries.
- Storage handles persistent data.

---

## Avoid Premature Complexity

Complexity is introduced only when there is a concrete need.

NC-PoRe avoids:

- optimization without measurement
- abstractions without purpose
- scalability mechanisms without requirements
- technical solutions without a concrete use case

---

# Phase 1: Technical Project Foundation

## Goal

Create a reproducible development environment.

## Result

- repository structure defined
- development environment documented
- build process established
- automated checks available
- development workflow documented

## Decisions to Clarify

- programming languages
- build system
- workspace structure
- development tools

---

# Phase 2: Core Implementation

## Goal

Implement the domain foundation of NC-PoRe.

The Core is the first executable implementation of the architecture.

## Result

- Production Sessions can be created
- central domain objects exist
- business rules can be tested
- the Core operates independently of clients

## Focus

- Production Session
- Participants
- Roles
- Recordings
- Assets
- Activity History

---

# Phase 3: API and Communication Foundation

## Goal

Describe the capabilities of the existing Core as a stable communication boundary.

This phase initially defines the API as an architectural contract.

A concrete communication protocol is not selected during this phase.

## Result

- Core capabilities are described as API capabilities
- API boundaries follow domain objects and production workflows
- inputs and results of essential operations are defined
- API and internal implementation remain separated
- versioning and extensibility requirements are documented
- future communication protocols can build on this contract

## Focus

- domain-oriented API
- session-related operations
- participant management
- recording-related operations
- asset-related operations
- state changes
- error and result semantics
- API versioning
- documentation

## Explicit Scope Boundary

This phase does not implement a concrete external communication protocol.

In particular, the following are not selected without a concrete requirement:

- REST
- WebSocket
- gRPC
- concrete network architecture
- complete authentication infrastructure
- external developer API

The technical implementation of the communication boundary is selected only when a concrete client or another external system requires it.

## References

- ADR-028 API Design Principles
- ADR-034 Implementation Architecture

Existing internal technical interfaces remain unaffected.

They define technical boundaries within the system and must not be confused with the later external API.

---

# Phase 4: First Client

## Goal

Validate the API and communication boundary through a real client.

The first client does not need to be complete.

It should demonstrate that:

- a client can use defined Core capabilities
- sessions can be managed through the intended communication boundary
- the separation between client and Core remains intact
- internal Core implementation details do not become part of the client

The concrete technical form of communication is selected only to the extent required by the first client.

---

# Phase 5: Local Recording

## Goal

Implement the central NC-PoRe principle technically:

> Record locally. Synchronize afterwards.

## Result

- local audio recording
- local storage
- recording metadata
- preparation for synchronization

## Principle

Recording must not depend on an active network connection.

---

# Phase 6: Synchronization

## Goal

Connect local production data with the central environment.

## Result

- assets can be transferred
- synchronization states are traceable
- conflicts can be handled

## References

- ADR-029 Distributed Recording Architecture
- ADR-030 Synchronization Strategy

---

# Phase 7: Production Workflow

## Goal

Connect the individual components into a complete workflow.

## Result

A complete production workflow:

```text
Create Production Session

↓

Manage participants

↓

Record locally

↓

Synchronize assets

↓

Check production status

↓

Export result
```

---

# Phase 8: First Usable Version

## Goal

Create a version that can be used for real podcast production.

## Result

- stable recording process
- reliable synchronization
- usable clients
- documented installation

---

# Decision Points

During implementation, new decisions are documented as ADRs when they:

- affect multiple components
- have long-term consequences
- change established architectural principles

---

# What We Deliberately Do Not Do Yet

NC-PoRe deliberately avoids:

- complete user interfaces before stable core logic
- optimization before real-world use
- technical complexity without a concrete need
- replacing existing tools without a reason
- solutions for hypothetical problems
- selecting a communication protocol without a concrete requirement

---

# Current Implementation Status

Status:

## Technical Implementation In Progress

The architecture foundation is complete.

The first Core domain models as well as central Recorder and persistence components have been implemented and validated through automated tests.

The Recorder now supports:

- recording lifecycle
- capture boundary
- workflow coordination
- Recording Artifacts
- Artifact Registry
- artifact processing
- Persistence Provider
- filesystem persistence
- artifact recovery

The next steps focus on defining the API and communication foundation and on implementing further domain and technical workflows.

---

# Relationship to Other Documents

```text
README.md

↓

project-status.md

↓

implementation-plan.md

↓

ADR documents

↓

Source Code
```

The documents have different purposes:

- README describes the project.
- project-status describes the current state.
- implementation-plan describes the path to implementation.
- ADRs explain decisions.
- Source Code implements the results.

---

# Guiding Principle

NC-PoRe is not developed by writing code as quickly as possible.

NC-PoRe is developed through traceable decisions.

These decisions are then implemented as reliable software.
