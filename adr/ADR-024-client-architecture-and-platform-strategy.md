# ADR-024: Client Architecture and Platform Strategy

* Status: Accepted
* Date: 2026-07-23
* Decision Type: Architecture

---

# Deutsch (English version below)

## Kontext

NC-PoRe soll Menschen auf unterschiedlichen Geräten und Betriebssystemen ermöglichen, an Medien-Sessions teilzunehmen.

Die geplante Nutzung umfasst unter anderem:

* Desktop-Systeme
* mobile Geräte
* unterschiedliche Betriebssysteme
* verteilte Teilnehmer
* zukünftige Erweiterungen

Eine Architektur, die nur für eine einzelne Plattform entwickelt wird, würde zukünftige Entwicklungen erschweren.

Gleichzeitig soll vermieden werden, dass jede Plattform dieselbe Logik mehrfach implementieren muss.

---

## Entscheidung

NC-PoRe verwendet eine getrennte Core- und Client-Architektur.

Die zentrale Geschäfts- und Session-Logik wird in einem plattformunabhängigen Core umgesetzt.

Plattformabhängige Funktionen werden durch spezialisierte Clients bereitgestellt.

```text
                    NC-PoRe Core

                         |
        -------------------------------------
        |                  |                |
        v                  v                v

 Desktop Client     Mobile Client     weitere Clients

 Linux             iOS                Browser
 Windows           Android            zukünftige Plattformen
 macOS
```

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

Der Core kennt keine konkreten Plattformdetails.

---

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

Der Nutzer denkt nicht in:

* Desktop Client
* Mobile Client
* Provider

sondern in:

> Ich nehme an einer Session teil.

---

### 2. Mobile Plattformen sind gleichwertige Teilnehmer

Mobile Geräte sind keine nachträglichen Erweiterungen.

iOS und Android werden von Beginn an als Teil der Zielarchitektur betrachtet.

Dabei werden die besonderen Eigenschaften mobiler Systeme berücksichtigt:

* Energieverbrauch
* Berechtigungsmodelle
* Hardwarezugriffe
* Netzwerkbedingungen

---

### 3. Gemeinsame Logik statt mehrfacher Implementierung

Funktionalität soll möglichst nur einmal im Core implementiert werden.

Clients sollen diese Funktionen nutzen und nicht eigene Varianten entwickeln.

Dadurch werden ermöglicht:

* konsistentes Verhalten
* einfachere Wartung
* geringere Fehleranfälligkeit

---

### 4. Plattformfreiheit durch klare Schnittstellen

Neue Clients sollen ergänzt werden können, ohne die Kernarchitektur grundlegend zu verändern.

Beispiele:

* neue Betriebssysteme
* spezielle Geräte
* zukünftige Bedienkonzepte

---

## Konsequenzen

### Vorteile

* klare Trennung von Logik und Oberfläche
* bessere Wartbarkeit
* weniger doppelte Entwicklung
* einfachere Erweiterung neuer Plattformen
* Vorbereitung für verteilte Sessions

### Nachteile

* höherer initialer Architekturaufwand
* zusätzliche Schnittstellen
* komplexere Projektstruktur

Diese Nachteile werden bewusst akzeptiert.

Die langfristige Erweiterbarkeit ist wichtiger als eine kurzfristig einfachere Umsetzung.

---

## Nicht-Ziele

Diese Entscheidung legt nicht fest:

* welches konkrete UI-Framework verwendet wird
* welche Programmiersprache jeder Client verwendet
* welche Plattform zuerst vollständig umgesetzt wird

Diese Entscheidungen werden später getroffen.

---

## Leitgedanke

NC-PoRe soll Menschen verbinden, nicht Geräte.

Die technische Komplexität verschiedener Plattformen soll innerhalb des Systems gelöst werden.

Für den Nutzer bleibt die Erfahrung einfach:

> Eine Session. Ein System. Viele Möglichkeiten.

---

# English (Deutsche Version oben)

## Context

NC-PoRe should enable people using different devices and operating systems to participate in media sessions.

The planned usage includes:

* desktop systems
* mobile devices
* different operating systems
* distributed participants
* future extensions

An architecture designed for only one platform would make future development more difficult.

At the same time, the same logic should not have to be implemented multiple times for different platforms.

---

## Decision

NC-PoRe uses a separated Core and Client architecture.

Central business and session logic is implemented in a platform-independent Core.

Platform-specific functionality is provided through specialized Clients.

```text
                    NC-PoRe Core

                         |
        -------------------------------------
        |                  |                |
        v                  v                v

 Desktop Client     Mobile Client     Additional Clients

 Linux             iOS                Browser
 Windows           Android            Future platforms
 macOS
```

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

The Core does not know platform-specific details.

---

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

Users should not think in terms of:

* desktop client
* mobile client
* provider

but:

> I participate in a session.

---

### 2. Mobile platforms are first-class participants

Mobile devices are not later additions.

iOS and Android are considered part of the target architecture from the beginning.

Their specific characteristics are respected:

* energy consumption
* permission models
* hardware access
* network conditions

---

### 3. Shared logic instead of duplicated implementation

Functionality should be implemented once in the Core whenever possible.

Clients should consume this functionality instead of creating separate versions.

This enables:

* consistent behavior
* easier maintenance
* fewer errors

---

### 4. Platform independence through clear interfaces

New clients should be addable without fundamentally changing the Core architecture.

Examples:

* new operating systems
* specialized devices
* future interaction concepts

---

## Consequences

### Benefits

* clear separation of logic and interface
* better maintainability
* less duplicated development
* easier platform expansion
* preparation for distributed sessions

### Costs

* higher initial architecture effort
* additional interfaces
* more complex project structure

These costs are consciously accepted.

Long-term extensibility is more important than short-term simplicity.

---

## Non-Goals

This decision does not define:

* the specific UI framework
* the programming language of each client
* which platform will be completed first

These decisions will be made later.

---

## Guiding Principle

NC-PoRe should connect people, not devices.

The complexity of different platforms should be solved inside the system.

For users, the experience remains simple:

> One session. One system. Many possibilities.
