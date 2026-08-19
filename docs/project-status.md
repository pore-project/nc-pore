# NC-PoRe Project Status

- Version: 3.1
- Date: 2026-08-19

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
- Recording Modell
- Recording Lifecycle
- Participant Modell
- Participation Modell
- Activity History Grundstruktur
- API Operationen für ProductionSession Lifecycle
- API Operationen für ProductionSession-Verwaltung und Recording-Verknüpfung
- Read-API-Operationen für Participants, Recordings und Activity History

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

Die Persistenzarchitektur bleibt unabhängig von konkreten Storage-Technologien.

Relevante Architekturentscheidungen:

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
- API Boundary Operationen für ProductionSession
- Rollen- und Zustandslogik
- Recording-Verknüpfungen
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
- Core als Autorität für Geschäftslogik
- technische Details bleiben von der Domäne getrennt
- Recording Artifacts bleiben von Domainobjekten getrennt
- Capture und Storage werden über technische Grenzen abstrahiert
- Artifact Registry und Persistence bleiben getrennte Verantwortlichkeiten
- Application Flow verbindet Workflow, Artifact Processing und Persistence über definierte Grenzen
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

Milestone #64 umfasst insbesondere den fachlichen Recording-Lifecycle, die Verbindung von Recording und RecordingArtifact, die Application Use Cases sowie die definierten Recovery- und Reconciliation-Semantiken.

---

# Next Steps

Die nächsten Arbeiten werden als größere technische Meilensteine verfolgt. Ein Meilenstein darf mehrere konkrete Issues und PRs umfassen.

1. **`milestone: Production Management & Collaboration Foundation`**

   Weiterentwicklung der ProductionSession- und Recording-Welt zu einer belastbaren fachlichen Management- und Kollaborationsgrundlage, auf der spätere Clients und weitere Schnittstellen aufbauen können.

2. **`milestone: Distributed Recording & Synchronisation`**

   Aufbau der technischen Grundlage für Offline-first verteilte Aufnahme, Synchronisation und Remote Storage. Dieser Meilenstein baut auf der abgeschlossenen lokalen Recording-Pipeline und den abgeschlossenen fachlichen Recording-Lifecycle-Grenzen auf.

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

Milestone #64 establishes the domain-level Recording lifecycle across ProductionSession, Recording, RecordingArtifact, persistence and recovery, including application use cases and deterministic reconciliation semantics.

The next major milestones are:

1. **`milestone: Production Management & Collaboration Foundation`** — establish a robust production-management and collaboration foundation for future clients and interfaces.
2. **`milestone: Distributed Recording & Synchronisation`** — establish the offline-first foundation for distributed recording, synchronization and remote storage.

Each milestone may contain multiple implementation issues and pull requests. Dependencies and implementation order are reviewed before work begins.
