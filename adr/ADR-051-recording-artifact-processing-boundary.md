# ADR-051: Recording Artifact Processing Boundary

* Status: Accepted
* Date: 2026-08-05
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

---

# Kontext

NC-PoRe trennt technische Verantwortlichkeiten zwischen mehreren Komponenten:

* CaptureProvider stellt Audioaufnahme-Funktionalität bereit.
* RecorderWorkflow koordiniert den Ablauf einer Aufnahme.
* CaptureResult beschreibt das technische Ergebnis einer abgeschlossenen Capture-Operation.
* RecordingArtifact repräsentiert das technische Ergebnis einer lokalen Aufnahme.
* RecordingArtifactFactory erzeugt RecordingArtifact-Instanzen.
* ArtifactCoordinator koordiniert Registrierung und Persistenz.
* LocalArtifactRegistry verwaltet bekannte lokale Artifact-Referenzen.
* PersistenceProvider definiert die Persistenzgrenze.

Mit ADR-049 wurde definiert, dass die Erzeugung eines RecordingArtifact ein eigener Schritt innerhalb der Workflow-Integration ist.

Mit ADR-050 wurde die Erzeugung von RecordingArtifact durch eine Factory gekapselt.

Damit existieren die notwendigen Einzelkomponenten:

CaptureProvider

↓

CaptureResult

↓

RecordingArtifactFactory

↓

RecordingArtifact

↓

ArtifactCoordinator

Es fehlt jedoch eine definierte Verantwortung für die Verbindung dieser Schritte.

---

# Problem

Der RecorderWorkflow darf nicht die vollständige technische Verarbeitung eines Capture-Ergebnisses übernehmen.

Seine Verantwortung bleibt:

* Session-Zustände koordinieren
* Capture starten
* Capture stoppen
* Ablauf steuern

Wenn der Workflow zusätzlich Artifact-Verarbeitung implementieren würde, müsste er Kenntnisse besitzen über:

* RecordingArtifact
* RecordingArtifactFactory
* ArtifactCoordinator
* Persistenzabläufe

Dadurch würde die Trennung zwischen Workflow- und Artifact-Schicht aufgehoben.

Es wird daher eine zusätzliche Grenze benötigt, die ein abgeschlossenes Capture-Ergebnis in den nächsten technischen Verarbeitungsschritt überführt.

---

# Entscheidung

NC-PoRe führt eine RecordingArtifactProcessor-Komponente ein.

Der RecordingArtifactProcessor übernimmt die Verarbeitung eines abgeschlossenen CaptureResult.

Seine Verantwortung:

* CaptureResult entgegennehmen
* RecordingArtifactFactory zur Erzeugung eines Artifacts verwenden
* ArtifactCoordinator zur weiteren Verarbeitung verwenden

Der Processor übernimmt nicht:

* Audioaufnahme
* Workflow-Steuerung
* Persistenzimplementierung
* Registry-Implementierung
* Synchronisation

Die Verantwortlichkeiten bleiben getrennt:

CaptureProvider

↓

CaptureResult

↓

RecordingArtifactProcessor

↓

RecordingArtifactFactory

↓

RecordingArtifact

↓

ArtifactCoordinator

↓

LocalArtifactRegistry / PersistenceProvider

---

# Begründung

Die neue Boundary trennt drei unterschiedliche Verantwortlichkeiten:

## Capture

Erzeugt ein technisches Aufnahmeergebnis.

Verantwortlich:

CaptureProvider

---

## Processing

Überführt ein technisches Ergebnis in den nächsten Artifact-Lifecycle-Schritt.

Verantwortlich:

RecordingArtifactProcessor

---

## Artifact Management

Verwaltet Registrierung und Persistenz.

Verantwortlich:

ArtifactCoordinator

Diese Trennung verhindert, dass einzelne Komponenten Wissen über fremde Verantwortungsbereiche benötigen.

---

# Konsequenzen

## Positiv

* RecorderWorkflow bleibt auf Ablaufkoordination beschränkt.
* Artifact-Verarbeitung erhält eine eigene technische Grenze.
* Factory und Coordinator können unabhängig getestet werden.
* Zukünftige Verarbeitungsschritte können innerhalb des Processors ergänzt werden.

## Negativ

* Eine zusätzliche Komponente entsteht.
* Der Ablauf enthält einen zusätzlichen Verarbeitungsschritt.

Diese zusätzliche Komplexität wird akzeptiert, weil sie eine langfristige Architekturgrenze schafft.

---

# Nicht Teil dieser Entscheidung

Diese ADR definiert nicht:

* konkrete Artifact-Felder
* Persistenzmechanismen
* Synchronisationsstrategien
* Recovery-Verfahren
* Audioformate
* Exportprozesse

Diese Themen bleiben durch andere ADRs getrennt.

---

# Beziehung zu anderen ADRs

Diese Entscheidung baut auf folgenden ADRs auf:

* ADR-039 Recording Architecture and Capture Boundary
* ADR-040 Recorder Workflow and Capture Lifecycle Coordination
* ADR-042 Recording Artifact Model and Lifecycle Boundary
* ADR-047 Local Artifact Registry and Discovery Strategy
* ADR-048 Artifact Registry and Persistence Coordination Boundary
* ADR-049 Artifact Creation and Workflow Integration
* ADR-050 Recording Artifact Factory

---

# English Version ([Deutsche Version oben](#deutsch))

---

# Context

NC-PoRe separates technical responsibilities between multiple components:

* CaptureProvider provides audio capture functionality.
* RecorderWorkflow coordinates recording execution.
* CaptureResult describes the technical result of a completed capture operation.
* RecordingArtifact represents the technical result of a local recording.
* RecordingArtifactFactory creates RecordingArtifact instances.
* ArtifactCoordinator coordinates registration and persistence.
* LocalArtifactRegistry manages known local artifact references.
* PersistenceProvider defines the persistence boundary.

ADR-049 defined that RecordingArtifact creation is a separate step within workflow integration.

ADR-050 encapsulated RecordingArtifact creation through a factory.

The required individual components now exist:

CaptureProvider

↓

CaptureResult

↓

RecordingArtifactFactory

↓

RecordingArtifact

↓

ArtifactCoordinator

A defined responsibility connecting these steps is still missing.

---

# Problem

RecorderWorkflow must not implement complete technical processing of capture results.

Its responsibility remains:

* coordinating session states
* starting capture
* stopping capture
* controlling execution flow

If workflow logic also handled artifact processing, it would require knowledge about:

* RecordingArtifact
* RecordingArtifactFactory
* ArtifactCoordinator
* persistence processes

This would break the separation between workflow and artifact responsibilities.

A dedicated boundary is therefore required to transform completed capture results into the next technical processing step.

---

# Decision

NC-PoRe introduces a RecordingArtifactProcessor component.

The RecordingArtifactProcessor processes completed CaptureResult instances.

Its responsibilities:

* receive CaptureResult
* use RecordingArtifactFactory to create artifacts
* use ArtifactCoordinator for further handling

The processor does not handle:

* audio capture
* workflow control
* persistence implementation
* registry implementation
* synchronization

Responsibilities remain separated:

CaptureProvider

↓

CaptureResult

↓

RecordingArtifactProcessor

↓

RecordingArtifactFactory

↓

RecordingArtifact

↓

ArtifactCoordinator

↓

LocalArtifactRegistry / PersistenceProvider

---

# Rationale

The new boundary separates three different responsibilities:

## Capture

Creates a technical recording result.

Responsible:

CaptureProvider

---

## Processing

Transforms a technical result into the next artifact lifecycle step.

Responsible:

RecordingArtifactProcessor

---

## Artifact Management

Handles registration and persistence.

Responsible:

ArtifactCoordinator

This separation prevents individual components from requiring knowledge about unrelated responsibilities.

---

# Consequences

## Positive

* RecorderWorkflow remains focused on workflow coordination.
* Artifact processing receives its own technical boundary.
* Factory and coordinator can be tested independently.
* Future processing steps can be added within the processor.

## Negative

* An additional component is introduced.
* The processing chain contains another step.

This additional complexity is accepted because it establishes a long-term architectural boundary.

---

# Not Part of This Decision

This ADR does not define:

* concrete artifact fields
* persistence mechanisms
* synchronization strategies
* recovery procedures
* audio formats
* export processes

These topics remain separated through other ADRs.

---

# Relationship to Other ADRs

This decision builds upon:

* ADR-039 Recording Architecture and Capture Boundary
* ADR-040 Recorder Workflow and Capture Lifecycle Coordination
* ADR-042 Recording Artifact Model and Lifecycle Boundary
* ADR-047 Local Artifact Registry and Discovery Strategy
* ADR-048 Artifact Registry and Persistence Coordination Boundary
* ADR-049 Artifact Creation and Workflow Integration
* ADR-050 Recording Artifact Factory
