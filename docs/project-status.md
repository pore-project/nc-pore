# NC-PoRe Project Status

- Version: 4.0
- Date: 2026-09-11

---

# Deutsch (English version below)

## Projektphase

NC-PoRe befindet sich in der technischen V1-Umsetzung und -Härtung.

Die grundlegenden Core-, Recorder- und Persistence-Grenzen sind implementiert und durch Tests abgesichert. Die browserbasierte Host-Integration befindet sich in der technischen Validierung.

## Core

Implementiert sind unter anderem:

- ProductionSession und Lifecycle
- Recording- und Participation-Modelle
- Participant Identity und sessionbezogene Participation
- Rollen- und Berechtigungssemantik
- fachliche Lifecycle- und Participation-Invarianten
- Activity History
- Application- und Read-APIs für die bestehenden fachlichen Bereiche

Der Core bleibt die fachliche Autorität. Technische Provider- und Host-Details liegen außerhalb des Domain-Core.

## Recorder

Implementiert sind unter anderem:

- Recording Lifecycle
- Capture Boundary
- Workflow Coordination
- RecordingArtifact mit Tracks und Chunks
- CaptureResult- und Artifact-Datenmodell
- Artifact Registry und Processing
- lokale Persistenz und Recovery
- explizite technische Identitätstypen
- CaptureProvider-Abstraktion
- CPAL-basierte lokale Capture-Implementierung
- native Capture-Selection entlang der bestehenden Audio-Konfiguration

Die lokale Recording-Pipeline ist damit als zusammenhängender Capture-to-Artifact-Pfad vorhanden.

## Nextcloud Talk V1 Integration

Die erste Host-Integration ist Nextcloud Talk.

Der aktuelle browserseitige Integrationsstand umfasst insbesondere:

- Talk-spezifische Ermittlung der lokalen Audioquelle
- getrennte PoRE-Capture-Quelle für die Aufnahme
- explizite Steuerung des Recording-Lifecycles
- sichtbaren Recording-Status
- Behandlung von Änderungen der lokalen Audioquelle
- generische browserseitige Recording-Grenze ohne Talk-Abhängigkeit im Recorder-Kern

Die Talk-Kommunikationspipeline und die PoRE-Aufnahmepipeline bleiben getrennt. Talk ist nicht die fachliche Quelle des PoRE-Recording-Artifacts.

Die allgemeine Host-Connector-Architektur ist in **ADR-076** dokumentiert. Die Qualitätsgrenze zwischen Kommunikation und Aufnahme ist in **ADR-074** dokumentiert; die Unabhängigkeit der lokalen Aufnahme ist in **ADR-075** festgehalten.

## Persistence und Transfer

Die technischen Grenzen für lokale Persistenz, Artifact-Verarbeitung, Synchronisation und kontrollierten Transfer sind vorhanden.

Für V1 ist Nextcloud die konkrete Host- und Speicherintegration. Die Übergabe fertiger Browser-Artefakte wird über die bestehende Application-/Nextcloud-Grenze geführt und muss vor dem erfolgreichen Abschluss eindeutig verifiziert werden.

## Validation

Der dokumentierte technische Stand umfasst insbesondere:

- Core- und Recorder-Tests
- Persistence-Integrationstests
- automatisierte Prüfungen der Browser-Skripte
- lokale Audioaufnahme bis zum RecordingArtifact
- technische Validierung der Talk-Capture-Grenze
- manuelle Validierung des browser- und hostbezogenen Media-Lifecycles

Automatisierte Tests und reale Browser-/Talk-Validierung werden als unterschiedliche Prüfebenen behandelt.

## Aktuelle Architekturprinzipien

- Production Session als zentrale fachliche Einheit
- Core als Autorität für fachliche Regeln und Lifecycle
- lokale Aufnahme unabhängig von der Kommunikationspipeline
- Capture und Storage über klare technische Grenzen
- RecordingArtifact getrennt von Domainobjekten
- Persistenz und Synchronisation als getrennte Verantwortlichkeiten
- Host-spezifische Logik in Connectors und Integrationsschichten
- keine provider-spezifischen Media-Interna im Core
- technische Identitäten bleiben explizit getrennt
- bestehende Architektur wird erweitert statt parallel dupliziert

## Aktueller Arbeitsstand

Die weitere V1-Arbeit konzentriert sich auf die vollständige Verbindung der bereits vorhandenen Grenzen zu einem reproduzierbaren End-to-End-Ablauf und auf die dafür erforderliche reale Validierung.

Konkrete Architekturentscheidungen werden in den ADRs dokumentiert; der aktuelle Entwicklungsstand ergibt sich aus dem Repository und den zugehörigen Tests.

## Historische Meilensteine

Die abgeschlossenen Entwicklungsschritte bleiben in `docs/milestones/` als historische Dokumentation erhalten.

## Relevante Dokumentation

- `adr/` — Architecture Decision Records
- `docs/project/` — Projektziele, MVP und aktuelle Roadmap
- `docs/implementation/` — Implementierungsdokumentation
- `docs/milestones/` — historische Meilensteine
- `docs/v1/` — V1-spezifische Hinweise und Prüfungen

---

# English Version

## Project Phase

NC-PoRe is in technical V1 implementation and hardening.

The fundamental Core, Recorder and persistence boundaries are implemented and covered by tests. The browser-based host integration is under technical validation.

## Core

Implemented areas include:

- ProductionSession and lifecycle
- recording and participation models
- participant identity and session-specific participation
- role and permission semantics
- domain lifecycle and participation invariants
- activity history
- application and read APIs for the existing domain areas

The Core remains the domain authority. Technical provider and host details remain outside the domain Core.

## Recorder

Implemented areas include:

- recording lifecycle
- capture boundary
- workflow coordination
- RecordingArtifact with tracks and chunks
- CaptureResult and artifact data models
- artifact registry and processing
- local persistence and recovery
- explicit technical identity types
- CaptureProvider abstraction
- CPAL-based local capture implementation
- native capture selection along the established audio configuration

The local recording pipeline therefore exists as a coherent capture-to-artifact path.

## Nextcloud Talk V1 Integration

The first host integration is Nextcloud Talk.

The current browser integration includes in particular:

- Talk-specific discovery of the local audio source
- a separate PoRE capture source for recording
- explicit recording lifecycle control
- visible recording state
- handling of local audio-source changes
- a generic browser recording boundary without Talk dependencies in the recorder core

The Talk communication pipeline and PoRE recording pipeline remain separate. Talk is not the domain source of the PoRE RecordingArtifact.

The general host connector architecture is documented in **ADR-076**. The communication/recording quality boundary is documented in **ADR-074**, and local capture independence in **ADR-075**.

## Persistence and Transfer

The technical boundaries for local persistence, artifact processing, synchronization and controlled transfer are in place.

For V1, Nextcloud is the concrete host and storage integration. Finalized browser artifacts are handed off through the existing Application/Nextcloud boundary and must be explicitly verified before successful completion is reported.

## Validation

The documented technical state includes in particular:

- Core and Recorder tests
- persistence integration tests
- automated browser-script checks
- local audio capture through RecordingArtifact
- technical validation of the Talk capture boundary
- manual validation of browser- and host-specific media lifecycle behavior

Automated tests and real browser/Talk validation are treated as separate validation levels.

## Current Architectural Principles

- Production Session as the central domain entity
- Core as authority for domain rules and lifecycle
- local recording independent of the communication pipeline
- clear technical boundaries around capture and storage
- RecordingArtifact separated from domain objects
- persistence and synchronization as separate responsibilities
- host-specific logic in connectors and integration layers
- no provider-specific media internals in Core
- explicit separation of technical identities
- extend existing architecture rather than creating parallel implementations

## Current Work State

Further V1 work focuses on connecting the existing boundaries into a reproducible end-to-end workflow and on the real-world validation required for that workflow.

Concrete architectural decisions are documented in the ADRs; the current development state is represented by the repository and its tests.

## Historical Milestones

Completed development steps remain available as historical documentation under `docs/milestones/`.

## Relevant Documentation

- `adr/` — Architecture Decision Records
- `docs/project/` — project goals, MVP and current roadmap
- `docs/implementation/` — implementation documentation
- `docs/milestones/` — historical milestones
- `docs/v1/` — V1-specific notes and checks
