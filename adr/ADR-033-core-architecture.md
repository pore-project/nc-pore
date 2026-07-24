# ADR-033 Core Architecture

- Status: Proposed
- Date: 2026-07-24

---

# Deutsch ([English version below](#english-version))

## Context

NC-PoRe benötigt eine zentrale fachliche Instanz, die die Regeln und Zustände
des Systems verwaltet.

Die bisherigen Architekturentscheidungen definieren:

- Production Session als zentrale fachliche Einheit
- Trennung von Identity, Participant und Role
- Local Recording First als grundlegendes Prinzip
- Trennung von Control Synchronization und Media Synchronization
- nachvollziehbare Produktionshistorie
- offene und erweiterbare Architektur

Damit diese Prinzipien konsistent umgesetzt werden können, benötigt NC-PoRe
eine klare Definition der Verantwortung des Core.

---

# Decision

NC-PoRe verwendet eine Core-Architektur als zentrale fachliche Instanz.

Der Core ist verantwortlich für:

- fachliche Geschäftslogik
- Verwaltung von Production Sessions
- Durchsetzung der Domain Rules
- Verwaltung fachlicher Zustände
- Prüfung von Berechtigungen
- Verarbeitung fachlicher Ereignisse
- Bereitstellung stabiler Schnittstellen für Clients und externe Komponenten

Der Core bildet die fachliche Wahrheit des Systems ab.

---

# Core Responsibilities

Der Core verwaltet insbesondere:

## Production Sessions

Der Core ist verantwortlich für:

- Erstellung von Sessions
- Verwaltung des Session-Lebenszyklus
- Zuordnung von Teilnehmern
- Verwaltung von Session-bezogenen Zuständen

---

## Identity and Roles

Der Core verwaltet:

- Identitäten innerhalb des Systems
- Teilnehmer einer Produktion
- Rollen innerhalb einer Session
- daraus abgeleitete Berechtigungen

---

## Domain Rules

Der Core stellt sicher, dass fachlich ungültige Zustände nicht entstehen.

Beispiele:

- Ein Recording kann nicht ohne Production Session existieren.
- Rollen gelten nur innerhalb ihres fachlichen Kontextes.
- Abgeschlossene Zustände können nicht unkontrolliert verändert werden.

---

## Activity History

Der Core erzeugt und verwaltet nachvollziehbare fachliche Ereignisse.

Die Activity History beschreibt:

- was passiert ist
- wann es passiert ist
- in welchem fachlichen Kontext es passiert ist

---

# Non-Responsibilities

Der Core ist ausdrücklich nicht verantwortlich für:

- direkte Audioaufnahme
- Benutzeroberflächen
- Hardwarezugriff
- plattformspezifische Funktionen
- langfristige Speicherung einzelner Dateien
- Darstellung und Interaktion mit Anwendern

Diese Aufgaben gehören zu anderen Systemkomponenten.

---

# Architecture Principle

Der Core schützt die fachliche Integrität des Systems.

Technische Komponenten dürfen den Core erweitern oder verwenden.

Sie ersetzen jedoch nicht seine fachliche Verantwortung.

---

# Consequences

## Positive Consequences

- klare Trennung von Fachlogik und technischer Umsetzung
- bessere Erweiterbarkeit
- unabhängige Entwicklung verschiedener Clients
- einfachere Tests fachlicher Regeln
- langfristig stabile Architektur

---

## Negative Consequences

- zusätzlicher initialer Entwicklungsaufwand
- mehr Abstimmung zwischen Komponenten notwendig
- Architektur erfordert klare Schnittstellen

---

# Alternatives Considered

## Fachlogik direkt in Clients

Nicht gewählt.

Begründung:

Dies würde zu unterschiedlichen Interpretationen der Regeln führen
und die Konsistenz des Systems gefährden.

---

## Fachlogik vollständig in Speicher- oder Infrastrukturkomponenten

Nicht gewählt.

Begründung:

Speicherung und technische Infrastruktur sollen nicht die fachliche Bedeutung
des Systems definieren.

---

# Status

Diese Entscheidung definiert die grundlegende Rolle des Core in NC-PoRe.

Technische Details der Core-Implementierung werden in späteren Entscheidungen
behandelt.

---

# English Version ([Deutsche Version oben](#deutsch))

## Context

NC-PoRe requires a central domain authority responsible for managing
the rules and states of the system.

Previous architecture decisions define:

- Production Session as the central domain unit
- separation of Identity, Participant and Role
- Local Recording First as a fundamental principle
- separation of Control Synchronization and Media Synchronization
- traceable production history
- open and extensible architecture

To implement these principles consistently, NC-PoRe requires a clear
definition of Core responsibilities.

---

# Decision

NC-PoRe uses a Core architecture as the central domain authority.

The Core is responsible for:

- domain business logic
- management of Production Sessions
- enforcement of Domain Rules
- management of domain states
- permission validation
- processing of domain events
- providing stable interfaces for clients and external components

The Core represents the domain truth of the system.

---

# Core Responsibilities

The Core manages:

## Production Sessions

The Core is responsible for:

- creating sessions
- managing session lifecycle
- assigning participants
- managing session-related states

---

## Identity and Roles

The Core manages:

- identities within the system
- production participants
- roles within sessions
- derived permissions

---

## Domain Rules

The Core ensures that invalid domain states cannot occur.

Examples:

- A Recording cannot exist without a Production Session.
- Roles only exist within their domain context.
- Completed states cannot be changed without control.

---

## Activity History

The Core creates and manages traceable domain events.

Activity History describes:

- what happened
- when it happened
- within which domain context it happened

---

# Non-Responsibilities

The Core is explicitly not responsible for:

- direct audio recording
- user interfaces
- hardware access
- platform-specific functions
- long-term storage of individual files
- user presentation and interaction

These responsibilities belong to other system components.

---

# Architecture Principle

The Core protects the domain integrity of the system.

Technical components may extend or use the Core.

They do not replace its domain responsibility.

---

# Consequences

## Positive Consequences

- clear separation of domain logic and technical implementation
- improved extensibility
- independent development of different clients
- easier testing of domain rules
- long-term stable architecture

---

## Negative Consequences

- additional initial development effort
- more coordination between components required
- architecture requires clear interfaces

---

# Alternatives Considered

## Domain logic directly in clients

Rejected.

Reason:

This would lead to different interpretations of rules and threaten system
consistency.

---

## Domain logic fully inside storage or infrastructure components

Rejected.

Reason:

Storage and technical infrastructure must not define the domain meaning
of the system.

---

# Status

This decision defines the fundamental role of the Core in NC-PoRe.

Technical details of the Core implementation will be addressed in later
decisions.
