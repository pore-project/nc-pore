# ADR-050: Recording Artifact Factory

* Status: Accepted
* Date: 2026-08-05
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

---

# Kontext

NC-PoRe trennt verschiedene technische Verantwortlichkeiten:

* CaptureProvider stellt Audioaufnahme-Funktionalität bereit.
* RecorderWorkflow koordiniert den Aufnahmeablauf.
* CaptureResult beschreibt das technische Ergebnis einer Capture-Operation.
* RecordingArtifact repräsentiert das technische Ergebnis einer lokalen Aufnahme.
* LocalArtifactRegistry verwaltet bekannte lokale Artifact-Referenzen.
* PersistenceProvider definiert die Persistenzgrenze.

Mit ADR-049 wurde definiert, dass die Erzeugung eines Recording Artifacts ein eigener Schritt innerhalb der Recorder-Workflow-Integration ist.

Damit entsteht die nächste Architekturfrage:

> Wie wird die Erzeugung eines RecordingArtifact technisch gekapselt?

Eine direkte Erzeugung innerhalb des Workflows würde dazu führen, dass der Workflow die interne Konstruktion von RecordingArtifact kennen muss.

Beispiele:

* welche Felder benötigt werden
* wie Identifikatoren erzeugt werden
* welche Initialzustände gelten
* welche Validierungen bei der Erzeugung notwendig sind

Diese Verantwortung gehört nicht in den Workflow.

Der Workflow soll koordinieren:

* Aufnahme starten
* Aufnahme stoppen
* Ergebnis weitergeben
* nächste Verarbeitungsschritte auslösen

Er soll jedoch nicht die Konstruktion technischer Artefakte implementieren.

---

# Problem

Ohne eine separate Erzeugungskomponente entstehen mehrere Probleme:

* Workflow-Code kennt interne Details des Artifact-Modells.
* Änderungen am RecordingArtifact-Modell beeinflussen Workflow-Code.
* Erzeugungslogik kann nicht zentral erweitert werden.
* Tests müssen mehr technische Details kennen als notwendig.

Eine reine Builder-Struktur wäre ebenfalls nicht passend.

Ein Builder beschreibt typischerweise:

* schrittweise Konstruktion komplexer Objekte
* optionale Konfiguration
* teilweise unvollständige Zwischenzustände

Die Erzeugung eines RecordingArtifact folgt jedoch einem anderen Muster:

Ein abgeschlossenes technisches Ergebnis wird in ein gültiges Artifact-Modell überführt.

---

# Entscheidung

NC-PoRe verwendet eine RecordingArtifactFactory zur Erzeugung von RecordingArtifact-Instanzen.

Die Factory kapselt:

* Erstellung neuer RecordingArtifact-Objekte
* Zuordnung technischer Identifikatoren
* Initialisierung des Artifact-Zustands
* spätere Erweiterungen der Erzeugungslogik

Die Factory ist Bestandteil der Artifact-Schicht.

Sie übernimmt nicht:

* Workflow-Koordination
* Audioaufnahme
* Persistenz
* Registry-Verwaltung
* Synchronisation

Die Verantwortlichkeiten bleiben getrennt:

CaptureProvider

↓

CaptureResult

↓

RecordingArtifactFactory

↓

RecordingArtifact

↓

LocalArtifactRegistry / PersistenceProvider

---

# Begründung

Die Factory schafft eine klare technische Grenze zwischen:

* dem Ergebnis einer Capture-Operation
* dem daraus entstehenden RecordingArtifact

Dadurch bleibt der Workflow unabhängig von Details der Artifact-Erzeugung.

Änderungen an der Artifact-Konstruktion können innerhalb der Factory umgesetzt werden, ohne dass Workflow-Logik angepasst werden muss.

---

# Konsequenzen

## Positiv

* Artifact-Erzeugung ist an einer Stelle definiert.
* Workflow-Code bleibt auf Koordination beschränkt.
* Änderungen am Artifact-Modell haben weniger Auswirkungen.
* Tests können die Erzeugung unabhängig prüfen.

## Negativ

* Eine zusätzliche Abstraktion entsteht.
* Kleine Erzeugungsschritte benötigen eine eigene Komponente.

Diese zusätzliche Komplexität wird akzeptiert, weil die Factory eine langfristige Architekturgrenze bildet.

---

# Nicht Teil dieser Entscheidung

Diese ADR definiert nicht:

* die Speicherung von Artifacts
* die Synchronisation zwischen Geräten
* Recovery-Mechanismen
* konkrete Audioformate
* Exportprozesse

Diese Themen bleiben durch andere Architekturentscheidungen getrennt.

---

# Beziehung zu anderen ADRs

Diese Entscheidung baut auf folgenden ADRs auf:

* ADR-039 Recording Architecture and Capture Boundary
* ADR-040 Recorder Workflow and Capture Lifecycle Coordination
* ADR-042 Recording Artifact Model and Lifecycle Boundary
* ADR-047 Local Artifact Registry and Discovery Strategy
* ADR-048 Artifact Registry and Persistence Coordination Boundary
* ADR-049 Artifact Creation and Workflow Integration

---

# English Version

---

# Context

NC-PoRe separates different technical responsibilities:

* CaptureProvider provides audio capture functionality.
* RecorderWorkflow coordinates the recording workflow.
* CaptureResult describes the technical result of a capture operation.
* RecordingArtifact represents the technical result of a local recording.
* LocalArtifactRegistry manages known local artifact references.
* PersistenceProvider defines the persistence boundary.

ADR-049 defined that the creation of a Recording Artifact is a separate step within recorder workflow integration.

This creates the next architectural question:

> How should the creation of a RecordingArtifact be technically encapsulated?

Creating artifacts directly inside the workflow would require the workflow to know internal construction details of RecordingArtifact.

Examples:

* required fields
* identifier creation
* initial state handling
* future creation validation rules

This responsibility does not belong to the workflow.

The workflow should coordinate:

* starting recording
* stopping recording
* forwarding results
* triggering subsequent processing steps

It should not implement artifact construction.

---

# Problem

Without a separate creation component, several problems occur:

* Workflow code becomes coupled to artifact internals.
* Changes to RecordingArtifact affect workflow implementation.
* Creation logic cannot be extended centrally.
* Tests need unnecessary knowledge about internal construction details.

A builder pattern would also not be appropriate.

A builder usually describes:

* step-by-step construction of complex objects
* optional configuration
* incomplete intermediate states

RecordingArtifact creation follows a different pattern:

A completed technical result is transformed into a valid artifact model.

---

# Decision

NC-PoRe uses a RecordingArtifactFactory for creating RecordingArtifact instances.

The factory encapsulates:

* creation of new RecordingArtifact objects
* assignment of technical identifiers
* initialization of artifact state
* future extensions of creation logic

The factory belongs to the Artifact layer.

It does not handle:

* workflow coordination
* audio capture
* persistence
* registry management
* synchronization

Responsibilities remain separated:

CaptureProvider

↓

CaptureResult

↓

RecordingArtifactFactory

↓

RecordingArtifact

↓

LocalArtifactRegistry / PersistenceProvider

---

# Rationale

The factory creates a clear technical boundary between:

* the result of a capture operation
* the resulting RecordingArtifact

This keeps workflow logic independent from artifact creation details.

Changes to artifact construction can be implemented inside the factory without modifying workflow coordination logic.

---

# Consequences

## Positive

* Artifact creation is defined in one place.
* Workflow code remains focused on coordination.
* Changes to the artifact model have fewer impacts.
* Creation logic can be tested independently.

## Negative

* An additional abstraction is introduced.
* Small creation steps require a dedicated component.

This additional complexity is accepted because the factory establishes a long-term architectural boundary.

---

# Not Part of This Decision

This ADR does not define:

* artifact storage
* synchronization between devices
* recovery mechanisms
* concrete audio formats
* export processes

These topics remain separated through other architecture decisions.

---

# Relationship to Other ADRs

This decision builds upon:

* ADR-039 Recording Architecture and Capture Boundary
* ADR-040 Recorder Workflow and Capture Lifecycle Coordination
* ADR-042 Recording Artifact Model and Lifecycle Boundary
* ADR-047 Local Artifact Registry and Discovery Strategy
* ADR-048 Artifact Registry and Persistence Coordination Boundary
* ADR-049 Artifact Creation and Workflow Integration
