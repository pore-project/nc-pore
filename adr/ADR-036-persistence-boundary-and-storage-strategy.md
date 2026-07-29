# ADR-036 Development Workflow and Source of Truth

* Status: Proposed
* Date: 2026-07-29
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

---

# Kontext

NC-PoRe befindet sich am Übergang von der Architekturphase
zur technischen Umsetzung.

Während der bisherigen Entwicklung wurde deutlich, dass
nachvollziehbare Arbeitsabläufe genauso wichtig sind wie
technische Entscheidungen.

Die Architektur definiert:

* fachliche Verantwortlichkeiten
* Systemgrenzen
* technische Prinzipien

Die tägliche Entwicklung benötigt zusätzlich einen
klar definierten Umgang mit:

* aktuellem Projektzustand
* vorhandenen Dateien
* Änderungen
* Tests
* Entscheidungen

Ohne eine eindeutige Grundlage können Annahmen entstehen,
die nicht dem tatsächlichen Projektstand entsprechen.

---

# Entscheidung

NC-PoRe verwendet einen zustandsorientierten
Entwicklungsworkflow.

Vor technischen Änderungen wird zunächst der aktuelle
Projektzustand festgestellt.

Der aktuelle Zustand der vorhandenen Dateien und Dokumente
ist die Grundlage für weitere Entscheidungen.

Änderungen werden nicht auf Basis früherer Annahmen,
sondern auf Basis des tatsächlich vorhandenen Projektstands
durchgeführt.

---

# Source of Truth

Die jeweils aktuelle Projektstruktur und der aktuelle
Repository-Inhalt bilden die technische Wahrheit.

Dazu gehören insbesondere:

* vorhandene Dateien
* aktuelle Dokumentation
* aktueller Git-Status
* aktueller Branch
* vorhandener Quellcode

Frühere Diskussionen oder Annahmen ersetzen nicht den
aktuellen Projektstand.

---

# Development Process

Der Entwicklungsablauf folgt grundsätzlich diesem Muster:

```text
Aktuellen Zustand prüfen

↓

Bestehende Struktur verstehen

↓

Kleinste notwendige Änderung definieren

↓

Änderung durchführen

↓

Tests oder Prüfungen ausführen

↓

Änderung dokumentieren

↓

Commit erstellen
```

---

# Verification Before Change

Vor Änderungen werden relevante Informationen geprüft.

Beispiele:

```bash
git status

git branch --show-current

Dateistruktur prüfen

relevante Dateien lesen
```

Die Prüfung soll verhindern, dass Änderungen auf falschen
Annahmen basieren.

---

# Small Controlled Changes

Änderungen sollen:

* möglichst klein bleiben
* eine klare fachliche oder technische Absicht besitzen
* nachvollziehbar überprüfbar sein

Große nicht überprüfbare Änderungen sollen vermieden werden.

---

# Documentation Synchronization

Dokumentation und Implementierung müssen konsistent bleiben.

Wenn eine technische Änderung Architekturprinzipien,
Modulgrenzen oder langfristige Entscheidungen betrifft,
wird geprüft, ob ein ADR erforderlich ist.

---

# Consequences

## Positive Consequences

* weniger Fehlannahmen während der Entwicklung
* nachvollziehbare Änderungen
* bessere Zusammenarbeit
* geringeres Risiko durch veraltete Informationen
* reproduzierbare Entwicklungsabläufe

---

## Negative Consequences

* zusätzlicher Prüfaufwand vor Änderungen
* weniger spontane Änderungen ohne Analyse

Diese Nachteile werden bewusst akzeptiert.

Die zusätzliche Prüfung reduziert langfristig Fehler und
vermeidbare Rückarbeiten.

---

# Alternatives Considered

## Development Based on Previous Context

Nicht gewählt.

Begründung:

Frühere Diskussionen können unvollständig oder veraltet sein.
Der tatsächliche Projektzustand muss entscheidend bleiben.

---

## Large Batch Changes

Nicht gewählt.

Begründung:

Große Änderungspakete erschweren Fehleranalyse,
Review und Rückverfolgbarkeit.

---

# Relationship to Existing Architecture

Diese Entscheidung erweitert:

* ADR-034 Implementation Architecture
* ADR-035 Domain Lifecycle and State Transition Management

Sie definiert den praktischen Entwicklungsprozess,
der die technische Umsetzung der Architektur unterstützt.

---

# Status

Diese Entscheidung definiert den grundlegenden Workflow
für die weitere technische Entwicklung von NC-PoRe.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# Context

NC-PoRe is transitioning from the architecture phase
to technical implementation.

During previous development it became clear that
traceable workflows are as important as technical decisions.

The architecture defines:

* domain responsibilities
* system boundaries
* technical principles

Daily development additionally requires a clear handling of:

* current project state
* existing files
* changes
* tests
* decisions

Without a defined foundation, assumptions may diverge from
the actual project state.

---

# Decision

NC-PoRe uses a state-oriented development workflow.

Before technical changes, the current project state is
determined.

The actual content of existing files and documentation is
the basis for further decisions.

Changes are not performed based on previous assumptions,
but based on the actual current project state.

---

# Source of Truth

The current repository content and project structure define
the technical source of truth.

This includes:

* existing files
* current documentation
* current Git status
* current branch
* existing source code

Previous discussions or assumptions do not replace the
current project state.

---

# Development Process

The development workflow follows this pattern:

```text
Inspect current state

↓

Understand existing structure

↓

Define smallest required change

↓

Apply change

↓

Run tests or checks

↓

Document change

↓

Create commit
```

---

# Verification Before Change

Relevant information is checked before changes.

Examples:

```bash
git status

git branch --show-current

inspect file structure

read relevant files
```

This prevents changes based on incorrect assumptions.

---

# Small Controlled Changes

Changes should:

* remain as small as possible
* have a clear technical or domain purpose
* be verifiable

Large unverified changes should be avoided.

---

# Documentation Synchronization

Documentation and implementation must remain consistent.

When a technical change affects architecture principles,
module boundaries or long-term decisions, an ADR is created
when required.

---

# Consequences

## Positive Consequences

* fewer incorrect assumptions
* traceable changes
* improved collaboration
* reduced risk from outdated information
* reproducible development workflow

---

## Negative Consequences

* additional verification effort before changes
* fewer spontaneous unreviewed modifications

These disadvantages are consciously accepted.

---

# Alternatives Considered

## Development Based on Previous Context

Rejected.

Reason:

Previous discussions may be incomplete or outdated.
The actual project state must remain authoritative.

---

## Large Batch Changes

Rejected.

Reason:

Large change sets make review, debugging and traceability
more difficult.

---

# Relationship to Existing Architecture

This decision extends:

* ADR-034 Implementation Architecture
* ADR-035 Domain Lifecycle and State Transition Management

It defines the practical development workflow supporting
the technical implementation of the architecture.

---

# Status

This decision defines the fundamental workflow for the
continued technical development of NC-PoRe.
