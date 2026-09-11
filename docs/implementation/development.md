# NC-PoRe Development Guide

## Deutsch ([English version below](#english-version))

## Zweck

Dieses Dokument beschreibt grundlegende Entwicklungsregeln, Arbeitsweisen und die Entwicklungsumgebung für NC-PoRe.

Architekturentscheidungen selbst werden nicht hier getroffen, sondern über Architecture Decision Records (ADRs) dokumentiert.

## Entwicklungsprinzipien

NC-PoRe folgt insbesondere diesen Prinzipien:

- Open Source first
- nachvollziehbare Entscheidungen
- kleine, überprüfbare Änderungen
- offene Standards
- saubere Dokumentation
- Qualität vor Geschwindigkeit

## Code-Kommentare

Der Quellcode erklärt **was** passiert. Kommentare erklären **warum** es passiert.

Gute Kommentare dokumentieren insbesondere:

- Architekturgrenzen
- fachliche Regeln
- Lifecycle-Beschränkungen
- bewusst gewählte Einschränkungen
- nicht offensichtliche Designentscheidungen
- Verweise auf relevante ADRs

Kommentare werden zusammen mit dem Code gepflegt und sollen die Implementierung nicht lediglich wiederholen.

## Repository-Struktur

Die Dokumentation muss die tatsächliche Repository-Struktur widerspiegeln. Die wesentlichen Bereiche sind:

```text
nc-pore/
├── README.md
├── LICENSE
├── adr/
├── application/
├── core/
├── recorder/
├── runtime/
├── infrastructure/
├── web/
├── lib/
├── js/
├── css/
├── appinfo/
├── docs/
└── tools/
```

Die genaue Modulstruktur wird durch den Quellcode und die jeweils geltenden ADRs bestimmt.

## Branch-Strategie

Der `main`-Branch enthält den integrierten Projektstand. Größere Änderungen werden bevorzugt in klar abgegrenzten Branches entwickelt und nach Prüfung über Pull Requests integriert.

## Commit-Richtlinien

Commits sollen eine klar erkennbare Aufgabe beschreiben, möglichst klein bleiben und nachvollziehbar sein.

Beispiel:

```text
Add production session lifecycle validation
```

## Dokumentationsregeln

- Architekturentscheidungen werden als ADR dokumentiert.
- Projekt- und Implementierungsdokumentation gehört unter `docs/`.
- Der tatsächliche Quellcode und die ADRs sind maßgeblich für technische Entscheidungen.
- Veraltete Dokumentation soll korrigiert oder entfernt werden, sobald sie dem aktuellen Projektstand widerspricht.

## Coding-Prinzipien

NC-PoRe-Code soll lesbar, modular, testbar und dokumentiert sein.

Der Core enthält fachliche Modelle, Geschäftsregeln, Zustände und Domain-Operationen. Benutzeroberflächen, provider-spezifische Logik und technische Speicherdetails gehören nicht in den Domain-Core.

## Testing-Strategie

Jede fachlich relevante Änderung benötigt passende Tests.

Geeignete Testebenen sind insbesondere:

- Unit Tests
- Integration Tests
- Real-World-/Systemtests

Praktische Tests sollen reale Nutzungssituationen wie lange Aufnahmen, unterschiedliche Hardware und Netzwerkunterbrechungen abdecken, soweit sie für die jeweilige Änderung relevant sind.

## Testbenennung

Tests werden nach fachlicher Bedeutung benannt. Eine Test-ID beschreibt die fachliche Anforderung, nicht die Position im Quellcode.

## Entwicklungsumgebung

Die Referenzentwicklung verwendet bevorzugt freie Werkzeuge. Konkrete Technologieentscheidungen werden durch die entsprechenden ADRs festgelegt.

Grundlegende Werkzeuge sind:

- Git
- Entwicklungseditor
- Build-Werkzeuge
- Testumgebung

## Entwicklungsworkflow

Typischer Ablauf:

```text
Änderung
  ↓
lokale Prüfung und Tests
  ↓
Commit
  ↓
Push / Pull Request
  ↓
Review / Integration
```

Vor Änderungen sind insbesondere Repository-Status und aktueller Branch zu prüfen.

## Issue Management

Aufgaben und Fehler werden nachvollziehbar dokumentiert. Architekturveränderungen benötigen einen ADR.

## Release-Philosophie

NC-PoRe verwendet nachvollziehbare Versionen. Entwicklungsstände und stabile Releases werden klar unterschieden.

## Contribution-Philosophie

Beiträge von außen sind erwünscht. Änderungen sollen nachvollziehbar sein, dokumentiert werden und die Projektprinzipien einhalten.

## Security Development

Sicherheitsrelevante Änderungen werden besonders geprüft. Besondere Aufmerksamkeit gilt unter anderem Zugangsdaten, Audiodaten, Uploads, Berechtigungen und Synchronisation.

## Leitgedanke

NC-PoRe soll verständlich, überprüfbar und langfristig wartbar bleiben.

---

# English Version

## Purpose

This document describes the fundamental development rules, working methods and development environment for NC-PoRe.

Architecture decisions are not made here; they are documented through Architecture Decision Records (ADRs).

## Development Principles

NC-PoRe follows these principles in particular:

- Open Source first
- traceable decisions
- small, verifiable changes
- open standards
- clean documentation
- quality over speed

## Code Comments

The source code explains **what** happens. Comments explain **why** it happens.

Good comments document in particular:

- architectural boundaries
- domain rules
- lifecycle constraints
- intentional limitations
- non-obvious design decisions
- references to relevant ADRs

Comments are maintained together with the code and should not merely repeat the implementation.

## Repository Structure

Documentation must reflect the actual repository structure. The main areas are:

```text
nc-pore/
├── README.md
├── LICENSE
├── adr/
├── application/
├── core/
├── recorder/
├── runtime/
├── infrastructure/
├── web/
├── lib/
├── js/
├── css/
├── appinfo/
├── docs/
└── tools/
```

The exact module structure is defined by the source tree and the applicable ADRs.

## Branch Strategy

The `main` branch contains the integrated project state. Larger changes are preferably developed in clearly scoped branches and integrated through pull requests after validation.

## Commit Guidelines

Commits should describe a clearly identifiable task, remain small where possible, and be traceable.

Example:

```text
Add production session lifecycle validation
```

## Documentation Rules

- Architecture decisions are documented as ADRs.
- Project and implementation documentation belongs under `docs/`.
- The source tree and ADRs are authoritative for technical decisions.
- Outdated documentation should be corrected or removed when it contradicts the current project state.

## Coding Principles

NC-PoRE code should be readable, modular, testable and documented.

The Core contains domain models, business rules, states and domain operations. User interfaces, provider-specific logic and technical storage details do not belong in the domain Core.

## Testing Strategy

Relevant domain changes require corresponding tests.

Suitable test levels include:

- unit tests
- integration tests
- real-world/system tests

Practical tests should cover real usage situations such as long recordings, different hardware and network interruptions where relevant to the change.

## Test Naming

Tests are named according to domain meaning. A test identifier describes the functional requirement, not its location in the source tree.

## Development Environment

The reference development environment preferably uses open-source tools. Concrete technology choices are defined by the applicable ADRs.

Basic tooling includes:

- Git
- development editor
- build tools
- test environment

## Development Workflow

Typical workflow:

```text
Change
  ↓
local validation and tests
  ↓
Commit
  ↓
Push / Pull Request
  ↓
Review / Integration
```

Before changes, check the repository status and current branch.

## Issue Management

Tasks and defects are documented traceably. Architecture changes require an ADR.

## Release Philosophy

NC-PoRe uses traceable versions. Development states and stable releases are clearly distinguished.

## Contribution Philosophy

External contributions are welcome. Changes should be traceable, documented and consistent with the project principles.

## Security Development

Security-related changes receive particular attention. Important areas include credentials, audio data, uploads, permissions and synchronization.

## Guiding Principle

NC-PoRe should remain understandable, verifiable and maintainable over time.
