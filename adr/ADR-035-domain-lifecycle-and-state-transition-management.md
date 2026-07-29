# ADR-035: Domain Lifecycle and State Transition Management

- Status: Proposed
- Date: 2026-07-29
- Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

---

# Context

NC-PoRe verwaltet fachliche Objekte mit definierten Lebenszyklen.

Beispiele:

- Production Session
- Recording
- Synchronization
- Export
- Assets

Die bisherigen Architekturentscheidungen definieren:

- die Production Session als zentrale fachliche Einheit
- den Core als Domain Authority
- fachliche Regeln als Verantwortung des Core
- die Vermeidung ungültiger Systemzustände

Während der ersten Core-Implementierung wurde sichtbar,
dass Zustände nicht nur technische Eigenschaften von Objekten
sind.

Ein Zustand beschreibt eine fachliche Situation innerhalb
des Produktionsprozesses.

Daher müssen Zustände und deren Übergänge als Teil des
Domain-Modells behandelt werden.

---

# Decision

NC-PoRe modelliert fachliche Lebenszyklen explizit.

Der Core ist verantwortlich für:

- gültige fachliche Zustände
- erlaubte Zustandsübergänge
- Ablehnung ungültiger Übergänge
- Durchsetzung der Domain Rules während Zustandsänderungen

Zustandsänderungen erfolgen über definierte fachliche
Operationen (Domain Operations).

Externe Komponenten verändern fachliche Zustände nicht direkt.

---

# Architectural Principle

Ein Zustand ist nicht nur ein gespeicherter Wert.

Ein Zustand beschreibt eine fachliche Aussage über ein Objekt
innerhalb des Systems.

Daher gehören Zustandsänderungen zur fachlichen Logik.

Die Verantwortung dafür liegt im Core und nicht in:

- Clients
- Benutzeroberflächen
- Speicherkomponenten
- externen Providern

---

# Lifecycle Modeling

Lebenszyklen werden so modelliert, dass:

- erlaubte Übergänge nachvollziehbar sind
- ungültige Zustände verhindert werden
- fachliche Regeln zentral bleiben
- Änderungen durch Tests überprüfbar sind

Die konkrete technische Umsetzung eines Lebenszyklus ist
nicht Bestandteil dieser Entscheidung.

---

# Example: Production Session

Eine Production Session besitzt einen kontrollierten
fachlichen Lebenszyklus.

Beispiel:

Created

↓

Active

↓

Completed

Der Core definiert, welche Übergänge erlaubt sind.

Ein abgeschlossener Zustand darf nicht ohne definierte
fachliche Regel in einen früheren Zustand zurückgeführt werden.

---

# Consequences

## Positive Consequences

- fachlich ungültige Zustände werden verhindert
- Domain Rules bleiben zentral im Core
- Clients müssen keine Geschäftslogik duplizieren
- Lebenszyklen bleiben nachvollziehbar dokumentiert
- Tests können fachliche Regeln direkt überprüfen

---

## Negative Consequences

- zusätzliche Modellierung notwendig
- Zustandsübergänge müssen bewusst definiert werden
- einfache Änderungen von Zustandswerten sind nicht immer möglich

Diese Nachteile werden bewusst akzeptiert.

---

# Alternatives Considered

## Freie Zustandsänderung durch externe Komponenten

Nicht gewählt.

Begründung:

Dies würde die Verantwortung des Core als Domain Authority
schwächen und könnte zu inkonsistenten fachlichen Zuständen führen.

---

## Implizite Zustandslogik ohne explizites Lebenszyklusmodell

Nicht gewählt.

Begründung:

Lebenszyklen sind fachliche Regeln und müssen sichtbar,
nachvollziehbar und testbar sein.

---

# Relationship to Existing Architecture

Diese Entscheidung erweitert bestehende Architekturentscheidungen:

- ADR-019 Recording Session Data Model
- ADR-026 Session Data and Storage Architecture
- ADR-027 Core Architecture and Module Boundaries
- ADR-033 Core Architecture
- ADR-034 Implementation Architecture

Sie konkretisiert insbesondere die Verantwortung des Core
als fachliche Autorität für Zustände und deren Übergänge.

---

# Future Considerations

Weitere fachliche Lebenszyklen werden separat betrachtet:

- Recording Lifecycle
- Synchronization Lifecycle
- Export Lifecycle
- Asset Lifecycle

---

# Status

Diese Entscheidung definiert das grundlegende Muster
für fachliche Lebenszyklen innerhalb des NC-PoRe Core.

Die konkreten Zustandsmodelle einzelner Domänenobjekte
werden durch spätere Implementierungen und Entscheidungen
festgelegt.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# Context

NC-PoRe manages domain objects with defined lifecycles.

Examples:

- Production Session
- Recording
- Synchronization
- Export
- Assets

Previous architecture decisions define:

- Production Session as the central domain entity
- the Core as Domain Authority
- domain rules as responsibility of the Core
- prevention of invalid system states

During the first Core implementation it became clear
that states are not merely technical object properties.

A state represents a domain condition within the production
workflow.

Therefore, states and their transitions must be treated
as part of the domain model.

---

# Decision

NC-PoRe explicitly models domain lifecycles.

The Core is responsible for:

- valid domain states
- allowed state transitions
- rejection of invalid transitions
- enforcement of Domain Rules during state changes

State changes happen through defined domain operations.

External components do not directly modify domain states.

---

# Architectural Principle

A state is not merely a stored value.

A state represents a domain statement about an object
within the system.

Therefore, state changes belong to domain logic.

Responsibility lies in the Core and not in:

- clients
- user interfaces
- storage components
- external providers

---

# Status

This decision defines the general pattern
for domain lifecycles within the NC-PoRe Core.

The concrete state models of individual domain objects
will be defined through later implementations and decisions.
