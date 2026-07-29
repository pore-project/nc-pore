# ADR-038 Core Implementation Structure and Module Organization

* Status: Proposed
* Date: 2026-07-29
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

---

# Kontext

NC-PoRe beginnt nach Abschluss der Architekturphase
mit der schrittweisen technischen Umsetzung.

Die bisherigen Architekturentscheidungen definieren:

* den Core als fachliche Autorität
* die Trennung zwischen Domäne und Infrastruktur
* fachliche Verantwortlichkeiten
* kontrollierte Lebenszyklen
* nachvollziehbare Entwicklungsprozesse

Mit der konkreten Implementierung entsteht die Frage,
wie der Core intern strukturiert wird.

Die interne Struktur des Core muss:

* fachliche Verantwortlichkeiten sichtbar machen
* klare Modulgrenzen schaffen
* Abhängigkeiten kontrollieren
* Tests ermöglichen
* zukünftige Erweiterungen unterstützen

Die Struktur darf nicht primär durch technische
Implementierungsdetails bestimmt werden.

---

# Entscheidung

NC-PoRe strukturiert den Core nach fachlichen
Verantwortungsbereichen.

Der Core wird in eigenständige Domain-Module aufgeteilt.

Die Modulstruktur orientiert sich an fachlichen
Konzepten und nicht an technischen Schichten.

Grundstruktur:

```text
core

|
+-- session
|
+-- participant
|
+-- participation
|
+-- role
|
+-- identity
|
+-- activity
|
+-- future domain modules
```

Jedes Modul besitzt eine klar definierte Verantwortung.

---

# Core Responsibility

Der Core enthält ausschließlich fachliche Logik.

Dazu gehören:

* Domain-Modelle
* fachliche Regeln
* Zustandsübergänge
* Validierungen
* fachliche Ereignisse

Nicht Bestandteil des Core sind:

* Datenbankzugriffe
* Dateispeicherung
* Netzwerkkommunikation
* Benutzeroberflächen
* konkrete externe Dienste

---

# Modulverantwortlichkeiten

## Session Module

Verantwortlich für:

* Production Session
* Session Lifecycle
* erlaubte Zustandsübergänge
* sessionbezogene Regeln

Die Session ist die zentrale fachliche Einheit
einer Produktion.

---

## Participant Module

Verantwortlich für:

* Teilnehmerobjekte
* Teilnehmeridentität innerhalb einer Produktion
* teilnehmerbezogene fachliche Informationen

---

## Participation Module

Verantwortlich für:

* Beziehung zwischen Participant und Session
* Teilnahme an einer Produktion
* Zuordnung fachlicher Rollen

---

## Role Module

Verantwortlich für:

* fachliche Rollen
* Rollenwerte
* rollenbezogene Domain-Konzepte

Das Modul definiert keine technische Autorisierung.

---

## Identity Module

Verantwortlich für:

* stabile Identitäten fachlicher Objekte
* Identifikationswerte

Identity entscheidet nicht über:

* Berechtigungen
* Zugriff
* Benutzerrechte

---

## Activity Module

Verantwortlich für:

* Produktionshistorie
* fachliche Ereignisse
* nachvollziehbare Zustandsänderungen

Activity dokumentiert Ereignisse,
entscheidet jedoch nicht über fachliche Regeln.

---

# Dependency Rules

Die Abhängigkeiten zwischen Core-Modulen
folgen klaren Regeln.

Grundprinzip:

```text
Domain Rules

↓

Domain Modules

↓

Infrastructure Boundaries
```

Core-Module dürfen voneinander abhängig sein,
wenn die fachliche Beziehung dies erfordert.

Nicht erlaubt sind Abhängigkeiten zu:

* Datenbanken
* Dateisystemen
* Netzwerkdiensten
* UI-Komponenten
* externen Providern

---

# Public and Internal Interfaces

Module definieren bewusst ihre Grenzen.

Ein Modul stellt nur diejenigen Bestandteile öffentlich
bereit, die andere Domain-Module benötigen.

Interne Implementierungsdetails bleiben verborgen.

Beispiel:

```text
session

public:
- ProductionSession
- Lifecycle Operations

internal:
- interne Validierungsdetails
- Hilfsfunktionen
```

Dadurch bleiben Änderungen innerhalb eines Moduls
möglich, ohne unnötige Auswirkungen auf andere Bereiche.

---

# Testing Consequences

Die Modulstruktur ermöglicht:

* isolierte Tests einzelner Fachbereiche
* Prüfung von Lifecycle-Regeln
* Prüfung von Domain-Validierungen
* kleinere Testeinheiten

Tests orientieren sich an fachlichen Anforderungen,
nicht an technischen Implementierungsdetails.

---

# Alternatives Considered

## Technical Layer Based Core Structure

Nicht gewählt.

Begründung:

Eine Struktur nach technischen Schichten würde
fachliche Zusammenhänge verteilen.

Beispiel:

```text
models

services

controllers

repositories
```

würde technische Kategorien über fachliche
Verantwortlichkeiten stellen.

---

## Single Large Core Module

Nicht gewählt.

Begründung:

Ein einzelnes großes Modul würde langfristig:

* Verantwortlichkeiten vermischen
* Änderungen erschweren
* Tests komplizierter machen

---

# Beziehung zu bestehenden Architekturentscheidungen

Diese Entscheidung erweitert:

* ADR-027 Core Architecture and Module Boundaries
* ADR-033 Core Architecture
* ADR-034 Implementation Architecture
* ADR-035 Domain Lifecycle and State Transition Management
* ADR-036 Persistence Boundary and Storage Strategy
* ADR-037 Development Workflow and Source of Truth

ADR-038 definiert die interne Struktur des Core
für die technische Umsetzung.

---

# Future Considerations

Weitere Domain-Module werden erst eingeführt,
wenn konkrete fachliche Anforderungen entstehen.

Mögliche zukünftige Module:

* Recording
* Asset Management
* Synchronization
* Export
* Production Workflow

Diese Module werden nicht vorzeitig erzeugt,
sondern aus tatsächlichen Anforderungen entwickelt.

---

# Status

Diese Entscheidung definiert die grundlegende
Modulstruktur des NC-PoRe Core.

Die konkrete Implementierung erfolgt schrittweise
innerhalb dieser Architekturgrenzen.

---

# English Version

---

# Context

NC-PoRe starts gradual technical implementation after
completion of the architecture phase.

Previous architecture decisions define:

* the Core as domain authority
* separation between domain and infrastructure
* domain responsibilities
* controlled lifecycles
* traceable development processes

Implementation requires a decision about the internal
structure of the Core.

The Core structure must:

* expose domain responsibilities
* create clear module boundaries
* control dependencies
* enable testing
* support future extensions

The structure must not primarily be driven by technical
implementation details.

---

# Decision

NC-PoRe structures the Core according to domain
responsibilities.

The Core is divided into independent domain modules.

The module structure follows domain concepts instead of
technical layers.

Basic structure:

```text
core

|
+-- session
|
+-- participant
|
+-- participation
|
+-- role
|
+-- identity
|
+-- activity
|
+-- future domain modules
```

Each module has a clearly defined responsibility.

---

# Core Responsibility

The Core contains domain logic only.

This includes:

* domain models
* domain rules
* state transitions
* validations
* domain events

The Core does not contain:

* database access
* file storage
* network communication
* user interfaces
* concrete external services

---

# Module Responsibilities

## Session Module

Responsible for:

* Production Session
* session lifecycle
* valid state transitions
* session-related rules

The session is the central domain unit
of a production.

---

## Participant Module

Responsible for:

* participant objects
* participant identity within a production
* participant-related domain information

---

## Participation Module

Responsible for:

* relationship between Participant and Session
* participation in a production
* assignment of domain roles

---

## Role Module

Responsible for:

* domain roles
* role values
* role-related domain concepts

The module does not define technical authorization.

---

## Identity Module

Responsible for:

* stable identities of domain objects
* identification values

Identity does not decide:

* permissions
* access
* user rights

---

## Activity Module

Responsible for:

* production history
* domain events
* traceable state changes

Activity records events but does not define
domain rules.

---

# Dependency Rules

Dependencies between Core modules follow clear rules.

Principle:

```text
Domain Rules

↓

Domain Modules

↓

Infrastructure Boundaries
```

Core modules may depend on each other when required
by domain relationships.

Dependencies to the following are not allowed:

* databases
* file systems
* network services
* UI components
* external providers

---

# Public and Internal Interfaces

Modules define their boundaries explicitly.

A module exposes only elements required by other
domain modules.

Internal implementation details remain hidden.

Example:

```text
session

public:
- ProductionSession
- Lifecycle Operations

internal:
- internal validation details
- helper functions
```

This allows changes inside modules without unnecessary
impact on other areas.

---

# Testing Consequences

The module structure enables:

* isolated domain tests
* lifecycle validation
* domain validation testing
* smaller test units

Tests follow domain requirements rather than technical
implementation details.

---

# Alternatives Considered

## Technical Layer Based Core Structure

Rejected.

Reason:

A technical layer structure would distribute domain
relationships.

Example:

```text
models

services

controllers

repositories
```

would place technical categories above domain
responsibilities.

---

## Single Large Core Module

Rejected.

Reason:

A single large module would eventually:

* mix responsibilities
* make changes harder
* complicate testing

---

# Relationship to Existing Architecture

This decision extends:

* ADR-027 Core Architecture and Module Boundaries
* ADR-033 Core Architecture
* ADR-034 Implementation Architecture
* ADR-035 Domain Lifecycle and State Transition Management
* ADR-036 Persistence Boundary and Storage Strategy
* ADR-037 Development Workflow and Source of Truth

ADR-038 defines the internal structure of the Core
for technical implementation.

---

# Future Considerations

Additional domain modules are introduced only when
concrete domain requirements arise.

Possible future modules:

* Recording
* Asset Management
* Synchronization
* Export
* Production Workflow

These modules are not created prematurely,
but developed from actual requirements.

---

# Status

This decision defines the fundamental module structure
of the NC-PoRe Core.

Concrete implementation proceeds incrementally
within these architectural boundaries.
