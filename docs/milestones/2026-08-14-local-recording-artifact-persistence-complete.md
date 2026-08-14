# Local Recording Artifact Persistence Complete

- Date: 2026-08-14
- Status: Completed
- Related issue: #9

---

# Deutsch ([English version below](#english-version))

---

# Zweck

Dieser Milestone dokumentiert den Abschluss des lokalen RecordingArtifact-Pfads von der abgeschlossenen Aufnahme bis zum dauerhaft persistierten und nach einem Neustart wieder ladbaren RecordingArtifact.

Der Milestone fasst die zuvor einzeln implementierten und dokumentierten Architektur- und Implementierungsschritte zu einem nachvollziehbaren lokalen End-to-End-Pfad zusammen.

Der Milestone beschreibt keinen vollständigen Recorder im Produktivbetrieb. Er dokumentiert den abgeschlossenen technischen lokalen Persistenzpfad und seine Grenzen.

---

# Abgeschlossener lokaler Ablauf

Der technische Ablauf ist:

ProductionSession / Recording

↓

Recorder Workflow

↓

CaptureResult

↓

RecordingArtifactFactory

↓

RecordingArtifact

↓

Domain Recording Association

↓

Artifact Processing

↓

Artifact Coordination

↓

Persistence Provider

↓

Filesystem Persistence

↓

persistiertes RecordingArtifact

↓

Recovery / Consistency Assessment

↓

wieder aufgebautes Registry-Wissen

Der Ablauf endet bewusst an der lokalen Persistenz- und Recovery-Grenze. Eine Synchronisation mit anderen Geräten oder ein Remote Storage ist nicht Bestandteil dieses Milestones.

---

# Daten- und Verantwortungsgrenzen

## CaptureResult

CaptureResult beschreibt das technische Ergebnis der Capture-Schicht.

Es enthält die für die weitere Verarbeitung erforderlichen Tracks, Chunks und Payload-Daten, ist aber weder ein RecordingArtifact noch eine Persistenzrepräsentation.

Die Capture-Schicht bleibt damit von Artifact Management und Persistence getrennt.

## RecordingArtifact

RecordingArtifact bildet das technische lokale Recording-Artefakt.

Es strukturiert Recording Tracks und Chunks und trägt die Zuordnung zur zugrunde liegenden RecordingSession sowie die optionale Domain Recording Association.

Das Artifact ist unabhängig von der konkreten physischen Speicherung.

## RecordingArtifact Processing

RecordingArtifact Processing bildet die technische Grenze zwischen abgeschlossenem Capture und Artifact Management.

Der Processor:

- übernimmt ein abgeschlossenes CaptureResult,
- erzeugt daraus ein RecordingArtifact,
- übernimmt ProductionId und RecordingId als bestehende Zuordnung,
- setzt das Artifact auf Available,
- übergibt es an Artifact Coordination.

Der Processor enthält weder Capture-Logik noch konkrete Persistenzimplementierung.

## Persistence

Persistence übernimmt ausschließlich die dauerhafte Speicherung und das Laden von RecordingArtifacts.

Der Persistence Provider bleibt hinter einer technischen Schnittstelle verborgen. Die konkrete lokale Implementierung verwendet das in ADR-055 definierte Filesystem-Layout.

---

# Persistenzsemantik

Ein erfolgreich verarbeitetes Artifact wird nach erfolgreicher Persistenz als Stored zurückgegeben.

Bei einem Persistenzfehler wird kein fälschlich als Stored dargestelltes Artifact erzeugt. Das zur Speicherung übergebene Artifact bleibt Available und der Fehler wird an den aufrufenden Verarbeitungspfad weitergegeben.

Wird ein Artifact mit identischer Identität und äquivalentem Inhalt erneut gespeichert, ist der Vorgang idempotent. Es entsteht kein zweites Artifact.

Wird dieselbe Artifact-Identität mit abweichendem Inhalt erneut verwendet, wird der Vorgang als Conflict abgelehnt. Die bestehende persistierte Repräsentation wird nicht stillschweigend überschrieben.

Eine vorhandene, aber unvollständige persistierte Repräsentation wird ebenfalls nicht durch einen normalen Store-Vorgang überschrieben.

---

# Persistierte Daten

Die Filesystem Persistence trennt Artifact-Metadaten von den eigentlichen Recording-Daten.

Die persistierte Repräsentation umfasst insbesondere:

- Artifact-Metadaten
- RecordingSession-Zuordnung
- Domain Recording Association, soweit vorhanden
- Track-Struktur
- Chunk-Metadaten
- tatsächliche Payload-Daten

Die Payload ist damit Bestandteil der dauerhaft gespeicherten RecordingArtifact-Repräsentation und nicht lediglich ein flüchtiges Capture-Ergebnis.

---

# Recovery und Konsistenz

Recovery liest die vorhandene lokale Persistenz und rekonstruiert daraus das Wissen über vorhandene Artifacts.

Recovery erzeugt keine neuen Artifacts, verändert keine Lifecycle-Zustände und implementiert weder Storage noch Synchronisation.

Die Persistenzbewertung unterscheidet unter anderem zwischen gültigen, nicht vorhandenen, unvollständigen und inkonsistenten persistierten Repräsentationen.

Beispiele für erkannte Inkonsistenzen sind:

- fehlende Payload-Daten,
- widersprüchliche Payload-Größen,
- nicht lesbare oder ungültige Artifact-Metadaten.

Damit wird ein unvollständiges oder beschädigtes lokales Artifact nicht stillschweigend als gültig behandelt.

---

# Rückverfolgbarkeit

Die Zuordnung zwischen Domain Recording und technischem RecordingArtifact wird an der definierten Application-/Processing-Grenze hergestellt.

ProductionId und RecordingId werden als bestehende Werte übernommen und im RecordingArtifact erhalten. Die technische Artifact-Schicht erzeugt daraus keine eigene fachliche Interpretation.

Damit bleibt nachvollziehbar, aus welchem Domain Recording ein lokales RecordingArtifact hervorgegangen ist.

---

# Validierung

Der abgeschlossene Pfad wird durch mehrere Testebenen abgesichert.

Relevante Testreferenzen sind insbesondere:

- TEST-23: CaptureResult wird durch Processing in ein koordiniertes und persistiertes RecordingArtifact überführt.
- TEST-24 und TEST-25: Der Recorder Application Flow verbindet Session, Capture, Processing und Artifact Management einschließlich der Domain-Zuordnung.
- TEST-34: Erfolgreiche Persistenz liefert ein Stored Artifact zurück.
- TEST-35: Wiederholte Verarbeitung desselben äquivalenten Artifacts ist idempotent.
- TEST-37: Fehlende Payload wird als unvollständig erkannt.
- TEST-38: Abweichende Payload-Größe wird als inkonsistent erkannt.
- TEST-39: Ungültige Artifact-Metadaten werden als inkonsistent erkannt.
- TEST-40: Ein Persistenzfehler wird weitergegeben; das versuchte Artifact wird nicht fälschlich als Stored behandelt.
- TEST-41: Äquivalenter erneuter Store ist idempotent.
- TEST-42: Eine abweichende Repräsentation derselben Artifact-Identität wird als Conflict abgelehnt.

Die Tests validieren damit sowohl die einzelnen Grenzen als auch die wesentlichen Semantiken des lokalen Persistenzpfads.

---

# Architekturentscheidungen

Dieser Milestone baut insbesondere auf folgenden Entscheidungen auf:

- ADR-051 Recording Artifact Processing Boundary
- ADR-053 Artifact Recovery and Consistency Boundary
- ADR-054 Recording Artifact and Local Recording Data Association
- ADR-055 Filesystem Persistence Layout
- ADR-056 Capture Result and Recording Artifact Data Boundary
- ADR-057 Domain Recording to RecordingArtifact Association Boundary
- ADR-058 Recording Payload Representation
- ADR-059 Recording Payload Filesystem Persistence
- ADR-060 Filesystem Store Semantics

Die vorgelagerten Issues #5, #7 und #8 bilden die wesentlichen fachlichen und technischen Vorbedingungen dieses Milestones.

---

# Bewusst nicht Bestandteil dieses Milestones

Der abgeschlossene lokale Persistenzpfad bedeutet ausdrücklich nicht:

- Synchronisation zwischen mehreren Geräten,
- Remote Storage,
- verteilte Replikation,
- Netzwerkkommunikation für Artifact-Daten,
- Konfliktauflösung zwischen unterschiedlichen Geräten,
- Upload eines lokalen RecordingArtifacts auf einen entfernten Server.

Diese Themen beginnen an einer späteren Grenze. Die lokale Persistenz muss dafür nicht mit einer Synchronisationslogik vermischt werden.

Ebenso sind reale Audio-Hardware, Benutzeroberflächen und ein produktionsfertiger Recorder-Betrieb nicht Bestandteil dieses Milestones.

---

# Ergebnis

Der lokale RecordingArtifact-Pfad ist als abgeschlossener technischer Meilenstein dokumentiert.

Von einem abgeschlossenen CaptureResult kann ein RecordingArtifact erzeugt, seiner Domain Recording Association zugeordnet, lokal persistiert und nach einem Neustart aus der lokalen Persistenz wieder bewertet und in das Registry-Wissen übernommen werden.

Persistenzfehler, unvollständige Daten, inkonsistente Daten, idempotente Wiederholung und Identitätskonflikte besitzen definierte technische Semantik.

Damit ist die lokale RecordingArtifact-Persistenz als klar abgegrenzte technische Grundlage für den nächsten Entwicklungsschritt dokumentiert.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# Purpose

This milestone documents completion of the local RecordingArtifact path from a completed recording through durable persistence to loading and assessing the persisted RecordingArtifact again after a restart.

The milestone consolidates previously implemented and documented architectural and implementation steps into a traceable local end-to-end path.

It does not describe a complete production-ready Recorder. It documents the completed technical local persistence path and its boundaries.

---

# Completed Local Flow

The technical flow is:

ProductionSession / Recording

↓

Recorder Workflow

↓

CaptureResult

↓

RecordingArtifactFactory

↓

RecordingArtifact

↓

Domain Recording Association

↓

Artifact Processing

↓

Artifact Coordination

↓

Persistence Provider

↓

Filesystem Persistence

↓

persisted RecordingArtifact

↓

Recovery / Consistency Assessment

↓

reconstructed registry knowledge

The flow intentionally ends at the local persistence and recovery boundary. Synchronization with other devices and remote storage are outside this milestone.

---

# Data and Responsibility Boundaries

## CaptureResult

CaptureResult represents the technical result of the capture layer.

It contains the tracks, chunks, and payload data required for downstream processing, but it is neither a RecordingArtifact nor a persistence representation.

The capture layer therefore remains separate from Artifact Management and Persistence.

## RecordingArtifact

RecordingArtifact represents the technical local recording artifact.

It structures recording tracks and chunks and carries the association with the underlying RecordingSession as well as the optional Domain Recording Association.

The artifact is independent of the concrete physical storage mechanism.

## RecordingArtifact Processing

RecordingArtifact Processing forms the technical boundary between completed capture and Artifact Management.

The processor:

- accepts a completed CaptureResult,
- creates a RecordingArtifact from it,
- preserves ProductionId and RecordingId as existing associations,
- transitions the artifact to Available,
- passes it to Artifact Coordination.

The processor contains neither capture logic nor concrete persistence implementation.

## Persistence

Persistence is responsible only for durable storage and loading of RecordingArtifacts.

The Persistence Provider remains behind a technical interface. The concrete local implementation uses the filesystem layout defined by ADR-055.

---

# Persistence Semantics

A successfully processed artifact is returned as Stored after successful persistence.

If persistence fails, no artifact is falsely reported as Stored. The artifact submitted for persistence remains Available and the error is propagated to the calling processing path.

When an artifact with identical identity and equivalent content is stored again, the operation is idempotent. No second artifact is created.

When the same artifact identity is reused with different content, the operation is rejected as a Conflict. The existing persisted representation is not silently overwritten.

An existing but incomplete persisted representation is likewise not overwritten by a normal store operation.

---

# Persisted Data

Filesystem Persistence separates artifact metadata from the actual recording data.

The persisted representation includes in particular:

- artifact metadata,
- RecordingSession association,
- Domain Recording Association where present,
- track structure,
- chunk metadata,
- actual payload data.

The payload is therefore part of the durably persisted RecordingArtifact representation rather than merely a transient capture result.

---

# Recovery and Consistency

Recovery reads the existing local persistence and reconstructs knowledge about existing artifacts.

Recovery does not create artifacts, change lifecycle states, implement storage, or perform synchronization.

Persistence assessment distinguishes, among other states, valid, not-found, incomplete, and inconsistent persisted representations.

Examples of detected inconsistencies include:

- missing payload data,
- conflicting payload sizes,
- unreadable or invalid artifact metadata.

Incomplete or damaged local artifacts are therefore not silently treated as valid.

---

# Traceability

The association between a domain recording and its technical RecordingArtifact is established at the defined application/processing boundary.

ProductionId and RecordingId are passed through as existing values and preserved on the RecordingArtifact. The technical artifact layer does not derive its own domain interpretation from them.

This keeps the origin of a local RecordingArtifact traceable to its domain recording.

---

# Validation

The completed path is covered by several test levels.

Relevant test references include:

- TEST-23: CaptureResult is transformed by Processing into a coordinated and persisted RecordingArtifact.
- TEST-24 and TEST-25: The Recorder Application Flow connects Session, Capture, Processing, and Artifact Management including domain association.
- TEST-34: Successful persistence returns a Stored artifact.
- TEST-35: Repeated processing of the same equivalent artifact is idempotent.
- TEST-37: Missing payload is detected as incomplete.
- TEST-38: Payload size disagreement is detected as inconsistent.
- TEST-39: Invalid artifact metadata is detected as inconsistent.
- TEST-40: A persistence failure is propagated and the attempted artifact is not falsely reported as Stored.
- TEST-41: Repeated storage of equivalent content is idempotent.
- TEST-42: A different representation using the same artifact identity is rejected as a Conflict.

The tests therefore validate both the individual boundaries and the essential semantics of the local persistence path.

---

# Architectural Decisions

This milestone builds in particular on:

- ADR-051 Recording Artifact Processing Boundary
- ADR-053 Artifact Recovery and Consistency Boundary
- ADR-054 Recording Artifact and Local Recording Data Association
- ADR-055 Filesystem Persistence Layout
- ADR-056 Capture Result and Recording Artifact Data Boundary
- ADR-057 Domain Recording to RecordingArtifact Association Boundary
- ADR-058 Recording Payload Representation
- ADR-059 Recording Payload Filesystem Persistence
- ADR-060 Filesystem Store Semantics

The preceding issues #5, #7, and #8 form the main functional and technical prerequisites for this milestone.

---

# Explicitly Outside This Milestone

Completion of the local persistence path explicitly does not mean:

- synchronization between multiple devices,
- remote storage,
- distributed replication,
- network communication for artifact data,
- conflict resolution between different devices,
- uploading a local RecordingArtifact to a remote server.

These topics begin at a later boundary. Local persistence must not be mixed with synchronization logic for that purpose.

Real audio hardware, user interfaces, and production-ready Recorder operation are also outside this milestone.

---

# Result

The local RecordingArtifact path is documented as a completed technical milestone.

A completed CaptureResult can be transformed into a RecordingArtifact, associated with its domain recording, persisted locally, and assessed again from local persistence after a restart and incorporated into reconstructed registry knowledge.

Persistence failures, incomplete data, inconsistent data, idempotent repetition, and identity conflicts have defined technical semantics.

The local RecordingArtifact persistence therefore provides a clearly bounded technical foundation for the next development step.
