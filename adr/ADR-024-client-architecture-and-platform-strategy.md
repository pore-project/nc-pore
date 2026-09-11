<a id="deutsch"></a>

# Deutsch

# ADR-024: Client Architecture and Platform Strategy

* Status: Accepted
* Date: 2026-07-23
* Decision Type: Architecture

---

## Kontext

NC-PoRe soll Menschen auf unterschiedlichen Geräten und Betriebssystemen ermöglichen, an Sessions teilzunehmen. Eine Architektur, die nur für eine einzelne Plattform entwickelt wird, würde Erweiterungen erschweren. Gleichzeitig soll gemeinsame fachliche Logik nicht mehrfach implementiert werden.

---

## Entscheidung

NC-PoRe verwendet eine getrennte Core- und Client-Architektur.

Die zentrale Geschäfts- und Session-Logik wird in einem plattformunabhängigen Core umgesetzt. Plattformabhängige Funktionen werden durch spezialisierte Clients bereitgestellt.

```text
                    NC-PoRe Core
                         |
        -------------------------------------
        |                  |                |
        v                  v                v
   Client A            Client B       weitere Clients
```

Der Core kennt keine konkreten Plattformdetails.

---

## Verantwortlichkeiten

### NC-PoRe Core

Der Core ist verantwortlich für:

* Session-Modell
* Session-Lifecycle
* Medienmodell
* Metadaten
* Synchronisationslogik
* Provider-Schnittstellen
* zentrale Geschäftslogik

### Clients

Clients sind verantwortlich für:

* Benutzeroberfläche
* Bedienkonzepte
* Plattformintegration
* Zugriff auf Gerätehardware
* lokale Berechtigungen
* Betriebssystem-spezifische Funktionen

---

## Grundprinzipien

### 1. Ein System, viele Clients

Für Anwender soll NC-PoRe unabhängig vom verwendeten Gerät als ein zusammenhängendes System erscheinen.

### 2. Gemeinsame Logik statt mehrfacher Implementierung

Funktionalität soll möglichst nur einmal im Core implementiert werden. Clients nutzen diese Funktionen, statt eigene fachliche Varianten zu entwickeln.

### 3. Plattformfreiheit durch klare Schnittstellen

Neue Clients sollen ergänzt werden können, ohne die Kernarchitektur grundlegend zu verändern.

---

## Konsequenzen

### Vorteile

* klare Trennung von Logik und Oberfläche
* bessere Wartbarkeit
* weniger doppelte Entwicklung
* einfachere Erweiterung weiterer Clients

### Nachteile

* höherer initialer Architekturaufwand
* zusätzliche Schnittstellen
* komplexere Projektstruktur

Diese Nachteile werden bewusst akzeptiert.

---

## Nicht-Ziele

Diese Entscheidung legt nicht fest:

* welches konkrete UI-Framework verwendet wird
* welche Programmiersprache ein Client verwendet
* welche konkreten Plattformen unterstützt werden

Solche Entscheidungen gehören in die jeweiligen technischen oder produktspezifischen Entscheidungen.

---

## Leitgedanke

NC-PoRe soll Menschen verbinden, nicht Geräte. Die technische Komplexität verschiedener Plattformen soll innerhalb des Systems gelöst werden.

---

<a id="english-version"></a>

# English Version

# ADR-024: Client Architecture and Platform Strategy

* Status: Accepted
* Date: 2026-07-23
* Decision Type: Architecture

---

## Context

NC-PoRe should enable people using different devices and operating systems to participate in sessions. An architecture designed for only one platform would make extensions more difficult. At the same time, shared domain logic should not have to be implemented multiple times.

---

## Decision

NC-PoRe uses a separated Core and Client architecture.

Central business and session logic is implemented in a platform-independent Core. Platform-specific functionality is provided through specialized Clients.

```text
                    NC-PoRe Core
                         |
        -------------------------------------
        |                  |                |
        v                  v                v
     Client A           Client B       additional clients
```

The Core does not know platform-specific details.

---

## Responsibilities

### NC-PoRe Core

The Core is responsible for:

* session model
* session lifecycle
* media model
* metadata
* synchronization logic
* provider interfaces
* central business logic

### Clients

Clients are responsible for:

* user interface
* interaction concepts
* platform integration
* hardware access
* local permissions
* operating-system-specific functions

---

## Principles

### 1. One system, many clients

For users, NC-PoRe should appear as one consistent system regardless of the device being used.

### 2. Shared logic instead of duplicated implementation

Functionality should be implemented once in the Core whenever possible. Clients should consume this functionality instead of creating separate domain variants.

### 3. Platform independence through clear interfaces

New clients should be addable without fundamentally changing the Core architecture.

---

## Consequences

### Benefits

* clear separation of logic and interface
* better maintainability
* less duplicated development
* easier addition of further clients

### Costs

* higher initial architecture effort
* additional interfaces
* more complex project structure

These costs are consciously accepted.

---

## Non-Goals

This decision does not define:

* the specific UI framework
* the programming language of a client
* which specific platforms will be supported

Such decisions belong in the respective technical or product-specific decisions.

---

## Guiding Principle

NC-PoRe should connect people, not devices. The complexity of different platforms should be solved inside the system.
