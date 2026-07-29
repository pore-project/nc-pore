# ADR-036 Persistence Boundary and Storage Strategy

* Status: Proposed
* Date: 2026-07-29
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

---

# Kontext

NC-PoRe verwendet den Core als fachliche Autorität
(Domain Authority).

Die bisherigen Architekturentscheidungen definieren:

* zentrale fachliche Modelle
* Verantwortlichkeiten des Core
* Lebenszyklen fachlicher Objekte
* Trennung zwischen Domäne und technischen Komponenten

Mit Beginn der technischen Umsetzung entsteht die Frage,
wie fachliche Objekte dauerhaft gespeichert und wieder geladen
werden.

Die Speicherung darf die fachliche Architektur nicht bestimmen.

Insbesondere darf der Core nicht von konkreten technischen
Speicherlösungen abhängig werden.

---

# Entscheidung

NC-PoRe definiert eine klare Grenze zwischen fachlicher Logik
und Persistenz.

Der Core besitzt keine direkte Abhängigkeit zu:

* Datenbanken
* Dateisystemen
* Nextcloud-Speicherstrukturen
* externen Speicherprovidern
* konkreten Serialisierungsformaten

Persistenz wird über definierte Schnittstellen angebunden.

---

# Architectural Principle

Die Domäne beschreibt:

* welche Daten fachlich existieren
* welche Regeln gelten
* welche Zustände erlaubt sind

Die Persistenz beschreibt:

* wie Daten gespeichert werden
* wo Daten gespeichert werden
* wie Daten technisch übertragen werden

Diese Verantwortlichkeiten bleiben getrennt.

---

# Persistence Boundary

Die Grenze zwischen Core und Speicherung wird explizit
modelliert.

Beispiel:

```text
Core

ProductionSession

        |
        |
        v

Persistence Boundary

        |
        |
        v

Storage Implementation
```

Der Core kennt die fachliche Operation.

Die konkrete Speicherung bleibt austauschbar.

---

# Repository Responsibility

Persistenzkomponenten sind verantwortlich für:

* Speichern fachlicher Objekte
* Laden fachlicher Objekte
* technische Fehlerbehandlung der Speicherung

Sie sind nicht verantwortlich für:

* fachliche Regeln
* Lifecycle-Entscheidungen
* Berechtigungen
* Validierung fachlicher Zustände

Diese Verantwortung bleibt im Core.

---

# Technology Independence

Die Auswahl konkreter Speichertechnologien erfolgt
unabhängig von der Domänenarchitektur.

Mögliche Implementierungen können sein:

* lokale Dateien
* relationale Datenbanken
* objektbasierte Speicher
* Nextcloud-basierte Speicherung

Die Auswahl einer Technologie darf keine Änderung
an fachlichen Modellen erzwingen.

---

# Lifecycle and Persistence

Persistenz speichert Zustände.

Sie entscheidet jedoch nicht über gültige Zustandsübergänge.

Beispiel:

```text
Created

↓

Active

↓

Completed
```

Die Entscheidung, ob ein Übergang erlaubt ist,
bleibt Bestandteil der Core-Logik.

Die Speicherung hält lediglich den gültigen Zustand fest.

---

# Consequences

## Positive Consequences

* Domänenlogik bleibt unabhängig von Infrastruktur
* Speichertechnologien können ausgetauscht werden
* Tests können ohne reale Infrastruktur durchgeführt werden
* Architekturgrenzen bleiben nachvollziehbar
* zukünftige Skalierungsmöglichkeiten bleiben offen

---

## Negative Consequences

* zusätzliche Schnittstellen notwendig
* mehr initialer Modellierungsaufwand
* einfache direkte Speicherung wird vermieden

Diese Nachteile werden bewusst akzeptiert.

Die langfristige Wartbarkeit und Erweiterbarkeit
überwiegt den zusätzlichen Anfangsaufwand.

---

# Alternatives Considered

## Direct Storage Access from Core

Nicht gewählt.

Begründung:

Eine direkte Abhängigkeit des Core von Speichertechnologien
würde die fachliche Architektur mit Infrastrukturdetails
vermischen.

---

## Storage-driven Domain Model

Nicht gewählt.

Begründung:

Das Datenmodell der Speicherung darf nicht die fachlichen
Modelle bestimmen.

---

# Relationship to Existing Architecture

Diese Entscheidung erweitert:

* ADR-026 Session Data and Storage Architecture
* ADR-027 Core Architecture and Module Boundaries
* ADR-033 Core Architecture
* ADR-034 Implementation Architecture
* ADR-035 Domain Lifecycle and State Transition Management

Sie konkretisiert insbesondere die Trennung zwischen
fachlicher Logik und technischer Infrastruktur.

---

# Future Considerations

Konkrete Persistenzimplementierungen werden separat betrachtet.

Mögliche zukünftige Entscheidungen:

* Datenbankauswahl
* Serialisierungsformat
* Repository-Implementierungen
* Synchronisationsspeicher

Diese Entscheidungen erfolgen erst,
wenn konkrete technische Anforderungen bestehen.

---

# Status

Diese Entscheidung definiert die grundlegende Grenze
zwischen Core und Persistenz innerhalb von NC-PoRe.

Die konkrete technische Speicherung wird durch spätere
Implementierungen und Entscheidungen festgelegt.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# Context

NC-PoRe uses the Core as the Domain Authority.

Previous architecture decisions define:

* central domain models
* Core responsibilities
* lifecycles of domain objects
* separation between domain and technical components

With the beginning of technical implementation,
the question arises how domain objects are stored
and restored.

Persistence must not define the domain architecture.

The Core must not depend on specific storage technologies.

---

# Decision

NC-PoRe defines a clear boundary between domain logic
and persistence.

The Core has no direct dependency on:

* databases
* file systems
* Nextcloud storage structures
* external storage providers
* concrete serialization formats

Persistence is connected through defined interfaces.

---

# Architectural Principle

The domain defines:

* which data exists
* which rules apply
* which states are valid

Persistence defines:

* how data is stored
* where data is stored
* how data is technically transferred

These responsibilities remain separated.

---

# Persistence Boundary

The boundary between Core and storage is explicitly modeled.

Example:

```text
Core

ProductionSession

        |
        |
        v

Persistence Boundary

        |
        |
        v

Storage Implementation
```

The Core knows domain operations.

The concrete storage remains replaceable.

---

# Repository Responsibility

Persistence components are responsible for:

* storing domain objects
* loading domain objects
* technical storage error handling

They are not responsible for:

* domain rules
* lifecycle decisions
* permissions
* validation of domain states

These responsibilities remain in the Core.

---

# Technology Independence

The selection of concrete storage technologies
is independent from domain architecture.

Possible implementations include:

* local files
* relational databases
* object storage
* Nextcloud-based storage

Technology choices must not require changes
to domain models.

---

# Lifecycle and Persistence

Persistence stores states.

It does not decide valid state transitions.

Example:

```text
Created

↓

Active

↓

Completed
```

The decision whether a transition is allowed
remains part of Core logic.

Persistence only stores the valid state.

---

# Consequences

## Positive Consequences

* domain logic remains independent from infrastructure
* storage technologies can be replaced
* tests can run without real infrastructure
* architecture boundaries remain traceable
* future scalability options remain open

---

## Negative Consequences

* additional interfaces are required
* more initial modeling effort
* direct simple storage is avoided

These disadvantages are consciously accepted.

Long-term maintainability and extensibility
outweigh the additional initial effort.

---

# Alternatives Considered

## Direct Storage Access from Core

Rejected.

Reason:

Direct Core dependencies on storage technologies
would mix domain architecture with infrastructure details.

---

## Storage-driven Domain Model

Rejected.

Reason:

Storage models must not define domain models.

---

# Relationship to Existing Architecture

This decision extends:

* ADR-026 Session Data and Storage Architecture
* ADR-027 Core Architecture and Module Boundaries
* ADR-033 Core Architecture
* ADR-034 Implementation Architecture
* ADR-035 Domain Lifecycle and State Transition Management

It further defines the separation between
domain logic and technical infrastructure.

---

# Future Considerations

Concrete persistence implementations are considered separately.

Possible future decisions:

* database selection
* serialization format
* repository implementations
* synchronization storage

These decisions are made only when concrete
technical requirements exist.

---

# Status

This decision defines the fundamental boundary
between Core and Persistence within NC-PoRe.

The concrete technical storage implementation
will be defined through later implementations
and decisions.
