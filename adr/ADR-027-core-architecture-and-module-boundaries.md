# ADR-027: Core Architecture and Module Boundaries

* Status: Accepted
* Date: 2026-07-23
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

## Kontext

NC-PoRe ist eine Umgebung zur kollaborativen Podcast-Produktion.

Die bisherigen Architekturentscheidungen haben folgende Grundlagen geschaffen:

* Die zentrale fachliche Einheit ist die **Production Session**.
* Medien werden als Assets innerhalb einer Session betrachtet.
* Speicher und externe Systeme sollen austauschbar bleiben.
* Clients und Integrationen sollen unabhängig vom Core entwickelt werden können.

Damit NC-PoRe erweiterbar und wartbar bleibt, benötigt das Projekt klare Modulgrenzen und eine saubere Trennung von Verantwortlichkeiten.

---

## Entscheidung

NC-PoRe wird modular aufgebaut.

Die Architektur trennt zwischen:

1. **Core**
2. **Interfaces / APIs**
3. **Clients**
4. **Infrastructure und Provider**

Der Core enthält die fachliche Logik von NC-PoRe und bleibt unabhängig von konkreten technischen Implementierungen.

---

# Architekturprinzip

Die zentrale Regel lautet:

> Der Core kennt die Welt nicht. Die Welt kennt den Core.

Der Core soll keine Abhängigkeiten besitzen zu:

* konkreten Host-Anwendungen
* Betriebssystemen oder Plattformimplementierungen
* Benutzeroberflächen
* konkreten Speichertechnologien
* externen Kommunikationssystemen

Stattdessen werden diese über definierte Schnittstellen angebunden.

---

## Architekturübersicht

```text
                 Clients
                    |
                 API Layer
                    |
               NC-PoRe Core
                    |
          Interfaces / Adapters
                    |
          +---------+---------+
          |                   |
     Storage Provider    Host/Integration
          |                   |
       Storage            Host System
```

---

# Core-Verantwortung

Der Core ist die fachliche Wahrheit von NC-PoRe.

Er verwaltet unter anderem:

* Production Sessions
* Teilnehmer
* Assets
* Session-Zustände
* Produktionsregeln
* Events
* Metadaten

Der Core entscheidet:

> Was ist eine gültige Produktion?

Der Core entscheidet nicht:

> Wo wird eine Datei gespeichert?

oder:

> Welche Oberfläche oder technische Integration benutzt der Benutzer?

---

# Module

## Core Module

Verantwortlich für:

* Domain Model
* Business Logic
* Session Management
* Events
* Interfaces

Der Core soll möglichst stabil bleiben.

---

## Client Module

Clients stellen die Benutzerinteraktion bereit.

Clients enthalten keine zentrale Geschäftslogik.

---

## Storage Module

Storage Provider kümmern sich um die technische Speicherung.

Der Core kennt nur das Storage Interface.

---

## Integration Module

Externe Systeme werden über Integrationen bzw. Adapter angebunden.

Die Produktionslogik bleibt unabhängig vom konkreten Integrationsweg.

---

# API First Prinzip

NC-PoRe wird API-orientiert entwickelt.

Dies bedeutet nicht zwingend, dass jede API öffentlich angeboten werden muss.

Die API dient als klare Grenze zwischen:

* Benutzeroberflächen
* externen Systemen
* Core-Logik

Dadurch können unterschiedliche Clients und Integrationen dieselbe fachliche Basis verwenden.

---

# Dependency Rule

Abhängigkeiten fließen nur nach innen:

```text
Client / Integration
        ↓
       API
        ↓
      Core
        ↓
Interfaces / Provider
```

Der Core darf keine Infrastrukturdetails kennen.

---

# Grundprinzipien

## 1. Modularität statt Monolith

Funktionalität wird über klar abgegrenzte Module organisiert.

Bestehende Kernfunktionen sollen stabil bleiben.

---

## 2. Boring Core

Der Core soll bewusst einfach und langlebig bleiben.

Er enthält:

* Modelle
* Regeln
* Zustände

Er enthält nicht:

* UI
* Netzwerkdetails
* Dateisystemzugriffe
* Provider-spezifische Logik

---

## 3. Erweiterung durch Adapter

Externe Systeme werden über Adapter integriert.

Die Integrationslogik bleibt dabei außerhalb des Core.

---

## 4. Nutzerorientierung

Technische Komplexität bleibt hinter klaren Schnittstellen verborgen.

Der Benutzer soll nicht wissen müssen:

* welcher Speicher verwendet wird
* welche Kommunikationstechnologie aktiv ist
* welche technische Integration die Session bereitstellt

Die fachliche Erfahrung bleibt konsistent.

---

# Konsequenzen

## Vorteile

* klare Modulgrenzen
* geringere Abhängigkeiten
* austauschbare Provider und Integrationen
* bessere Testbarkeit
* einfachere Wartung

## Nachteile

* höherer initialer Entwicklungsaufwand
* zusätzliche Schnittstellen müssen gepflegt werden
* Architektur benötigt Disziplin

Diese Nachteile werden bewusst akzeptiert.

---

# Nicht-Ziele

Diese Entscheidung bedeutet nicht:

* dass jede mögliche Client- oder Integrationvariante umgesetzt werden muss
* dass jede Schnittstelle öffentlich dokumentiert werden muss
* dass die konkrete Programmiersprache festgelegt wird
* dass technische Implementierungen des Core ausgeschlossen werden

Konkrete technische Entscheidungen werden separat getroffen, sobald sie erforderlich sind.

---

# Leitgedanke

NC-PoRe soll nicht nur funktionieren.

Ein stabiler Kern, klare Grenzen und austauschbare Module bilden die Grundlage für eine wartbare Softwarearchitektur.

---

# English Version ([Deutsche Version oben](#deutsch))

## Context

NC-PoRe is an environment for collaborative podcast production.

Previous architecture decisions established:

* The central domain entity is the **Production Session**.
* Media is managed as assets within sessions.
* Storage and external systems should remain replaceable.
* Clients and integrations should be able to evolve independently from the Core.

To keep NC-PoRe extensible and maintainable, the project requires clear module boundaries and separation of responsibilities.

---

## Decision

NC-PoRe is built as a modular architecture.

The architecture separates:

1. **Core**
2. **Interfaces / APIs**
3. **Clients**
4. **Infrastructure and Providers**

The Core contains NC-PoRe's domain logic and remains independent from concrete technical implementations.

---

# Architecture Principle

The central rule is:

> The Core does not know the world. The world knows the Core.

The Core must not depend on:

* concrete host applications
* operating systems or platform implementations
* user interfaces
* concrete storage technologies
* external communication systems

These are connected through defined interfaces.

---

## Architecture Overview

```text
                 Clients
                    |
                 API Layer
                    |
               NC-PoRe Core
                    |
          Interfaces / Adapters
                    |
          +---------+---------+
          |                   |
     Storage Provider    Host/Integration
          |                   |
       Storage            Host System
```

---

# Core Responsibility

The Core represents the domain truth of NC-PoRe.

It manages, among other things:

* Production Sessions
* Participants
* Assets
* Session states
* Production rules
* Events
* Metadata

The Core decides:

> What is a valid production?

The Core does not decide:

> Where is a file stored?

or:

> Which interface or technical integration is used by the user?

---

# Modules

## Core Module

Responsible for:

* Domain Model
* Business Logic
* Session Management
* Events
* Interfaces

The Core should remain as stable as possible.

---

## Client Module

Clients provide user interaction.

Clients contain no central business logic.

---

## Storage Module

Storage providers handle technical persistence.

The Core knows only the storage interface.

---

## Integration Module

External systems are connected through integrations or adapters.

Production logic remains independent of the concrete integration path.

---

# API First Principle

NC-PoRe is developed API-oriented.

This does not necessarily mean that every API must be publicly exposed.

The API provides a clear boundary between:

* user interfaces
* external systems
* Core logic

Different clients and integrations can therefore share the same domain foundation.

---

# Dependency Rule

Dependencies point inward only:

```text
Client / Integration
        ↓
       API
        ↓
      Core
        ↓
Interfaces / Provider
```

The Core must not know infrastructure details.

---

# Principles

## 1. Modularity instead of Monolith

Functionality is organized through clearly separated modules.

Existing core functionality should remain stable.

---

## 2. Boring Core

The Core should remain deliberately simple and long-lived.

It contains:

* models
* rules
* states

It does not contain:

* UI
* networking details
* filesystem access
* provider-specific logic

---

## 3. Extension through Adapters

External systems are integrated through adapters.

Integration logic remains outside the Core.

---

## 4. User Orientation

Technical complexity remains behind clear interfaces.

Users should not need to know:

* which storage is used
* which communication technology is active
* which technical integration provides the session

The domain-level experience remains consistent.

---

# Consequences

## Benefits

* clear module boundaries
* fewer dependencies
* replaceable providers and integrations
* better testability
* easier maintenance

## Costs

* higher initial development effort
* additional interfaces require maintenance
* architecture requires discipline

These costs are consciously accepted.

---

# Non-Goals

This decision does not mean:

* that every possible client or integration variant must be implemented
* that every interface must be publicly documented
* that a specific programming language is mandated
* that technical Core implementations are excluded

Concrete technical decisions are made separately when required.

---

# Guiding Principle

NC-PoRe should not merely work.

A stable Core, clear boundaries, and replaceable modules form the foundation of a maintainable software architecture.
