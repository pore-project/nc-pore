# ADR-048: Artifact Registry and Persistence Coordination Boundary

* Status: Accepted
* Date: 2026-08-02
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

---

# Kontext

NC-PoRe verwaltet lokale Recording Artifacts über mehrere technische Komponenten.

Mit ADR-042 wurde der eigene Lifecycle von Recording Artifacts definiert.

Mit ADR-043 wurde die Persistence Boundary eingeführt.

Mit ADR-044 wurde der Persistence Provider als technische Abstraktion definiert.

Mit ADR-047 wurde eine Local Artifact Registry zur Verwaltung lokaler Artifact-Referenzen eingeführt.

Damit existieren aktuell zwei getrennte technische Bereiche:

```text
Recording Artifact
        │
        ├── Local Artifact Registry
        │
        └── Persistence Provider
```

Die Registry verwaltet Informationen über bekannte lokale Artifacts.

Der Persistence Provider verwaltet gespeicherte Artifact-Daten.

Es entsteht die Frage:

> Welche Komponente koordiniert die Zusammenarbeit zwischen Registry und Persistence?

---

# Entscheidung

NC-PoRe führt keine direkte Abhängigkeit zwischen Local Artifact Registry und Persistence Provider ein.

Die Koordination erfolgt durch eine übergeordnete Workflow-Schicht.

Die technische Struktur wird:

```text
Workflow Coordination

        │
        ├──────────────┐
        ▼              ▼

Local Artifact     Persistence
Registry           Provider

        │              │
        ▼              ▼

Artifact          Stored Artifact
References        Data
```

---

# Verantwortlichkeiten

## Local Artifact Registry

Verantwortlich für:

- lokale Artifact-Referenzen
- Auffindbarkeit
- technische Registry-Metadaten

Nicht verantwortlich für:

- Speicherung von Mediendaten
- Persistence-Operationen
- Workflow-Koordination

---

## Persistence Provider

Verantwortlich für:

- Speichern von Artifacts
- Laden von Artifacts
- Entfernen von Artifacts

Nicht verantwortlich für:

- Registry-Zustände
- Recovery-Entscheidungen
- Workflow-Abläufe

---

## Workflow Coordination

Verantwortlich für:

- Reihenfolge technischer Operationen
- Koordination mehrerer Grenzen
- Sicherstellen konsistenter Abläufe

---

# Konsequenzen

## Positive Konsequenzen

- Registry und Persistence bleiben unabhängig.
- Storage-Technologien bleiben austauschbar.
- Komponenten können separat getestet werden.
- Workflow-Abläufe bleiben explizit nachvollziehbar.

---

## Negative Konsequenzen

- Eine zusätzliche Koordinationsschicht entsteht.
- Workflow-Logik muss zukünftige Fehlerfälle behandeln.
- Die Reihenfolge technischer Operationen muss definiert werden.

---

# Nicht Bestandteil dieser Entscheidung

Diese ADR entscheidet nicht über:

- konkrete Storage-Technologien
- Datenbankmodelle
- Synchronisationsmechanismen
- Recovery-Algorithmen
- Audioaufnahmeprozesse

Diese Entscheidungen werden separat getroffen.

---

# Beziehung zu bestehenden ADRs

Diese Entscheidung erweitert:

- ADR-041 Local Recording Artifact and Storage Boundary
- ADR-042 Recording Artifact Model and Lifecycle Boundary
- ADR-043 Local Recording Persistence Boundary
- ADR-044 Persistence Provider Interface
- ADR-046 Local Artifact Recovery and Consistency Strategy
- ADR-047 Local Artifact Registry and Discovery Strategy

---

# Zusammenfassung

NC-PoRe koordiniert Local Artifact Registry und Persistence Provider nicht über direkte Abhängigkeiten, sondern über eine explizite Workflow Coordination Boundary.

Dadurch bleiben technische Komponenten getrennt, austauschbar und nachvollziehbar.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# Context

NC-PoRe manages local Recording Artifacts through multiple technical components.

ADR-042 defined the independent lifecycle of Recording Artifacts.

ADR-043 introduced the Persistence Boundary.

ADR-044 defined the Persistence Provider as a technical abstraction.

ADR-047 introduced a Local Artifact Registry for managing local artifact references.

The current architecture contains two separate technical areas:

```text
Recording Artifact
        │
        ├── Local Artifact Registry
        │
        └── Persistence Provider
```

The registry manages knowledge about locally known artifacts.

The Persistence Provider manages stored artifact data.

This creates the question:

> Which component coordinates the interaction between Registry and Persistence?

---

# Decision

NC-PoRe does not introduce direct dependencies between Local Artifact Registry and Persistence Provider.

Coordination is performed by a higher-level Workflow Coordination layer.

The technical structure becomes:

```text
Workflow Coordination

        │
        ├──────────────┐
        ▼              ▼

Local Artifact     Persistence
Registry           Provider

        │              │
        ▼              ▼

Artifact          Stored Artifact
References        Data
```

---

# Responsibilities

## Local Artifact Registry

Responsible for:

- local artifact references
- discovery
- technical registry metadata

Not responsible for:

- storing media data
- persistence operations
- workflow coordination

---

## Persistence Provider

Responsible for:

- storing artifacts
- loading artifacts
- removing artifacts

Not responsible for:

- registry state
- recovery decisions
- workflow execution

---

## Workflow Coordination

Responsible for:

- ordering technical operations
- coordinating multiple boundaries
- maintaining consistent workflows

---

# Consequences

## Positive Consequences

- Registry and Persistence remain independent.
- Storage technologies remain replaceable.
- Components can be tested separately.
- Workflow behavior remains explicit.

---

## Negative Consequences

- An additional coordination layer is introduced.
- Workflow logic must handle future failure cases.
- Operation ordering must be defined.

---

# Not Part of This Decision

This ADR does not decide:

- concrete storage technologies
- database models
- synchronization mechanisms
- recovery algorithms
- audio capture processes

These decisions are handled separately.

---

# Relationship to Existing ADRs

This decision extends:

- ADR-041 Local Recording Artifact and Storage Boundary
- ADR-042 Recording Artifact Model and Lifecycle Boundary
- ADR-043 Local Recording Persistence Boundary
- ADR-044 Persistence Provider Interface
- ADR-046 Local Artifact Recovery and Consistency Strategy
- ADR-047 Local Artifact Registry and Discovery Strategy

---

# Summary

NC-PoRe coordinates Local Artifact Registry and Persistence Provider through an explicit Workflow Coordination Boundary instead of direct dependencies.

This keeps technical components separated, replaceable and traceable.
