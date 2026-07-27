# ADR-022: Modulare Architektur und Provider-Design

* Status: Accepted
* Date: 2026-07-23
* Decision Type: Architecture

---

# Deutsch

## Kontext

NC-PoRe soll langfristig nicht nur ein einfacher Recorder sein, sondern ein offenes, erweiterbares System zur Erstellung und Verwaltung von Medien-Sessions.

Die Architektur soll bereits in der ersten Version zukünftige Anforderungen ermöglichen:

* mehrere Teilnehmer und Geräte
* unterschiedliche Betriebssysteme und Clients
* verschiedene Speicheranbieter
* Konferenzintegrationen
* zusätzliche Medienarten wie Video oder Bildschirmaufnahmen
* professionelle Erweiterungsmodule

Eine stark gekoppelte Architektur würde spätere Erweiterungen erschweren und hohe Änderungskosten verursachen.

---

## Entscheidung

NC-PoRe wird nach dem Prinzip einer modularen Architektur entwickelt.

Funktionale Bereiche werden durch klar definierte Schnittstellen getrennt.

Konkrete Technologien werden über austauschbare Provider angebunden.

Beispiele:

```
StorageProvider

├── Nextcloud Storage
├── Local Storage
├── WebDAV Storage
└── weitere Speicheranbieter
```

```
ConferenceProvider

├── Nextcloud Talk
├── BigBlueButton
├── Jitsi
└── weitere Systeme
```

```
MediaProvider

├── Audio
├── Video
├── Screen Capture
└── weitere Medienquellen
```

Die Kernlogik von NC-PoRe soll nicht von einzelnen Technologien abhängig sein.

---

## Grundprinzipien

### 1. Session statt Datei

NC-PoRe betrachtet eine Aufnahme nicht als einzelne Datei, sondern als Session.

Eine Session kann enthalten:

* Teilnehmer
* Medienströme
* Metadaten
* Ereignisse
* Exportinformationen

Dadurch bleibt die Architektur offen für Multi-Participant- und Multi-Media-Anwendungen.

---

### 2. Klare Verantwortlichkeiten

Jedes Modul besitzt eine klar definierte Aufgabe.

Beispiele:

* Session-Modul:
  Verwaltung des Lebenszyklus einer Aufnahme

* Media-Modul:
  Verarbeitung von Audio-, Video- und weiteren Medienquellen

* Metadata-Modul:
  Verwaltung zusätzlicher Informationen

* Storage-Modul:
  Speicherung und Abruf von Daten

* Export-Modul:
  Bereitstellung verschiedener Ausgabeformate

Module sollen möglichst unabhängig voneinander weiterentwickelt werden können.

---

### 3. Erweiterbarkeit durch Schnittstellen

Neue Funktionen sollen bevorzugt durch neue Module oder Provider ergänzt werden.

Bestehende Kernfunktionen sollen möglichst stabil bleiben.

Erweiterungen sollen nicht dazu führen, dass Anwender oder Entwickler die gesamte Architektur verstehen müssen.

---

## Konsequenzen

### Vorteile

* bessere Wartbarkeit
* einfachere Erweiterbarkeit
* geringere Abhängigkeit von einzelnen Technologien
* bessere Möglichkeiten für Community-Beiträge
* Vorbereitung auf zukünftige Plattformen

### Nachteile

* höherer initialer Entwicklungsaufwand
* zusätzliche Abstraktionsebenen
* komplexere Architektur am Anfang

Diese Nachteile werden bewusst akzeptiert, da langfristige Wartbarkeit wichtiger ist als kurzfristige Entwicklungsoptimierung.

---

## Nicht-Ziele

Diese Entscheidung bedeutet nicht:

* dass jede mögliche Plattform sofort unterstützt wird
* dass jede Schnittstelle von Anfang an vollständig implementiert wird
* dass unnötige Abstraktionen eingeführt werden

Die Architektur soll Möglichkeiten schaffen, nicht unnötige Komplexität erzeugen.

---

## Leitgedanke

NC-PoRe soll nicht als einzelner Recorder entwickelt werden.

NC-PoRe ist ein offenes Session-System, das Menschen, Geräte, Medienquellen und Speicherorte miteinander verbinden kann.

Komplexität soll innerhalb des Systems gelöst werden und nicht beim Anwender entstehen.

---

# English

## Context

NC-PoRe is not intended to become only a simple recorder.

The long-term goal is an open and extensible system for creating and managing media sessions.

The architecture should support future requirements such as:

* multiple participants and devices
* different operating systems and clients
* different storage providers
* conference integrations
* additional media types such as video or screen capture
* professional extension modules

A tightly coupled architecture would make future extensions difficult and expensive.

---

## Decision

NC-PoRe will follow a modular architecture approach.

Functional areas are separated through clearly defined interfaces.

Concrete technologies are connected through replaceable providers.

Examples:

```
StorageProvider

├── Nextcloud Storage
├── Local Storage
├── WebDAV Storage
└── additional providers
```

```
ConferenceProvider

├── Nextcloud Talk
├── BigBlueButton
├── Jitsi
└── additional systems
```

```
MediaProvider

├── Audio
├── Video
├── Screen Capture
└── additional media sources
```

The core logic of NC-PoRe should not depend on individual technologies.

---

## Principles

### 1. Session instead of file

NC-PoRe treats a recording not as a single file, but as a session.

A session may contain:

* participants
* media streams
* metadata
* events
* export information

This keeps the architecture open for multi-participant and multi-media scenarios.

---

### 2. Clear responsibilities

Each module has a clearly defined responsibility.

Examples:

* Session module:
  manages recording lifecycle

* Media module:
  handles audio, video and other media sources

* Metadata module:
  manages additional information

* Storage module:
  stores and retrieves data

* Export module:
  provides output formats

Modules should remain independently maintainable whenever possible.

---

### 3. Extension through interfaces

New functionality should preferably be added through new modules or providers.

Existing core functionality should remain stable whenever possible.

Extensions should not require users or developers to understand the complete system architecture.

---

## Consequences

### Benefits

* improved maintainability
* easier extension
* reduced dependency on individual technologies
* better support for community contributions
* preparation for future platforms

### Costs

* higher initial development effort
* additional abstraction layers
* more complex architecture at the beginning

These costs are accepted because long-term maintainability is more important than short-term development speed.

---

## Non-Goals

This decision does not mean:

* supporting every possible platform immediately
* implementing every interface from the beginning
* introducing unnecessary abstractions

The architecture should create possibilities, not unnecessary complexity.

---

## Guiding Principle

NC-PoRe is not developed as a single recorder.

NC-PoRe is an open session system connecting people, devices, media sources and storage locations.

Complexity should be handled inside the system, not transferred to the user.
