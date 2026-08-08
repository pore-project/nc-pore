# ADR-053: Artifact Recovery and Consistency Boundary

- Status: Accepted
- Date: 2026-08-08

---

# Deutsch ([English version below](#english-version))

---

# Kontext

NC-PoRe besitzt mit dem Local Artifact Registry eine technische Discovery-Schicht für lokal bekannte Recording Artifacts.

Die Registry enthält jedoch nicht die eigentlichen Artefaktdaten. Sie enthält lediglich technische Referenzen auf lokal bekannte Artifacts.

Damit entsteht eine mögliche Inkonsistenz zwischen:

- persistierten Recording Artifacts
- dem lokalen Artifact Registry

Eine solche Inkonsistenz kann beispielsweise entstehen, wenn ein Artifact bereits persistiert wurde, die Registry jedoch nicht mehr den entsprechenden Eintrag enthält.

Die Registry darf deshalb nicht als alleinige Quelle für die Existenz eines lokal persistierten Artifacts betrachtet werden.

NC-PoRe benötigt daher eine definierte Recovery-Grenze, über die die Registry aus den persistierten Daten wieder aufgebaut werden kann.

---

# Entscheidung

NC-PoRe führt eine eigene ArtifactRecoveryService-Komponente ein.

Die Recovery-Komponente ist ausschließlich für die Wiederherstellung des lokalen Discovery-Zustands aus persistierten Recording Artifacts verantwortlich.

Der Recovery-Prozess:

1. liest die vorhandenen Recording Artifacts über die PersistenceProvider-Schnittstelle,
2. prüft, ob für jedes Artifact bereits ein Registry-Eintrag existiert,
3. legt fehlende Registry-Einträge an,
4. verändert bereits vorhandene Registry-Einträge nicht.

Die Recovery verwendet damit die Persistence Boundary als Quelle für die Wiederherstellung der lokalen Registry-Kenntnis.

Die Recovery-Komponente ist bewusst von der konkreten Persistenzimplementierung unabhängig.

---

# Verantwortlichkeiten

Die Verantwortlichkeiten werden wie folgt getrennt:

```text
PersistenceProvider
        |
        | persisted RecordingArtifacts
        v
ArtifactRecoveryService
        |
        | registry knowledge
        v
LocalArtifactRegistry
```

## PersistenceProvider

Der PersistenceProvider stellt die persistierten Recording Artifacts zur Verfügung.

Er ist die Quelle für die Wiederherstellung.

---

## ArtifactRecoveryService

Der ArtifactRecoveryService verbindet Persistenz und lokale Discovery.

Er entscheidet nicht über:

- Artifact-Lifecycle
- Artifact-Erzeugung
- Speicherungstechnologie
- Synchronisation
- Produktionsworkflow

Seine Aufgabe ist ausschließlich die Wiederherstellung fehlender Registry-Kenntnis.

---

## LocalArtifactRegistry

Die LocalArtifactRegistry enthält technische Referenzen auf lokal bekannte Artifacts.

Sie enthält nicht die persistierten Artefaktdaten selbst.

---

# Recovery-Verhalten

Die Recovery ist additiv.

Ein fehlender Registry-Eintrag wird ergänzt:

```text
Persistenz:
Artifact A

Registry:
kein Eintrag

↓

Recovery

↓

Registry:
Artifact A
```

Ein bereits vorhandener Registry-Eintrag wird nicht erneut angelegt:

```text
Persistenz:
Artifact A

Registry:
Artifact A

↓

Recovery

↓

Registry:
Artifact A
```

Die Recovery entfernt keine Registry-Einträge.

Damit ist die Recovery bewusst konservativ.

Sie stellt aus der Persistenz fehlendes Wissen wieder her, ohne bestehendes Registry-Wissen automatisch zu löschen.

---

# Quelle der Wahrheit

Für die Recovery gilt:

> Persistierte Recording Artifacts sind die Quelle für die Wiederherstellung der lokalen Artifact-Discovery.

Die Registry ist dagegen eine lokale Discovery-Struktur.

Sie ist keine unabhängige Quelle für die dauerhafte Existenz eines Artifacts.

Damit wird zwischen:

- persistierten Daten
- lokalem Discovery-Zustand

unterschieden.

---

# Technische Grenze

Die Recovery arbeitet ausschließlich über die bestehende PersistenceProvider-Schnittstelle.

Dadurch bleibt die Recovery unabhängig von der konkreten Persistenzimplementierung.

Die gleiche Recovery-Logik kann daher beispielsweise mit:

- InMemoryPersistenceProvider
- FilesystemPersistenceProvider
- zukünftigen Persistence Providern

verwendet werden.

Die Recovery kennt weder Dateipfade noch Dateiformate noch konkrete Speichertechnologien.

---

# Abgrenzung zur Artifact Coordination

Die Artifact Coordination und die Recovery haben unterschiedliche Aufgaben.

Die ArtifactCoordinator-Komponente behandelt den normalen Ablauf eines neu erzeugten Artifacts:

```text
RecordingArtifact
        |
        v
Registry
        |
        v
Persistence
```

Die Recovery behandelt dagegen die Wiederherstellung nach einer bereits bestehenden Persistenz:

```text
Persistence
        |
        v
ArtifactRecoveryService
        |
        v
Registry
```

Damit wird der normale Produktionspfad nicht mit Recovery-Logik vermischt.

---

# Verhalten bei fehlenden Daten

Existiert ein Artifact nicht in der Persistenz, erzeugt die Recovery keinen Registry-Eintrag.

Beispiel:

```text
Persistence:
kein Artifact A

Registry:
kein Artifact A

↓

Recovery

↓

Persistence:
kein Artifact A

Registry:
kein Artifact A
```

Die Recovery erzeugt keine neuen Recording Artifacts.

Sie rekonstruiert ausschließlich vorhandene persistierte Informationen.

---

# Verhalten bei bestehenden Registry-Einträgen

Existiert bereits ein Registry-Eintrag, wird dieser durch die Recovery nicht ersetzt.

Damit übernimmt die Recovery keine allgemeine Registry-Bereinigung.

Insbesondere entscheidet sie nicht darüber, ob ein vorhandener Registry-Eintrag möglicherweise veraltet oder ungültig ist.

Solche Konsistenzregeln bleiben einer zukünftigen, ausdrücklich dafür vorgesehenen Funktion vorbehalten.

---

# Konsequenzen

## Vorteile

- Persistenz und lokale Discovery bleiben getrennt.
- Recovery ist unabhängig von der konkreten Speichertechnologie.
- Ein Verlust lokaler Registry-Kenntnis kann aus der Persistenz rekonstruiert werden.
- Der normale Artifact-Workflow bleibt von Recovery-Logik getrennt.
- Die bestehende Persistence Boundary wird wiederverwendet.

## Nachteile

- Recovery benötigt Zugriff auf die persistierten Artifacts.
- Die Registry kann nach einer Recovery zusätzliche Einträge enthalten, die vorher nicht vorhanden waren.
- Die Recovery ist bewusst nicht für vollständige Konsistenzprüfung oder Bereinigung verantwortlich.

---

# Nicht Teil dieser Entscheidung

Diese ADR definiert ausdrücklich nicht:

- Synchronisation zwischen Geräten
- Konfliktauflösung
- verteilte Artifact-Verwaltung
- Datenintegrität der eigentlichen Artifact-Dateien
- Garbage Collection
- automatische Entfernung veralteter Registry-Einträge
- Recovery beschädigter oder unlesbarer Artefaktdaten

Diese Themen können durch spätere ADRs behandelt werden.

---

# Testabsicherung

Die Recovery Boundary wird durch Tests abgesichert.

Die Tests stellen insbesondere sicher, dass:

- persistierte Artifacts in die Registry übernommen werden,
- fehlende persistierte Artifacts keine Registry-Einträge erzeugen,
- vorhandene Registry-Einträge erhalten bleiben.

Damit wird die in dieser ADR definierte Recovery-Verantwortung explizit gegen unbeabsichtigte Erweiterungen geschützt.

---

# Beziehung zu anderen ADRs

Diese ADR baut insbesondere auf folgenden Entscheidungen auf:

- ADR-047 Local Artifact Registry and Discovery Strategy
- ADR-051 Recording Artifact Processing Boundary
- ADR-052 Local Filesystem Persistence Provider
- ADR-046 Local Artifact Recovery and Consistency Strategy

ADR-047 definiert die lokale Artifact Registry.

ADR-051 definiert die Grenze für die Verarbeitung von Capture Results zu Recording Artifacts.

ADR-052 definiert einen konkreten Persistence Provider für lokale Speicherung.

ADR-046 definiert die übergeordnete Recovery-Strategie für lokale Recording Artifacts.

ADR-053 definiert die technische Grenze für die Wiederherstellung der lokalen Registry aus persistierten Artifacts.

---

# Implementierungsstatus

Die Recovery Boundary ist implementiert.

Die Implementierung besteht aus:

- ArtifactRecoveryService
- Verwendung von PersistenceProvider
- Wiederaufbau fehlender LocalArtifactRegistry-Einträge
- automatisierten Tests für das definierte Recovery-Verhalten

---

# Entscheidung

Die Entscheidung wird als **Accepted** geführt.

Die Recovery von lokalem Artifact-Discovery-Wissen wird als eigenständige technische Verantwortung behandelt und nicht in Persistence, Registry, Artifact Processing oder Workflow integriert.

---

# English ([Deutsche Version oben](#deutsch))

---

# Context

NC-PoRe has a technical discovery layer for locally known Recording Artifacts in the form of the Local Artifact Registry.

However, the registry does not contain the actual artifact data. It only contains technical references to locally known artifacts.

This creates a possible inconsistency between:

- persisted Recording Artifacts
- the local Artifact Registry

Such an inconsistency can occur, for example, when an artifact has already been persisted but the registry no longer contains the corresponding entry.

The registry must therefore not be considered the sole source for determining the existence of a locally persisted artifact.

NC-PoRe therefore requires a defined recovery boundary through which the registry can be rebuilt from persisted data.

---

# Decision

NC-PoRe introduces a dedicated ArtifactRecoveryService component.

The recovery component is exclusively responsible for restoring local discovery state from persisted Recording Artifacts.

The recovery process:

1. reads existing Recording Artifacts through the PersistenceProvider interface,
2. checks whether a registry entry already exists for each artifact,
3. creates missing registry entries,
4. does not modify existing registry entries.

Recovery therefore uses the Persistence Boundary as the source for restoring local registry knowledge.

The recovery component is deliberately independent of the concrete persistence implementation.

---

# Responsibilities

Responsibilities are separated as follows:

```text
PersistenceProvider
        |
        | persisted RecordingArtifacts
        v
ArtifactRecoveryService
        |
        | registry knowledge
        v
LocalArtifactRegistry
```

## PersistenceProvider

The PersistenceProvider provides persisted Recording Artifacts.

It is the source used for recovery.

---

## ArtifactRecoveryService

The ArtifactRecoveryService connects persistence and local discovery.

It does not decide:

- artifact lifecycle
- artifact creation
- storage technology
- synchronization
- production workflow

Its sole responsibility is restoring missing registry knowledge.

---

## LocalArtifactRegistry

The LocalArtifactRegistry contains technical references to locally known artifacts.

It does not contain the persisted artifact data itself.

---

# Recovery Behavior

Recovery is additive.

A missing registry entry is added:

```text
Persistence:
Artifact A

Registry:
no entry

↓

Recovery

↓

Registry:
Artifact A
```

An existing registry entry is not added again:

```text
Persistence:
Artifact A

Registry:
Artifact A

↓

Recovery

↓

Registry:
Artifact A
```

Recovery does not remove registry entries.

Recovery is therefore deliberately conservative.

It restores missing knowledge from persistence without automatically removing existing registry knowledge.

---

# Source of Truth

For recovery, the following rule applies:

> Persisted Recording Artifacts are the source for restoring local Artifact Discovery.

The registry, in contrast, is a local discovery structure.

It is not an independent source for the durable existence of an artifact.

This distinguishes between:

- persisted data
- local discovery state

---

# Technical Boundary

Recovery operates exclusively through the existing PersistenceProvider interface.

This keeps recovery independent of the concrete persistence implementation.

The same recovery logic can therefore be used with, for example:

- InMemoryPersistenceProvider
- FilesystemPersistenceProvider
- future Persistence Providers

Recovery knows neither file paths nor file formats nor concrete storage technologies.

---

# Separation from Artifact Coordination

Artifact Coordination and Recovery have different responsibilities.

The ArtifactCoordinator handles the normal flow of a newly created artifact:

```text
RecordingArtifact
        |
        v
Registry
        |
        v
Persistence
```

Recovery, in contrast, handles restoration from already persisted data:

```text
Persistence
        |
        v
ArtifactRecoveryService
        |
        v
Registry
```

This keeps the normal production path separate from recovery logic.

---

# Behavior for Missing Data

If an artifact does not exist in persistence, recovery does not create a registry entry.

Example:

```text
Persistence:
no Artifact A

Registry:
no Artifact A

↓

Recovery

↓

Persistence:
no Artifact A

Registry:
no Artifact A
```

Recovery does not create new Recording Artifacts.

It only reconstructs information that already exists in persistence.

---

# Behavior for Existing Registry Entries

If a registry entry already exists, recovery does not replace it.

Recovery therefore does not perform general registry cleanup.

In particular, it does not decide whether an existing registry entry may be outdated or invalid.

Such consistency rules remain the responsibility of a future function explicitly designed for that purpose.

---

# Consequences

## Advantages

- Persistence and local discovery remain separated.
- Recovery is independent of the concrete storage technology.
- Lost local registry knowledge can be reconstructed from persistence.
- The normal artifact workflow remains separate from recovery logic.
- The existing Persistence Boundary is reused.

## Disadvantages

- Recovery requires access to persisted artifacts.
- The registry may contain additional entries after recovery that were not present before.
- Recovery deliberately does not perform complete consistency validation or cleanup.

---

# Not Part of This Decision

This ADR explicitly does not define:

- synchronization between devices
- conflict resolution
- distributed artifact management
- data integrity of the actual artifact files
- garbage collection
- automatic removal of outdated registry entries
- recovery of damaged or unreadable artifact data

These topics may be addressed by future ADRs.

---

# Test Coverage

The Recovery Boundary is protected by tests.

The tests specifically ensure that:

- persisted artifacts are added to the registry,
- missing persisted artifacts do not create registry entries,
- existing registry entries are preserved.

This explicitly protects the recovery responsibility defined by this ADR against unintended extensions.

---

# Relationship to Other ADRs

This ADR builds in particular on the following decisions:

- ADR-047 Local Artifact Registry and Discovery Strategy
- ADR-051 Recording Artifact Processing Boundary
- ADR-052 Local Filesystem Persistence Provider
- ADR-046 Local Artifact Recovery and Consistency Strategy

ADR-047 defines the local Artifact Registry.

ADR-051 defines the boundary for processing Capture Results into Recording Artifacts.

ADR-052 defines a concrete Persistence Provider for local storage.

ADR-046 defines the overarching recovery strategy for local Recording Artifacts.

ADR-053 defines the technical boundary for restoring the local registry from persisted artifacts.

---

# Implementation Status

The Recovery Boundary is implemented.

The implementation consists of:

- ArtifactRecoveryService
- use of PersistenceProvider
- rebuilding missing LocalArtifactRegistry entries
- automated tests for the defined recovery behavior

---

# Decision

The decision is recorded as **Accepted**.

Recovery of local artifact discovery knowledge is treated as an independent technical responsibility and is not integrated into Persistence, Registry, Artifact Processing, or Workflow.
