# ADR-046 Local Artifact Recovery Strategy

* Status: Proposed
* Date: 2026-08-01
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

---

# Kontext

Mit ADR-041 wurde das Recording Artifact als eigenständige technische Einheit eingeführt.

Mit ADR-042 wurde der eigene Lifecycle von Recording Artifacts definiert.

Mit ADR-043 wurde die Persistence Boundary eingeführt.

Mit ADR-044 wurde das Persistence Provider Interface als technische Schnittstelle innerhalb dieser Boundary definiert.

Mit ADR-045 wurde die lokale Verwaltung von Recording Artifacts als technische Grundlage implementiert.

Die aktuelle Architektur ermöglicht damit:

```text
Recording Artifact

↓

Local Artifact Management

↓

Persistence Provider Interface

↓

Persistence Provider Implementation
```

Es besteht jedoch noch keine definierte Strategie für den Fall, dass ein lokales Recording Artifact nicht vollständig verfügbar oder nicht konsistent ist.

Mögliche Situationen:

* ein lokales Artifact wurde erzeugt, aber nicht vollständig gespeichert
* ein Persistenzvorgang wurde unterbrochen
* ein Client wurde während eines technischen Vorgangs beendet
* ein Artifact befindet sich in einem unvollständigen Zustand
* gespeicherte Metadaten stimmen nicht mit dem erwarteten Lifecycle-Zustand überein

NC-PoRe benötigt daher eine technische Strategie zur Behandlung solcher Zustände.

---

# Entscheidung

NC-PoRe führt eine **Local Artifact Recovery Strategy** ein.

Die Recovery-Verantwortung liegt innerhalb der technischen Artifact- und Persistence-Schichten.

Die Architektur lautet:

```text
Recording Artifact

↓

Artifact Lifecycle Validation

↓

Recovery Detection

↓

Recovery Handling

↓

Persistence Provider
```

Die Recovery-Logik gehört nicht zur fachlichen Production Session Logik.

---

# Architectural Principle

Lokale Wiederherstellung ist eine technische Konsistenzaufgabe.

Sie entscheidet nicht:

* ob eine Produktion fachlich gültig ist
* ob eine Aufnahme stattfinden darf
* welche Rolle ein Teilnehmer besitzt

Sie entscheidet ausschließlich:

* ob technische Artefakte konsistent verwaltet werden können
* ob unvollständige Zustände erkannt werden
* ob eine Wiederherstellung möglich ist

---

# Responsibilities

## Local Artifact Management

Verantwortlich für:

* Verwaltung lokaler Recording Artifacts
* Erkennung technischer Zustandsabweichungen
* Übergabe von Recovery-Anforderungen

Nicht verantwortlich für:

* fachliche Produktionsentscheidungen
* Benutzerentscheidungen
* Synchronisationsstrategie

---

## Recovery Handling

Verantwortlich für:

* Erkennen unvollständiger Zustände
* Wiederherstellung technisch möglicher Zustände
* Markierung nicht wiederherstellbarer Artifacts

Recovery erstellt keine neuen fachlichen Zustände.

---

## Persistence Provider

Der Persistence Provider bleibt ausschließlich verantwortlich für:

* Speichern
* Laden
* Auflisten
* Entfernen

Er enthält keine Recovery-Entscheidungen.

---

# Initial Recovery Scope

Die erste Version behandelt folgende Fälle:

```text
Created Artifact

↓

Stored Artifact

↓

Missing or incomplete state

↓

Recovery evaluation
```

Unterstützt werden zunächst:

* Erkennen fehlender Persistenzdaten
* erneutes Laden gespeicherter Artifacts
* Validierung technischer Artifact-Zustände

Automatische Reparatur komplexer Fälle ist nicht Bestandteil dieser Entscheidung.

---

# Recovery States

Die Recovery-Strategie verwendet zunächst technische Zustände:

```text
Valid

↓

Recoverable

↓

Invalid
```

## Valid

Das Artifact befindet sich in einem erwarteten technischen Zustand.

## Recoverable

Das Artifact kann durch technische Maßnahmen wieder in einen konsistenten Zustand gebracht werden.

## Invalid

Das Artifact kann nicht automatisch wiederhergestellt werden.

---

# Technology Independence

Diese Entscheidung definiert keine konkrete Recovery-Technologie.

Nicht Bestandteil dieser ADR:

* Datenbank-Recovery
* Dateisystem-Reparatur
* Backup-Systeme
* Cloud Recovery
* Verschlüsselungswiederherstellung

Diese Entscheidungen werden später getroffen.

---

# Consequences

## Positive Consequences

* technische Zustände werden explizit behandelt
* lokale Artefakte können sicherer verwaltet werden
* Persistence Boundary bleibt klar getrennt
* zukünftige Storage-Lösungen werden nicht eingeschränkt
* Fehlerfälle werden Teil der Architektur

---

## Negative Consequences

* zusätzliche technische Zustandslogik
* weiterer Modellierungsaufwand
* Recovery benötigt spätere konkrete Implementierungen

Diese Nachteile werden bewusst akzeptiert.

---

# Considered Alternatives

## Recovery im Persistence Provider

Nicht gewählt.

Begründung:

Der Persistence Provider kennt nur Speicherung.

Recovery benötigt Wissen über Artifact-Lifecycle und technische Zustände.

---

## Recovery im Recorder Workflow

Nicht gewählt.

Begründung:

Der Workflow würde dadurch technische Storage-Verantwortung übernehmen.

Dies würde die Trennung zwischen Workflow und Infrastruktur verletzen.

---

# Relationship to Existing Architecture

Diese Entscheidung erweitert:

* ADR-041 Local Recording Artifact and Storage Boundary
* ADR-042 Recording Artifact Model and Lifecycle Boundary
* ADR-043 Local Recording Persistence Boundary
* ADR-044 Persistence Provider Interface
* ADR-045 Local Artifact Management Foundation

Sie definiert die technische Grundlage für den Umgang mit lokalen inkonsistenten Artifact-Zuständen.

---

# Future Decisions

Spätere Entscheidungen behandeln:

* konkrete Recovery-Mechanismen
* Backup-Strategien
* Wiederaufnahme unterbrochener Aufnahmen
* Synchronisations-Recovery
* automatische Reparaturverfahren

---

# Status

Diese Entscheidung definiert die Architekturgrundlage für technische Wiederherstellung lokaler Recording Artifacts.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# Context

ADR-041 introduced the Recording Artifact as an independent technical entity.

ADR-042 defined the dedicated lifecycle of Recording Artifacts.

ADR-043 introduced the Persistence Boundary.

ADR-044 defined the Persistence Provider Interface as the technical contract inside this boundary.

ADR-045 implemented the foundation for local Recording Artifact management.

The current architecture enables:

```text
Recording Artifact

↓

Local Artifact Management

↓

Persistence Provider Interface

↓

Persistence Provider Implementation
```

However, no strategy currently exists for situations where a local Recording Artifact is incomplete or inconsistent.

Possible situations include:

* a local artifact was created but not fully stored
* a persistence operation was interrupted
* a client terminated during a technical operation
* an artifact remains in an incomplete state
* stored metadata does not match the expected lifecycle state

NC-PoRe therefore requires a technical strategy for handling such states.

---

# Decision

NC-PoRe introduces a **Local Artifact Recovery Strategy**.

Recovery responsibility belongs to the technical Artifact and Persistence layers.

The architecture becomes:

```text
Recording Artifact

↓

Artifact Lifecycle Validation

↓

Recovery Detection

↓

Recovery Handling

↓

Persistence Provider
```

Recovery logic is not part of Production Session domain logic.

---

# Architectural Principle

Local recovery is a technical consistency responsibility.

It does not decide:

* whether a production is valid
* whether recording is allowed
* which role a participant has

It only decides:

* whether technical artifacts can be managed consistently
* whether incomplete states can be detected
* whether recovery is technically possible

---

# Responsibilities

## Local Artifact Management

Responsible for:

* managing local Recording Artifacts
* detecting technical state inconsistencies
* forwarding recovery requirements

Not responsible for:

* production decisions
* user decisions
* synchronization strategy

---

## Recovery Handling

Responsible for:

* detecting incomplete states
* restoring technically possible states
* marking non-recoverable artifacts

Recovery does not create new domain states.

---

## Persistence Provider

The Persistence Provider remains responsible only for:

* storing
* loading
* listing
* removing

It contains no recovery decisions.

---

# Initial Recovery Scope

The first version handles:

```text
Created Artifact

↓

Stored Artifact

↓

Missing or incomplete state

↓

Recovery evaluation
```

Initially supported:

* detecting missing persistence data
* reloading stored artifacts
* validating technical artifact states

Automatic repair of complex cases is not part of this decision.

---

# Recovery States

The recovery strategy initially uses technical states:

```text
Valid

↓

Recoverable

↓

Invalid
```

## Valid

The artifact is in an expected technical state.

## Recoverable

The artifact can be returned to a consistent state through technical actions.

## Invalid

The artifact cannot be automatically recovered.

---

# Technology Independence

This decision does not define a concrete recovery technology.

Not part of this ADR:

* database recovery
* filesystem repair
* backup systems
* cloud recovery
* encryption recovery

These decisions will be addressed later.

---

# Consequences

## Positive Consequences

* technical states are explicitly handled
* local artifacts can be managed more safely
* Persistence Boundary remains clearly separated
* future storage solutions remain unrestricted
* failure scenarios become part of the architecture

---

## Negative Consequences

* additional technical state logic
* additional modeling effort
* recovery requires later concrete implementations

These disadvantages are consciously accepted.

---

# Considered Alternatives

## Recovery inside Persistence Provider

Rejected.

Reason:

The Persistence Provider only knows storage.

Recovery requires knowledge about artifact lifecycle and technical states.

---

## Recovery inside Recorder Workflow

Rejected.

Reason:

The workflow would take responsibility for technical storage handling.

This would violate separation between workflow and infrastructure.

---

# Relationship to Existing Architecture

This decision extends:

* ADR-041 Local Recording Artifact and Storage Boundary
* ADR-042 Recording Artifact Model and Lifecycle Boundary
* ADR-043 Local Recording Persistence Boundary
* ADR-044 Persistence Provider Interface
* ADR-045 Local Artifact Management Foundation

It defines the technical foundation for handling inconsistent local Recording Artifact states.

---

# Future Decisions

Future decisions will address:

* concrete recovery mechanisms
* backup strategies
* interrupted recording continuation
* synchronization recovery
* automated repair procedures

---

# Status

This decision defines the architectural foundation for technical recovery of local Recording Artifacts.
