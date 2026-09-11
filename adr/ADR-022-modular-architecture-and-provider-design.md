# Deutsch ([English version below](#english-version))

# ADR-022: Modulare Architektur und Provider-Design

* Status: Accepted
* Date: 2026-07-23
* Decision Type: Architecture

---

## Kontext

NC-PoRe benötigt eine modulare Architektur, in der fachliche Bereiche und technische Integrationen klar voneinander getrennt sind. Eine stark gekoppelte Architektur würde Änderungen und Erweiterungen unnötig erschweren.

---

## Entscheidung

NC-PoRe wird nach dem Prinzip einer modularen Architektur entwickelt.

Funktionale Bereiche werden durch klar definierte Schnittstellen getrennt. Konkrete Technologien werden über austauschbare Provider angebunden.

Konzeptionell:

```text
NC-PoRe Core
   |
   +-- Session / Domain
   +-- Media
   +-- Metadata
   +-- Storage Provider
   +-- Export
   +-- Host / Integration Provider
```

Die Kernlogik von NC-PoRe darf nicht von einzelnen Technologien abhängig sein.

---

## Grundprinzipien

### 1. Session statt Datei

NC-PoRe betrachtet eine Aufnahme als Teil einer Session. Eine Session kann Teilnehmer, Medienströme, Metadaten, Ereignisse und Exportinformationen enthalten.

### 2. Klare Verantwortlichkeiten

Jedes Modul besitzt eine klar definierte Aufgabe. Module sollen möglichst unabhängig voneinander weiterentwickelt werden können.

### 3. Provider hinter Schnittstellen

Technologie- und integrationsspezifische Funktionen werden hinter definierten Grenzen gekapselt. Der Core arbeitet mit fachlichen bzw. technischen Fähigkeiten und nicht mit den internen APIs einzelner Anbieter.

### 4. Erweiterbarkeit ohne unnötige Komplexität

Neue Funktionen sollen bevorzugt durch klar abgegrenzte Module oder Provider ergänzt werden. Bestehende Kernfunktionen sollen möglichst stabil bleiben.

---

## Konsequenzen

### Vorteile

* bessere Wartbarkeit
* einfachere Erweiterbarkeit
* geringere Abhängigkeit von einzelnen Technologien
* gute Voraussetzungen für Community-Beiträge

### Nachteile

* höherer initialer Entwicklungsaufwand
* zusätzliche Abstraktionsebenen
* komplexere Projektstruktur

Diese Nachteile werden bewusst akzeptiert.

---

## Nicht-Ziele

Diese Entscheidung bedeutet nicht:

* dass jede denkbare Technologie unterstützt werden muss
* dass jede Schnittstelle sofort vollständig implementiert wird
* dass Abstraktionen ohne konkreten Bedarf eingeführt werden

Die Architektur soll Möglichkeiten schaffen, nicht unnötige Komplexität erzeugen.

---

## Leitgedanke

NC-PoRe ist ein offenes Session-System. Fachliche Logik und technische Integrationen werden so getrennt, dass Änderungen an einzelnen Technologien nicht die gesamte Architektur bestimmen.

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-022: Modular Architecture and Provider Design

* Status: Accepted
* Date: 2026-07-23
* Decision Type: Architecture

---

## Context

NC-PoRe requires a modular architecture in which domain areas and technical integrations are clearly separated. A tightly coupled architecture would make changes and extensions unnecessarily difficult.

---

## Decision

NC-PoRe follows a modular architecture approach.

Functional areas are separated through clearly defined interfaces. Concrete technologies are connected through replaceable providers.

Conceptually:

```text
NC-PoRe Core
   |
   +-- Session / Domain
   +-- Media
   +-- Metadata
   +-- Storage Provider
   +-- Export
   +-- Host / Integration Provider
```

The core logic of NC-PoRe must not depend on individual technologies.

---

## Principles

### 1. Session instead of file

NC-PoRe treats a recording as part of a session. A session may contain participants, media streams, metadata, events and export information.

### 2. Clear responsibilities

Each module has a clearly defined responsibility. Modules should remain independently maintainable whenever possible.

### 3. Providers behind interfaces

Technology- and integration-specific functionality is encapsulated behind defined boundaries. The Core works with domain or technical capabilities rather than the internal APIs of individual providers.

### 4. Extensibility without unnecessary complexity

New functionality should preferably be added through clearly separated modules or providers. Existing core functionality should remain stable whenever possible.

---

## Consequences

### Benefits

* improved maintainability
* easier extension
* reduced dependency on individual technologies
* good support for community contributions

### Costs

* higher initial development effort
* additional abstraction layers
* more complex project structure

These costs are consciously accepted.

---

## Non-Goals

This decision does not mean:

* every conceivable technology must be supported
* every interface must be fully implemented immediately
* abstractions should be introduced without a concrete need

The architecture should create possibilities, not unnecessary complexity.

---

## Guiding Principle

NC-PoRe is an open session system. Domain logic and technical integrations are separated so that individual technology choices do not determine the entire architecture.
