# ADR-027: Core Architecture and Module Boundaries

* Status: Accepted
* Date: 2026-07-23
* Decision Type: Architecture

---

# Deutsch (English version below)

## Kontext

NC-PoRe wurde als **Nextcloud Podcast Production Environment** definiert.

Die bisherigen Architekturentscheidungen haben folgende Grundlagen geschaffen:

* NC-PoRe ist eine Umgebung zur kollaborativen Podcast-Produktion.
* Die zentrale fachliche Einheit ist die **Production Session**.
* Medien werden als Assets innerhalb einer Session betrachtet.
* Speicher und externe Systeme sollen austauschbar bleiben.
* Mehrere Clients und Plattformen sollen langfristig unterstützt werden.

Damit NC-PoRe über viele Jahre erweiterbar bleibt, benötigt das Projekt klare Modulgrenzen und eine saubere Trennung von Verantwortlichkeiten.

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

* Nextcloud
* Betriebssystemen
* mobilen Plattformen
* Benutzeroberflächen
* konkreten Speichertechnologien
* externen Kommunikationssystemen

Stattdessen werden diese über definierte Schnittstellen angebunden.

---

## Architekturübersicht

```text id="5o2k4s"
                 Clients

      Linux   Windows   macOS
        |        |        |
      iOS     Android    Web

                 |
                 |
              API Layer

                 |
                 |

              NC-PoRe Core

                 |
                 |

        Interfaces / Adapters

                 |
    --------------------------------

    Storage       Communication
    Provider      Provider

    Nextcloud     Talk
    Local         BBB
    S3            Future Systems
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

> Welche Oberfläche benutzt der Benutzer?

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

Mögliche Clients:

* Linux Desktop
* Windows Desktop
* macOS Desktop
* iOS
* Android
* Web

Clients enthalten keine zentrale Geschäftslogik.

---

## Storage Module

Storage Provider kümmern sich um die technische Speicherung.

Beispiele:

* Nextcloud
* lokale Speicherung
* WebDAV
* S3-kompatible Speicher
* weitere zukünftige Provider

Der Core kennt nur das Storage Interface.

---

## Communication Module

Kommunikationssysteme werden als Provider angebunden.

Mögliche Beispiele:

* Nextcloud Talk
* BigBlueButton
* zukünftige Kommunikationssysteme

Die Produktionslogik bleibt unabhängig vom Kommunikationsweg.

---

# API First Prinzip

NC-PoRe wird API-orientiert entwickelt.

Dies bedeutet nicht zwingend, dass jede API öffentlich angeboten werden muss.

Die API dient als klare Grenze zwischen:

* Benutzeroberflächen
* externen Systemen
* Core-Logik

Dadurch können mehrere Clients dieselbe Basis verwenden.

---

# Dependency Rule

Abhängigkeiten fließen nur nach innen:

```text id="8e1g8w"
Client

  ↓

API

  ↓

Core

  ↓

Adapter / Provider
```

Der Core darf keine Infrastrukturdetails kennen.

---

# Grundprinzipien

## 1. Modularität statt Monolith

Neue Funktionen sollen möglichst durch neue Module ergänzt werden.

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

Ein stabiler Core ermöglicht langfristige Entwicklung.

---

## 3. Erweiterung durch Adapter

Neue Systeme werden über Adapter integriert.

Beispiele:

Nicht:

> NC-PoRe wird direkt an BigBlueButton angepasst.

Sondern:

> NC-PoRe erhält einen BigBlueButton-Adapter.

---

## 4. Nutzerorientierung

Technische Komplexität bleibt hinter klaren Schnittstellen verborgen.

Der Benutzer soll nicht wissen müssen:

* welcher Speicher verwendet wird
* welche Kommunikationstechnologie aktiv ist
* welcher Client verwendet wird

Die Erfahrung bleibt konsistent.

---

# Konsequenzen

## Vorteile

* langfristige Erweiterbarkeit
* mehrere Clients möglich
* Provider austauschbar
* bessere Testbarkeit
* geringere Abhängigkeiten
* einfachere Wartung

## Nachteile

* höherer initialer Entwicklungsaufwand
* zusätzliche Schnittstellen müssen gepflegt werden
* Architektur benötigt Disziplin

Diese Nachteile werden bewusst akzeptiert.

---

# Nicht-Ziele

Diese Entscheidung bedeutet nicht:

* dass alle Module sofort implementiert werden müssen
* dass jede Plattform direkt unterstützt werden muss
* dass jede Schnittstelle öffentlich dokumentiert werden muss
* dass die konkrete Programmiersprache festgelegt wird

Diese Entscheidungen folgen in späteren ADRs.

---

# Leitgedanke

NC-PoRe soll nicht nur funktionieren.

NC-PoRe soll wachsen können.

Ein stabiler Kern, klare Grenzen und austauschbare Module ermöglichen eine Software, die über Jahre weiterentwickelt werden kann.

---

# English (Deutsche Version oben)

## Context

NC-PoRe has been defined as a **Nextcloud Podcast Production Environment**.

Previous architecture decisions established:

* NC-PoRe is an environment for collaborative podcast production.
* The central domain entity is the **Production Session**.
* Media is managed as assets within sessions.
* Storage and external systems should remain replaceable.
* Multiple clients and platforms should be supported in the future.

To keep NC-PoRe maintainable and extensible over many years, clear module boundaries and responsibility separation are required.

---

## Decision

NC-PoRe will be built as a modular architecture.

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

* Nextcloud
* operating systems
* mobile platforms
* user interfaces
* storage technologies
* external communication systems

These systems connect through defined interfaces.

---

# Core Responsibility

The Core represents the domain truth of NC-PoRe.

It manages:

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

> Which user interface is used?

---

# API First Principle

NC-PoRe is developed API-oriented.

This does not necessarily mean every API must be publicly exposed.

The API provides a clear boundary between:

* user interfaces
* external systems
* core logic

Multiple clients can therefore share the same foundation.

---

# Dependency Rule

Dependencies only point inward:

```text id="y5z7u1"
Client

  ↓

API

  ↓

Core

  ↓

Adapter / Provider
```

The Core must not know infrastructure details.

---

# Principles

## 1. Modularity instead of Monolith

New functionality should preferably be added through new modules.

Existing core functionality should remain stable.

---

## 2. Boring Core

The Core should remain simple and long-lived.

It contains:

* models
* rules
* states

It does not contain:

* UI
* networking details
* filesystem access
* provider-specific logic

A stable Core enables long-term development.

---

## 3. Extension through Adapters

New systems are integrated through adapters.

Example:

Not:

> NC-PoRe is directly modified for BigBlueButton.

But:

> NC-PoRe receives a BigBlueButton adapter.

---

## 4. User Orientation

Technical complexity remains hidden behind clear interfaces.

Users should not need to know:

* which storage is used
* which communication technology is active
* which client is used

The experience remains consistent.

---

# Consequences

## Benefits

* long-term extensibility
* multiple clients possible
* replaceable providers
* better testability
* fewer dependencies
* easier maintenance

## Costs

* higher initial development effort
* additional interfaces require maintenance
* architecture requires discipline

These costs are consciously accepted.

---

# Non-Goals

This decision does not mean:

* all modules must be implemented immediately
* every platform must be supported immediately
* every interface must be publicly documented
* the programming language is already decided

These decisions will follow in later ADRs.

---

# Guiding Principle

NC-PoRe should not only work.

NC-PoRe should be able to grow.

A stable core, clear boundaries and replaceable modules enable software that can evolve over many years.
