# NC-PoRe Project Status

- Version: 3.3
- Date: 2026-08-21

---

# Deutsch (English version below)

---

# Project Phase

## Technical Implementation Started

NC-PoRe befindet sich nach Abschluss der Architekturphase in der technischen Umsetzung.

Die grundlegenden Architekturentscheidungen, Entwicklungsprinzipien und technischen Grenzen sind definiert.

Die Core- und Recorder-Komponenten wurden implementiert und durch Tests validiert.

---

# Current Implementation Status

## Core

Implementiert:

- Core Modulstruktur
- ProductionSession Modell
- ProductionSession Lifecycle
- ProductionSession Lifecycle-Invarianten und Zustandsübergänge
- Recording Modell
- Recording Lifecycle
- Participant Modell
- Participation Modell
- Activity History Grundstruktur
- ProductionSession-, Participant- und Participation-Semantik
- sessionbezogene Rollen- und Berechtigungssemantik für Owner, Producer, Participant und Guest
- API Operationen für ProductionSession Lifecycle
- API Operationen für ProductionSession-Verwaltung und Recording-Verknüpfung
- stabile Application/API-Grenzen für Production-Management-Operationen
- Read-API-Operationen für Participants, Recordings und Activity History
- konsistente Domain-/Application-Fehlersemantik für Production-Management-Operationen
- fachliche Validierung von Lifecycle-, Rollen- und Participation-Invarianten an der Domain-Grenze
- Production Activity/History-Semantik einschließlich Actor-, Action-, Target-, Session- und Result-Kontext

Die Production-Management- und Collaboration-Foundation ist damit als zusammenhängende fachliche Grundlage für spätere Clients und Kollaborationsfunktionen implementiert und validiert.

Relevante Architekturentscheidungen:

- ADR-027 Core Architecture and Module Boundaries
- ADR-031 Identity, Authentication and User Roles
- ADR-032 Auditability and Activity History
- ADR-033 Core Architecture
- ADR-034 Implementation Architecture
- ADR-035 Domain Lifecycle and State Transition Management

---

## Recorder

Implementiert:

- Session Modul
- Statusmodell
- Lifecycle-Methoden
- Capture Boundary Interface
- Workflow Coordination Layer
- Recording Artifact Model
- Recording Artifact Track and Chunk Model
- Capture Result Track and Chunk Model
- Capture-to-Artifact Data Boundary
- Artifact Lifecycle Management
- Local Artifact Registry
- Artifact Coordination Boundary
- Artifact Processing Boundary
- Recorder Application Boundary
- Local Recording Artifact Flow
- Artifact Recovery Boundary
- RecordingSessionId Value Object für technische Session-Referenzen
- ArtifactId und RecordingSessionId als explizite Identitätstypen an Artifact-Grenzen
- storage-provider-unabhängige Payload-Referenz und technische Payload-Daten für Recording Chunks
- Persistenz des tatsächlichen Recording-Payloads im Filesystem Persistence Provider
- definierte Filesystem-Store-Semantik für lokale Recording Artifacts
- abgeschlossener lokaler RecordingArtifact-Persistenzpfad einschließlich Recovery und Konsistenzbewertung
- CPAL-basierte konkrete CaptureProvider-Implementierung
- erfolgreicher Aufbau und Start eines lokalen CPAL Input-Streams
- Übernahme empfangener F32-Samples in ein CaptureResult
- durchgängiger Recorder Application Flow von CPAL Capture bis RecordingArtifact-Persistenz
- realer lokaler Audio-Capture bis zur Erzeugung und Verarbeitung eines RecordingArtifacts
- technische Recording-Konfiguration entlang des Capture-to-Artifact-Pfades
- definierte Chunking- und Payload-Grenzen
- produktionsgeeignete Stream-Fehlerpropagation bis zur RecorderApplication-Grenze
- Lifecycle- und Fehlerbehandlung für fehlgeschlagenes Capture

Die lokale technische Recording-Pipeline ist damit als zusammenhängender Capture-to-Artifact-Pfad implementiert und validiert.

Relevante Architekturentscheidungen:

- ADR-039 Recording Architecture and Capture Boundary
- ADR-040 Recorder Workflow and Capture Lifecycle Coordination
- ADR-042 Recording Artifact Model and Lifecycle Boundary
- ADR-047 Local Artifact Registry and Discovery Strategy
- ADR-049 Artifact Creation and Workflow Integration
- ADR-051 Recording Artifact Processing Boundary
- ADR-053 Artifact Recovery and Consistency Boundary
- ADR-054 Local Recording Data Association
- ADR-056 Capture Result and Recording Artifact Data Boundary
- ADR-057 Domain Recording to RecordingArtifact Association Boundary
- ADR-058 Recording Payload Representation
- ADR-059 Recording Payload Filesystem Persistence
- ADR-060 Filesystem Artifact Store Semantics

---

## Persistence

Implementiert:

- Local Recording Persistence Boundary
- Persistence Provider Interface
- In-Memory Persistence Provider
- Filesystem Persistence Provider
- Persistence Integration Tests
- Persistenz des tatsächlichen Recording-Payloads einschließlich temporärer Veröffentlichung und vollständiger Artifact-Verzeichnisse
- definierte Semantik für das Speichern von Recording Artifacts im Filesystem Persistence Provider
- Idempotenz für äquivalente persistierte Artifacts
- Conflict-Verhalten bei abweichendem Inhalt unter gleicher Artifact-Identität
- Schutz vor dem stillschweigenden Überschreiben unvollständiger persistierter Artifacts
- Trennung der konkreten Persistence-Implementierung von der Core-Domain

Die Persistenzarchitektur bleibt unabhängig von konkreten Storage-Technologien.

Relevante Architekturentscheidungen:

- ADR-036 Persistence Boundary and Storage Strategy
- ADR-044 Persistence Provider Interface
- ADR-048 Artifact Registry and Persistence Coordination
- ADR-052 Local Filesystem Persistence Provider
- ADR-055 Filesystem Persistence Layout
- ADR-059 Recording Payload Filesystem Persistence
- ADR-060 Filesystem Store Semantics

---

# Validation

Aktueller dokumentierter Teststand:

- core tests: 40 passed
- recorder tests: 46 passed

Zusätzliche technische Integration wurde erfolgreich manuell ausgeführt:

- Default Audio Input Device erkannt
- Default Input Configuration erkannt: 2 Kanäle, 48000 Hz, F32
- CPAL Input-Stream erfolgreich gestartet
- innerhalb eines Testintervalls 95232 Samples empfangen
- CaptureChunk mit 380928 Payload-Bytes erzeugt
- CaptureTrack und CaptureResult erfolgreich aufgebaut
- vollständiger Recorder Application Flow mit CpalCaptureProvider erfolgreich ausgeführt
- reale Audiodaten aus dem lokalen Default-Input erfasst
- CaptureResult und RecordingArtifact mit den erfassten technischen Daten aufgebaut
- RecordingArtifact mit einem Track erzeugt und durch den bestehenden Persistenzpfad verarbeitet
- Recording-Konfiguration entlang der Capture-to-Artifact-Grenze erhalten

Die Tests validieren unter anderem:

- Lifecycle-Übergänge
- ProductionSession Lifecycle-Invarianten
- API Boundary Operationen für ProductionSession
- Rollen- und Zustandslogik
- Participant- und Participation-Semantik
- Recording-Verknüpfungen
- Application/API Boundary für Production Management
- konsistente Fehlersemantik für Production-Management-Operationen
- Activity History Semantik und Persistenz
- Artifact Lifecycle
- Artifact Registry Verhalten
- Persistence Provider Verhalten
- Filesystem Persistence Verhalten
- Workflow Coordination
- vollständiger Recorder Application Flow
- Recording Artifact Creation and Storage Flow
- Artifact Processing Coordination
- Artifact Recovery aus persistierten Daten
- Capture-to-Artifact Track-/Chunk-Übernahme
- Recording-Payload-Übernahme und Persistenz
- Idempotenz und Konfliktverhalten der Filesystem-Persistenz
- Konsistenzbewertung unvollständiger und inkonsistenter persistierter Artifacts
- Capture-Fehlerpropagation und Verhinderung nachgelagerter Artifact-/Persistence-Verarbeitung bei fehlgeschlagenem Capture

---

# Current Architecture State

NC-PoRe folgt aktuell diesen Architekturprinzipien:

- Production Session als zentrale fachliche Einheit
- Core als Autorität für Geschäftslogik, Lifecycle, Rollen und fachliche Berechtigungen
- technische Details bleiben von der Domäne getrennt
- Recording Artifacts bleiben von Domainobjekten getrennt
- Capture und Storage werden über technische Grenzen abstrahiert
- Artifact Registry und Persistence bleiben getrennte Verantwortlichkeiten
- Application Flow verbindet Workflow, Artifact Processing und Persistence über definierte Grenzen
- Production Management wird über stabile Application/API-Grenzen koordiniert, ohne Domainregeln an Recorder oder Storage zu koppeln
- Participant Identity und sessionbezogene Participation bleiben getrennte fachliche Konzepte
- Rollen sind sessionbezogen und folgen der definierten Owner/Producer/Participant/Guest-Autoritätsrichtung
- Activity History gehört zur ProductionSession und bleibt auf Produktionsaktivitäten bezogen
- Recovery stellt technische Konsistenz zwischen Persistence und Registry her
- CaptureResult und RecordingArtifact besitzen getrennte technische Datenmodelle
- RecordingArtifact strukturiert Tracks und Chunks unabhängig von der physischen Persistenz
- RecordingChunk kann tatsächliche technische Payload-Daten über eine storage-provider-unabhängige Referenz repräsentieren
- Persistenz bleibt austauschbar
- lokale Aufnahme bleibt unabhängig von Netzwerkverfügbarkeit
- Repository-Inhalt ist die technische Quelle der Wahrheit
- Identitäten innerhalb technischer Grenzen werden über explizite Value Objects modelliert
- Filesystem Persistence folgt definierten Store-Semantiken für Recording Artifacts
- die konkrete Capture-Technologie bleibt hinter CaptureProvider verborgen
- der aktuelle CPAL-Pfad bildet einen funktionierenden technischen Capture-to-Artifact-Pfad
- Recording-Konfiguration wird entlang der technischen Capture-to-Artifact-Grenze explizit erhalten
- der fachliche Recording-Lifecycle ist über ProductionSession, Recording, Artifact, Persistence und Recovery konsistent definiert
- Recovery wird für ein konkretes ProductionSession/Recording-Paar orchestriert und wahrt die Domain-Invarianten

---

# Completed Milestones

Die historische Entwicklung wird in einzelnen Milestones dokumentiert:

- `docs/milestones/2026-07-24-architecture-foundation-complete.md`
- `docs/milestones/2026-07-30-first-core-implementation.md`
- `docs/milestones/2026-07-31-recorder-architecture-foundation.md`
- `docs/milestones/2026-08-01-local-recording-persistence-foundation.md`
- `docs/milestones/2026-08-02-artifact-management-foundation.md`
- `docs/milestones/2026-08-07-artifact-recovery-foundation.md`
- `docs/milestones/2026-08-09-recording-artifact-data-boundary-foundation.md`
- `docs/milestones/2026-08-14-local-recording-artifact-persistence-complete.md`
- `docs/milestones/2026-08-15-cpal-capture-integration.md`

Abgeschlossen:

- **Milestone #55 – Local Technical Recording Pipeline**
- **Milestone #64 – Recording Lifecycle Foundation**
- **Milestone #65 – Production Management & Collaboration Foundation**

Milestone #64 umfasst insbesondere den fachlichen Recording-Lifecycle, die Verbindung von Recording und RecordingArtifact, die Application Use Cases sowie die definierten Recovery- und Reconciliation-Semantiken.

Milestone #65 umfasst insbesondere ProductionSession-Management, Participant- und Participation-Semantik, sessionbezogene Rollen und Berechtigungen, stabile Production-Management-Application/API-Grenzen sowie Production Activity/History.

### Architectural Milestone – ADR-068

Mit **ADR-068 – Recording Start and Audio Synchronization Signet** ist am 2026-08-21 eine neue Architekturentscheidung für den nächsten Entwicklungsschritt dokumentiert. ADR-068 befindet sich derzeit im Status **Proposed** und wird deshalb nicht als bereits implementierter Meilenstein dargestellt.

Die Entscheidung definiert für einen konkreten Recording-Start insbesondere:

- einen expliziten gemeinsamen Start durch den Host,
- die Trennung von Session-Mitgliedschaft und Recording-Teilnahme,
- eine zum Startzeitpunkt eingefrorene Recording-Teilnehmermenge,
- einen `READY`-Status erst nach tatsächlich gestarteter lokaler Aufnahme,
- ein gemeinsames Opening Sync Signet als logischen Beginn des Recordings,
- ein gemeinsames Closing Sync Signet als logisches Ende des Recordings,
- die technische Beendigung der lokalen Recorder erst nach dem Closing Signet,
- sowie zwei akustische Synchronisationsanker für die spätere manuelle oder automatisierte Ausrichtung der Audiospuren.

ADR-068 grenzt bewusst spätere automatische DAW-Integration, automatische Spurausrichtung, kontinuierliche Synchronisationskorrektur, Driftmessung und vollständige Recovery-/Re-Join-Verfahren aus. Diese Möglichkeiten bleiben für spätere Versionen offen, werden aber durch ADR-068 nicht vorgezogen.

Die daraus folgende technische Arbeit ist dem bestehenden architektonischen Meilenstein **#66 – Distributed Recording & Synchronisation** zugeordnet. Als erster konkreter Umsetzungsschritt ist **#140 – Define RecordingArtifact synchronization lifecycle invariants** angelegt. Die weiteren Arbeiten sollen auf dem in ADR-068 und #140 etablierten Vertrag aufbauen und nicht unabhängig konkurrierende Synchronisationsmodelle einführen.

---

# Next Steps

Die nächsten Arbeiten werden als größere technische Meilensteine verfolgt. Ein Meilenstein darf mehrere konkrete Issues und PRs umfassen.

1. **`milestone: Distributed Recording & Synchronisation`**

   Aufbau der technischen Grundlage für Offline-first verteilte Aufnahme, Synchronisation und Remote Storage. Dieser Meilenstein baut auf der abgeschlossenen lokalen Recording-Pipeline, den abgeschlossenen fachlichen Recording-Lifecycle-Grenzen und der Production-Management-Foundation auf.

   Der aktuelle architektonische Einstiegspunkt ist **ADR-068 – Recording Start and Audio Synchronization Signet**. Die Umsetzung soll zunächst den dort definierten gemeinsamen Recording-Start, die Recording-Teilnehmersemantik, die Ready-Bestätigungen und die beiden Synchronisationssignets in sinnvolle, zusammenhängende technische Arbeitspakete überführen.

   Parallel bzw. anschließend ist der in #140 definierte artifact-level Synchronisationslebenszyklus die Grundlage für die weitere Synchronisationsarchitektur. Konkrete Remote-Storage-, Transport-, Retry-, Idempotenz- und Recovery-Arbeiten werden darauf aufbauend bestimmt.

Die Meilensteine sind als übergeordnete Wegpunkte zu verstehen und werden jeweils in konkrete Umsetzungsschritte zerlegt. Die konkrete Reihenfolge und die Abhängigkeiten werden vor Beginn der jeweiligen Implementierung geprüft.

---

# Relevant Documentation

Wichtige Einstiegspunkte:

- `docs/architecture/`
- `docs/implementation/`
- `docs/project/`
- `docs/milestones/`
- `docs/architecture/adr-index.md`

---

# English Version (Deutsche Version oben)

The current project status mirrors the German section above.

Completed milestones include:

- **Milestone #55 – Local Technical Recording Pipeline**
- **Milestone #64 – Recording Lifecycle Foundation**
- **Milestone #65 – Production Management & Collaboration Foundation**

Milestone #64 establishes the domain-level Recording lifecycle across ProductionSession, Recording, RecordingArtifact, persistence and recovery, including application use cases and deterministic reconciliation semantics.

Milestone #65 establishes the production-management and collaboration foundation across ProductionSession management, participant and participation semantics, session-scoped roles and permissions, stable production-management application/API boundaries, and production activity/history.

### Architectural Milestone – ADR-068

**ADR-068 – Recording Start and Audio Synchronization Signet** was documented on 2026-08-21 as the architectural entry point for the next development direction. ADR-068 is currently **Proposed** and is therefore not presented as an already implemented milestone.

It defines the explicit host-controlled recording start, the separation of session membership and recording participation, frozen recording participant sets, `READY` confirmations after actual local capture start, and Opening/Closing Sync Signets as logical recording boundaries. It deliberately leaves automatic DAW integration, automatic alignment, continuous synchronization correction, drift measurement, and complete recovery/re-join behavior for later versions.

The resulting technical work belongs to **#66 – Distributed Recording & Synchronisation**. The first concrete work package is **#140 – Define RecordingArtifact synchronization lifecycle invariants**. Further work should build on these contracts rather than introducing competing synchronization models.

The next major milestone is:

1. **`milestone: Distributed Recording & Synchronisation`** — establish the offline-first foundation for distributed recording, synchronization and remote storage, beginning from the architectural direction established by ADR-068 and the artifact-level synchronization contract in #140.

Each milestone may contain multiple implementation issues and pull requests. Dependencies and implementation order are reviewed before work begins.
