# NC-PoRe Development Guide

## Version

0.3

## Date

2026-07-29

---

# Deutsch

---

# Zweck

Dieses Dokument beschreibt die grundlegenden Entwicklungsregeln,
Arbeitsweisen und die Entwicklungsumgebung für NC-PoRe.

Ziel ist eine nachvollziehbare, wartbare und gemeinschaftsfähige
Entwicklung.

Dieses Dokument definiert den praktischen Entwicklungsprozess.

Architekturentscheidungen selbst werden nicht hier getroffen,
sondern über Architecture Decision Records (ADRs) dokumentiert.

---

# Entwicklungsprinzipien

NC-PoRe folgt diesen Grundprinzipien:

* Open Source first
* nachvollziehbare Entscheidungen
* kleine, überprüfbare Änderungen
* offene Standards
* saubere Dokumentation
* Qualität vor Geschwindigkeit

---

## Code-Kommentare

Code-Kommentare sind Teil der technischen Projektdokumentation.

Der Quellcode erklärt **was** passiert.

Kommentare erklären **warum** es passiert.

Gute Kommentare dokumentieren:

- Architekturgrenzen
- fachliche Regeln
- Lifecycle-Beschränkungen
- bewusst gewählte Einschränkungen
- nicht offensichtliche Designentscheidungen
- Verweise auf relevante ADRs

Vermeide Kommentare, die lediglich die Implementierung wiederholen.
Kommentare müssen zusammen mit dem Code gepflegt werden.
Ein veralteter Kommentar ist schädlicher als ein fehlender Kommentar,
da er eine falsche technische Erklärung liefert.

Bevorzugt:

```rust
// Eine Production Session benötigt vor dem Abschluss
// einen Owner.
//
// Siehe ADR-031.
```

---

# Repository Struktur

Die grundlegende Struktur:

```text
nc-pore/

├── README.md
├── LICENSE
│
├── docs/
│   ├── vision.md
│   ├── requirements.md
│   ├── architecture.md
│   ├── project-status.md
│   │
│   └── implementation/
│       ├── mvp.md
│       └── development.md
│
├── adr/
│   └── ADR-xxx-description.md
│
├── core/
│   └── NC-PoRe Core Source
│
├── recorder/
│   └── Recorder Client Source
│
├── nextcloud-app/
│   └── Nextcloud Application Source
│
└── tests/
    └── Test Resources
```

Die Struktur kann durch spätere Architekturentscheidungen erweitert werden.

Änderungen an grundlegenden Strukturen sollen nachvollziehbar dokumentiert werden.

---

# Branch Strategie

NC-PoRe verwendet eine einfache und nachvollziehbare Git-Strategie.

## Main Branch

```text
main
```

Der Main Branch enthält den aktuellen integrierten Entwicklungsstand.

Änderungen werden nur nach erfolgreicher lokaler Prüfung integriert.

---

## Feature Branches

Größere Änderungen können über eigene Branches entwickelt werden.

Beispiele:

```text
feature/activity-history
feature/session-management
feature/audio-recorder
```

Kleinere, klar abgegrenzte Änderungen können direkt auf dem Entwicklungsstand erfolgen.

---

# Commit Richtlinien

Commits sollen:

* eine klar erkennbare Aufgabe beschreiben
* möglichst klein bleiben
* nachvollziehbar sein

Gut:

```text
Add production session lifecycle validation
```

```text
Implement activity history for session lifecycle
```

Schlecht:

```text
changes
```

```text
updates
```

Ein Commit sollte möglichst eine fachliche oder technische Änderung darstellen.

---

# Dokumentationsregeln

Architekturentscheidungen werden als ADR dokumentiert.

Grundlegende Projektdokumentation gehört nach:

```text
docs/
```

Implementierungsbezogene Dokumentation gehört nach:

```text
docs/implementation/
```

Code-Kommentare erklären:

* warum etwas so gelöst wurde
* welche technische oder fachliche Einschränkung besteht
* welcher Architekturbezug relevant ist

Code-Kommentare sollen nicht lediglich den Code wiederholen.

---

# Coding Prinzipien

NC-PoRe Code soll:

* lesbar
* modular
* testbar
* dokumentiert

sein.

Komplexität soll nur entstehen, wenn sie einen echten Nutzen bringt.

Der Core folgt insbesondere den Prinzipien aus:

* ADR-027 Core Architecture and Module Boundaries
* ADR-035 Domain Lifecycle and State Transition Management

Der Core enthält:

* fachliche Modelle
* Geschäftsregeln
* Zustände
* Domain-Operationen

Der Core enthält nicht:

* Benutzeroberflächen
* Provider-spezifische Logik
* Speicherdetails

---

# Testing Strategie

Tests sind Bestandteil der Entwicklung.

Jede fachlich relevante Änderung benötigt passende Tests.

---

## Unit Tests

Unit Tests prüfen einzelne fachliche Einheiten.

Beispiele:

* Session Lifecycle
* Rollenprüfung
* Identitätsprüfung
* Datenvalidierung

---

## Integration Tests

Integration Tests prüfen das Zusammenspiel mehrerer Komponenten.

Beispiele:

* Core und Storage
* Recorder und Sessionverwaltung
* Synchronisation
* Export

---

## Real World Tests

Praktische Tests prüfen reale Anwendungssituationen.

Beispiele:

* lange Aufnahmen
* unterschiedliche Hardware
* Netzwerkunterbrechungen
* große Produktionsdaten

---

# Test Benennung

Tests werden fachlich nummeriert.

Beispiel:

```text
TEST-01
```

Die Nummer beschreibt die fachliche Testanforderung,
nicht die Position im Quellcode.

Tests können intern verschoben werden,
ohne ihre fachliche Bedeutung zu verlieren.

---

# Entwicklungsumgebung

Die Referenzentwicklung erfolgt bevorzugt mit freien Werkzeugen.

Aktuelle Referenzumgebung:

```text
Linux Mint
```

Entwicklung erfolgt unter einem separaten Entwicklerkonto:

```text
developer
```

Grundanforderungen:

* Git
* Entwicklungseditor
* Build-Werkzeuge
* Testumgebung

Konkrete Technologien werden über separate Architekturentscheidungen festgelegt.

---

# Developer Setup

## Repository Zugriff

NC-PoRe verwendet Git zur Versionsverwaltung.

Repository:

```text
git@github.com:pore-project/nc-pore.git
```

Der private SSH-Schlüssel verbleibt ausschließlich auf dem jeweiligen Entwicklungsrechner.

Nur der öffentliche Schlüssel wird beim Repository-Anbieter hinterlegt.

---

## Lokales Entwicklungsverzeichnis

Beispiel:

```text
/home/developer/projects/nc-pore
```

---

## Git Konfiguration

Die lokale Git-Konfiguration verwendet eine Projektidentität.

Beispiel:

```text
PoRe Project
```

Private Entwicklerdaten gehören nicht in öffentliche Projektdokumentation.

---

# Entwicklungsworkflow

Vor Änderungen:

```bash
git status
```

Aktueller Branch:

```bash
git branch --show-current
```

Typischer Ablauf:

```text
Änderung

↓

lokaler Test

↓

Commit

↓

Push

↓

Review / Integration
```

---

# Issue Management

Aufgaben und Fehler werden nachvollziehbar dokumentiert.

Größere Änderungen sollten eine Begründung besitzen.

Architekturveränderungen benötigen einen ADR.

---

# Release Philosophie

NC-PoRe verwendet nachvollziehbare Versionen.

Beispiel:

```text
0.x.x
```

Entwicklungsphase.

```text
1.0.0
```

Erste stabile produktive Version.

---

# Contribution Philosophie

Beiträge von außen sind erwünscht.

Voraussetzungen:

* nachvollziehbarer Code
* dokumentierte Änderungen
* Einhaltung der Projektprinzipien

---

# Security Development

Sicherheitsrelevante Änderungen werden besonders behandelt.

Besondere Aufmerksamkeit:

* Zugangsdaten
* Audiodaten
* Uploads
* Berechtigungen
* Synchronisation

---

# Leitgedanke

NC-PoRe soll nicht nur funktionieren.

NC-PoRe soll verständlich, überprüfbar und langfristig
weiterentwickelbar sein.

---

# English Version

---

# Purpose

This document describes the fundamental development rules,
working methods and development environment for NC-PoRe.

The goal is a traceable, maintainable and collaborative
development process.

This document defines the practical development process.

Architecture decisions are documented separately through
Architecture Decision Records (ADRs).

---

# Development Principles

NC-PoRe follows these principles:

* Open Source first
* traceable decisions
* small verifiable changes
* open standards
* clean documentation
* quality over speed

---

## Code Comments

Code comments are part of the project's technical documentation.

The source code explains **what** happens.

Comments explain **why** it happens.

Good comments document:

- architectural boundaries
- domain rules
- lifecycle constraints
- intentional limitations
- non-obvious design decisions
- references to relevant ADRs

Avoid comments that simply repeat the implementation.
Comments must be maintained together with the code.
An outdated comment is worse than no comment,
because it provides incorrect technical information.

Prefer:

```rust
// A production session requires an owner
// before completion.
//
// See ADR-031.
```

Instead of:

```rust
// Complete the session.
session.complete();
```

---

# Repository Structure

The basic structure:

```text
nc-pore/

├── README.md
├── LICENSE
│
├── docs/
│   ├── vision.md
│   ├── requirements.md
│   ├── architecture.md
│   ├── project-status.md
│   │
│   └── implementation/
│       ├── mvp.md
│       └── development.md
│
├── adr/
│   └── ADR-xxx-description.md
│
├── core/
│   └── NC-PoRe Core Source
│
├── recorder/
│   └── Recorder Client Source
│
├── nextcloud-app/
│   └── Nextcloud Application Source
│
└── tests/
    └── Test Resources
```

The structure may be extended through later architecture decisions.

---

# Branch Strategy

NC-PoRe uses a simple and traceable Git strategy.

## Main Branch

```text
main
```

The main branch contains the current integrated development state.

Changes are integrated only after successful local validation.

---

## Feature Branches

Larger changes may be developed using dedicated branches.

Examples:

```text
feature/activity-history
feature/session-management
feature/audio-recorder
```

---

# Commit Guidelines

Commits should:

* describe a clearly identifiable task
* remain small where possible
* be traceable

Good:

```text
Add production session lifecycle validation
```

```text
Implement activity history for session lifecycle
```

Bad:

```text
changes
```

```text
updates
```

---

# Documentation Rules

Architecture decisions are documented as ADRs.

Project documentation belongs in:

```text
docs/
```

Implementation documentation belongs in:

```text
docs/implementation/
```

Code comments explain:

* why something exists
* technical or domain constraints
* architectural context

Comments should not simply repeat what the code does.

---

# Coding Principles

NC-PoRe code should be:

* readable
* modular
* testable
* documented

The Core follows especially:

* ADR-027 Core Architecture and Module Boundaries
* ADR-035 Domain Lifecycle and State Transition Management

The Core contains:

* domain models
* business rules
* states
* domain operations

The Core does not contain:

* user interfaces
* provider-specific logic
* storage details

---

# Testing Strategy

Tests are part of development.

Relevant domain changes require corresponding tests.

---

## Unit Tests

Unit tests verify individual domain units.

Examples:

* Session lifecycle
* role validation
* identity validation
* data validation

---

## Integration Tests

Integration tests verify cooperation between components.

Examples:

* Core and storage
* Recorder and session management
* synchronization
* export

---

## Real World Tests

Real world tests verify practical usage scenarios.

Examples:

* long recordings
* different hardware
* network interruptions
* large production data

---

# Test Naming

Tests are numbered by domain requirement.

Example:

```text
TEST-01
```

The number describes the domain test requirement,
not the location inside the source code.

Tests may be moved internally without changing their meaning.

---

# Development Environment

The reference development environment prefers open source tools.

Current reference environment:

```text
Linux Mint
```

Development account:

```text
developer
```

Requirements:

* Git
* development editor
* build tools
* test environment

Specific technologies are defined through separate architecture decisions.

---

# Developer Setup

## Repository Access

NC-PoRe uses Git for version control.

Repository:

```text
git@github.com:pore-project/nc-pore.git
```

The private SSH key remains exclusively on the respective development machine.

Only the public key is registered with the repository provider.

---

## Local Development Directory

Example:

```text
/home/developer/projects/nc-pore
```

---

## Git Configuration

The local Git configuration uses a project identity.

Example:

```text
PoRe Project
```

Private developer data does not belong in public project documentation.

---

# Development Workflow

Before changes:

```bash
git status
```

Current branch:

```bash
git branch --show-current
```

Workflow:

```text
Change

↓

Local Test

↓

Commit

↓

Push

↓

Review / Integration
```

---

# Issue Management

Tasks and defects are documented traceably.

Larger changes should have a clear reason.

Architecture changes require an ADR.

---

# Release Philosophy

NC-PoRe uses traceable versions.

Example:

```text
0.x.x
```

Development phase.

```text
1.0.0
```

First stable productive version.

---

# Contribution Philosophy

External contributions are welcome.

Requirements:

* traceable code
* documented changes
* adherence to project principles

---

# Security Development

Security-related changes receive special attention.

Important areas:

* credentials
* audio data
* uploads
* permissions
* synchronization

---

# Final Principle

NC-PoRe should not only work.

NC-PoRe should remain understandable, verifiable and
maintainable over many years.
