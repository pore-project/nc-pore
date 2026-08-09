# ADR-044 Persistence Provider Interface

* Status: Accepted
* Date: 2026-08-01
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

---

# Kontext

Mit ADR-042 wurde das Recording Artifact als eigenständiges technisches Modell mit eigenem Lifecycle eingeführt.

Mit ADR-043 wurde die Local Recording Persistence Boundary definiert.

Damit existiert eine technische Trennung zwischen:

```text
Recording Artifact

↓

Persistence Boundary

↓

Local Storage
```

ADR-043 definiert jedoch bewusst nur die architektonische Grenze.

Es fehlt noch die Entscheidung, wie Komponenten innerhalb dieser Grenze miteinander kommunizieren.

Die Persistenzschicht benötigt eine technische Schnittstelle, die:

- unabhängig von konkreten Speichertechnologien bleibt
- testbare Implementierungen ermöglicht
- zukünftige Storage-Varianten unterstützt
- keine Abhängigkeit zur Domänenlogik erzeugt

Die Persistence Boundary benötigt daher einen definierten Provider-Vertrag.

---

# Entscheidung

NC-PoRe führt ein **Persistence Provider Interface** ein.

Die lokale Speicherung von Recording Artifacts erfolgt ausschließlich über Implementierungen dieses Interfaces.

Die Architektur lautet:

```text
Recording Artifact

↓

Persistence Provider Interface

↓

Persistence Provider Implementation

↓

Local Storage Backend
```

Der Recorder Workflow und das Recording Artifact Modell kennen keine konkrete Storage-Implementierung.

---

# Architectural Principle

Das Persistence Provider Interface trennt:

- die Anforderung, Daten dauerhaft zu erhalten
- die technische Verwaltung persistierter Daten
- die konkrete Speicherung

Ein Persistence Provider definiert:

**Welche Persistenzoperationen verfügbar sind.**

Eine konkrete Implementierung definiert:

**Wie diese Operationen technisch umgesetzt werden.**

---

# Responsibilities

## Persistence Provider Interface

Das Interface ist verantwortlich für:

- Definition der Persistenzoperationen
- technische Abstraktion der Speicherung
- Austauschbarkeit verschiedener Provider
- klare technische Grenze zwischen Recorder und Storage

Das Interface ist nicht verantwortlich für:

- fachliche Produktionsregeln
- Recording Lifecycle Entscheidungen
- Synchronisationslogik
- Benutzerinteraktion

---

## Persistence Provider Implementation

Eine konkrete Implementierung ist verantwortlich für:

- Speicherung von Recording Artifacts
- Laden gespeicherter Artifacts
- Verwaltung lokaler Persistenzdaten
- Umsetzung der technischen Storage-Details

Eine Implementierung ist nicht verantwortlich für:

- Erzeugung von Recording Artifacts
- fachliche Zustände
- Workflow-Steuerung

---

## Recorder Workflow

Der Recorder Workflow verwendet ausschließlich das Persistence Provider Interface.

Er ist verantwortlich für:

- Übergabe von Recording Artifacts
- Anforderung von Persistenzoperationen
- Koordination des technischen Ablaufs

Der Recorder Workflow kennt nicht:

- Dateisysteme
- Datenbanken
- konkrete Speicherformate

---

# Initial Interface Scope

Das Persistence Provider Interface definiert zunächst die grundlegenden Operationen:

```text
store artifact

↓

load artifact

↓

list artifacts

↓

remove artifact
```

Weitere Operationen werden erst eingeführt, wenn ein konkreter Bedarf entsteht.

---

# In-Memory Provider

Die bestehende In-Memory Persistence Implementation wird als Referenzimplementierung verwendet.

Sie dient als:

- Testgrundlage
- Entwicklungsumgebung
- Validierung der Persistence Boundary

Sie stellt keine endgültige Storage-Lösung dar.

---

# Technology Independence

Diese Entscheidung legt weiterhin keine konkrete Speichertechnologie fest.

Nicht Bestandteil dieser ADR sind:

- SQLite
- Dateisystemstruktur
- Datenbankmodell
- Cloud Storage
- Verschlüsselung
- Synchronisationsprotokolle

Diese Entscheidungen werden durch spätere ADRs getroffen.

---

# Consequences

## Positive Consequences

- klare technische Schnittstelle
- austauschbare Storage Implementierungen
- bessere Testbarkeit
- Vorbereitung für unterschiedliche Speichertechnologien
- geringe Kopplung zwischen Recorder und Storage
- klare Verantwortlichkeiten

---

## Negative Consequences

- zusätzliche Abstraktionsschicht
- zusätzlicher Interface-Aufwand
- mögliche Überdimensionierung bei sehr einfachen Storage-Lösungen

Diese Nachteile werden bewusst akzeptiert.

---

# Considered Alternatives

## Direct Storage Access

Nicht gewählt.

Begründung:

Der Recorder Workflow würde direkt von einer konkreten Speicherlösung abhängen.

Dies würde die Trennung zwischen Aufnahme und Speicherung verletzen.

---

## Artifact Owns Persistence

Nicht gewählt.

Begründung:

Das Recording Artifact würde technische Infrastrukturverantwortung übernehmen.

Dies widerspricht der Trennung zwischen technischem Modell und Persistenz.

---

# Relationship to Existing Architecture

Diese Entscheidung erweitert:

- ADR-041 Local Recording Artifact and Storage Boundary
- ADR-042 Recording Artifact Model and Lifecycle Boundary
- ADR-043 Local Recording Persistence Boundary

Sie definiert die technische Schnittstelle innerhalb der Persistence Boundary.

---

# Future Decisions

Spätere Entscheidungen werden unter anderem behandeln:

- konkrete Storage Provider Implementierungen
- Dateisystem-basierte Speicherung
- Datenbank-basierte Speicherung
- Verschlüsselung persistierter Daten
- Synchronisationsintegration

Diese Entscheidungen erfolgen unabhängig von dieser Architekturentscheidung.

---

# Status

Diese Entscheidung definiert das Persistence Provider Interface als technische Grundlage für austauschbare lokale Speicherung von Recording Artifacts.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# Context

ADR-042 introduced the Recording Artifact as an independent technical model with its own lifecycle.

ADR-043 defined the Local Recording Persistence Boundary.

This creates a technical separation between:

```text
Recording Artifact

↓

Persistence Boundary

↓

Local Storage
```

However, ADR-043 intentionally defines only the architectural boundary.

A further decision is required:

How components inside this boundary communicate.

The persistence layer requires a technical interface that:

- remains independent from concrete storage technologies
- enables testable implementations
- supports future storage variants
- does not introduce dependencies into domain logic

Therefore, the Persistence Boundary requires a defined provider contract.

---

# Decision

NC-PoRe introduces a **Persistence Provider Interface**.

Local storage of Recording Artifacts is performed exclusively through implementations of this interface.

The architecture is:

```text
Recording Artifact

↓

Persistence Provider Interface

↓

Persistence Provider Implementation

↓

Local Storage Backend
```

The Recorder Workflow and Recording Artifact model do not know any concrete storage implementation.

---

# Architectural Principle

The Persistence Provider Interface separates:

- the requirement to preserve data
- technical management of persisted data
- concrete storage implementation

A Persistence Provider defines:

**Which persistence operations are available.**

A concrete implementation defines:

**How these operations are technically implemented.**

---

# Responsibilities

## Persistence Provider Interface

The interface is responsible for:

- defining persistence operations
- technical storage abstraction
- exchangeability of providers
- clear technical boundary between Recorder and Storage

The interface is not responsible for:

- production domain rules
- Recording Lifecycle decisions
- synchronization logic
- user interaction

---

## Persistence Provider Implementation

A concrete implementation is responsible for:

- storing Recording Artifacts
- loading stored Artifacts
- managing local persistence data
- implementing storage-specific details

An implementation is not responsible for:

- creating Recording Artifacts
- domain states
- workflow coordination

---

## Recorder Workflow

The Recorder Workflow uses only the Persistence Provider Interface.

It is responsible for:

- passing Recording Artifacts
- requesting persistence operations
- coordinating the technical workflow

The Recorder Workflow does not know:

- filesystems
- databases
- concrete storage formats

---

# Initial Interface Scope

The Persistence Provider Interface initially defines these basic operations:

```text
store artifact

↓

load artifact

↓

list artifacts

↓

remove artifact
```

Additional operations are introduced only when a concrete requirement exists.

---

# In-Memory Provider

The existing In-Memory Persistence Implementation is used as a reference implementation.

It serves as:

- test foundation
- development environment
- validation of the Persistence Boundary

It is not considered a final storage solution.

---

# Technology Independence

This decision does not define a specific storage technology.

The following are not part of this ADR:

- SQLite
- filesystem structure
- database model
- cloud storage
- encryption
- synchronization protocols

These decisions will be addressed by later ADRs.

---

# Consequences

## Positive Consequences

- clear technical interface
- replaceable storage implementations
- improved testability
- preparation for different storage technologies
- reduced coupling between Recorder and Storage
- clear responsibilities

---

## Negative Consequences

- additional abstraction layer
- additional interface effort
- possible overengineering for very simple storage solutions

These disadvantages are consciously accepted.

---

# Considered Alternatives

## Direct Storage Access

Rejected.

Reason:

The Recorder Workflow would directly depend on a concrete storage solution.

This would violate the separation between recording and storage.

---

## Artifact Owns Persistence

Rejected.

Reason:

The Recording Artifact would take responsibility for technical infrastructure.

This contradicts the separation between technical model and persistence.

---

# Relationship to Existing Architecture

This decision extends:

- ADR-041 Local Recording Artifact and Storage Boundary
- ADR-042 Recording Artifact Model and Lifecycle Boundary
- ADR-043 Local Recording Persistence Boundary

It defines the technical interface inside the Persistence Boundary.

---

# Future Decisions

Future decisions will address:

- concrete Storage Provider implementations
- filesystem-based storage
- database-based storage
- encryption of persisted data
- synchronization integration

These decisions will be made independently from this architecture decision.

---

# Status

This decision defines the Persistence Provider Interface as the technical foundation for replaceable local storage of Recording Artifacts.
