# NC-PoRe Project Status

- Version: 3.5
- Date: 2026-09-01

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
- sessionbezogene Rollen- und Berechtigungssemantik für Owner, Producer, Participant und Guest
- API Operationen für ProductionSession Lifecycle und Verwaltung
- Recording-Verknüpfungen
- Read-API-Operationen für Participants, Recordings und Activity History
- konsistente Domain-/Application-Fehlersemantik
- fachliche Validierung von Lifecycle-, Rollen- und Participation-Invarianten
- Production Activity/History-Semantik

Die Production-Management- und Collaboration-Foundation ist damit als zusammenhängende fachliche Grundlage für spätere Clients und Kollaborationsfunktionen implementiert und validiert.

---

## Recorder

Implementiert:

- Session Modul und Statusmodell
- Recording Lifecycle
- Capture Boundary Interface
- Workflow Coordination Layer
- Recording Artifact Model
- Recording Artifact Track and Chunk Model
- Capture Result Track and Chunk Model
- Capture-to-Artifact Data Boundary
- Artifact Lifecycle Management
- Local Artifact Registry
- Artifact Processing Boundary
- Recorder Application Boundary
- Local Recording Artifact Flow
- Artifact Recovery Boundary
- RecordingSessionId, ArtifactId und weitere explizite technische Identitätstypen
- storage-provider-unabhängige Payload-Referenzen
- Filesystem Persistence Provider einschließlich Recovery und Konsistenzbewertung
- CPAL-basierte konkrete CaptureProvider-Implementierung
- native Capture-Fähigkeitsermittlung und Auswahl über ADR-061
- native PCM16-, PCM24- und F32-Auswahl ohne Resampling oder Bit-Tiefen-Erweiterung
- erfolgreicher lokaler CPAL Capture-to-Artifact-Pfad
- technische Recording-Konfiguration entlang der Capture-to-Artifact-Grenze
- produktionsgeeignete Stream-Fehlerpropagation
- Lifecycle- und Fehlerbehandlung für fehlgeschlagenes Capture

Die lokale technische Recording-Pipeline ist damit als zusammenhängender Capture-to-Artifact-Pfad implementiert und validiert. Die native Capture-Selection ist in PR #228 als separater, backend-unabhängiger Entwicklungsschritt abgeschlossen vorbereitet; der PR wird nach vollständig grünem CI und Review abgeschlossen.

---

## Nextcloud Talk V1 Integration

Der erste Browser-Host-Pfad ist als technischer V1-Schnitt implementiert:

- Talk-spezifischer Connector zur lokalen Audio-Quelle
- unabhängige PoRE-Capture-Quelle getrennt vom Talk-Kommunikationspfad
- PoRE-Capture mit deaktivierter Kommunikationsverarbeitung, soweit der Browser diese Constraints unterstützt
- Talk dient nur zur Erkennung der aktuell ausgewählten Mikrofonquelle und eines Quellenwechsels
- Mikrofonwechsel wird als Recording-Grenze behandelt und nicht stillschweigend in einer Aufnahme fortgeführt
- generischer browserseitiger Recording Controller ohne Talk-Abhängigkeit
- explizite **Aufnahme starten** / **Aufnahme beenden** Steuerung
- sichtbarer Recording-Status
- Anzeige des tatsächlich gelieferten lokalen Mikrofonnamens und, soweit vom Browser verfügbar, Sample Rate, Sample Size und Kanalzahl
- Fehlerzustand bei nicht möglicher unabhängiger PoRE-Capture-Quelle
- keine Kopplung des Recording-Stopps an das Ende des Talk-Raums

Der Browserpfad verwendet in dieser V1-Stufe `MediaRecorder` und erzeugt zunächst ein Browser-Artifact. Dieses ist noch kein persistiertes `RecordingArtifact`; die explizite Übergabe in den bestehenden PoRE Artifact-/Persistence-Lifecycle ist als nächste Integrationsgrenze definiert.

Relevante Architekturentscheidungen:

- ADR-062 Browser-First Guest Participation
- ADR-068 Recording Start and Audio Synchronization Signet
- ADR-071 Recording Capture, Preservation and Transport Formats
- ADR-072 Host-Integrated Local Audio Capture via Connector
- ADR-075 Local Capture Independence from Communication Pipeline

---

## Persistence

Implementiert:

- Local Recording Persistence Boundary
- Persistence Provider Interface
- In-Memory Persistence Provider
- Filesystem Persistence Provider
- Persistence Integration Tests
- Persistenz des tatsächlichen Recording-Payloads einschließlich temporärer Veröffentlichung und vollständiger Artifact-Verzeichnisse
- definierte Store-Semantik
- Idempotenz für äquivalente persistierte Artifacts
- Conflict-Verhalten bei abweichendem Inhalt unter gleicher Artifact-Identität
- Schutz vor dem stillschweigenden Überschreiben unvollständiger persistierter Artifacts
- Trennung der konkreten Persistence-Implementierung von der Core-Domain

---

# Validation

Dokumentierter lokaler technischer Teststand:

- core tests: 40 passed
- recorder tests: 46 passed
- CPAL Default Audio Input Device erfolgreich erkannt
- reale lokale Audiodaten erfolgreich bis zum RecordingArtifact erfasst
- native Capture-Selection in der CPAL-Integration ausgeführt
- V1 Talk Browser-Skripte bestehen den JavaScript-Syntax-Check in GitHub Actions
- V1 Talk Rust Workspace Check, Workspace Tests und Rustfmt-Check sind in GitHub Actions grün

Die Talk-Browser-Unit-Spezifikationen dokumentieren den Connector-Lifecycle und die unabhängige Capture-Quelle; die CI-Stufe führt derzeit den deterministischen Syntax-Check der Browser-Skripte aus. Reale Browser-/Talk-Runtime-Validierung bleibt ein separater manueller Validierungsschritt.

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
- Participant Identity und sessionbezogene Participation bleiben getrennte fachliche Konzepte
- Activity History gehört zur ProductionSession und bleibt auf Produktionsaktivitäten bezogen
- Recovery stellt technische Konsistenz zwischen Persistence und Registry her
- CaptureResult und RecordingArtifact besitzen getrennte technische Datenmodelle
- RecordingArtifact strukturiert Tracks und Chunks unabhängig von der physischen Persistenz
- Persistenz bleibt austauschbar
- lokale Aufnahme bleibt unabhängig von Netzwerkverfügbarkeit
- Repository-Inhalt ist die technische Quelle der Wahrheit
- Filesystem Persistence folgt definierten Store-Semantiken
- die konkrete Capture-Technologie bleibt hinter CaptureProvider verborgen
- native Capture-Selection ist backend-unabhängig und verhindert künstliche Qualitätsversprechen
- Kommunikationspipeline und PoRE-Aufnahmepipeline sind getrennt
- Host-spezifische Talk-Logik bleibt im Connector
- Talk ist nicht die Aufnahmepipeline und nicht das Recording-Masterformat

---

# Completed Milestones

Die historische Entwicklung wird in einzelnen Milestones dokumentiert. Zusätzlich zum bestehenden lokalen Recording-Fortschritt ist die technische Grundlage für die erste Talk-Browser-Integration vorhanden.

Abgeschlossen:

- **Milestone #55 – Local Technical Recording Pipeline**
- **Milestone #64 – Recording Lifecycle Foundation**
- **Milestone #65 – Production Management & Collaboration Foundation**
- **ADR-068 – Recording Start and Audio Synchronization Signet** (Accepted)

---

# Next Steps

Die nächsten Arbeiten werden als größere technische Meilensteine verfolgt.

1. **Talk V1 Artifact Boundary**

   Browser-Recording-Artifact explizit an die bestehende PoRE `RecordingArtifact`-/Persistence-Grenze anbinden, ohne einen zweiten Artifact-Lifecycle einzuführen.

2. **Distributed Recording & Synchronisation**

   Aufbau der technischen Grundlage für Offline-first verteilte Aufnahme, Synchronisation und Remote Storage auf Basis der abgeschlossenen lokalen Recording-Pipeline und ADR-068.

3. **Talk Recording Lifecycle**

   ADR-068 mit dem tatsächlichen Host-/Browser-Lifecycle verbinden: eingefrorene Recording-Teilnehmer, READY nach realem Capture-Start sowie Opening-/Closing-Sync-Signet.

4. **Talk UI / recording information**

   Die V1-Oberfläche schrittweise in die produktive Talk-UI integrieren und dabei die bereits definierte Informationshierarchie beibehalten: Recording-Zustand und aktive lokale Quelle zuerst; technische Details sekundär.

5. **Runtime validation**

   Reale Validierung des Talk-Pfades mit Firefox, Chromium und Safari/WebKit sowie unterschiedlichen lokalen Audioquellen.

---

# Relevant Documentation

Wichtige Einstiegspunkte:

- `docs/architecture/`
- `docs/implementation/`
- `docs/project/`
- `docs/milestones/`
- `docs/architecture/adr-index.md`
- `docs/v1/IMPLEMENTATION-NOTE.md`

---

# English Version (Deutsche Version oben)

The current project status mirrors the German section above.

NC-PoRe is in active technical implementation. The local Core/Recorder/Persistence foundation is implemented and validated. The first Nextcloud Talk browser integration now has a separate PoRE capture source, explicit recording controls, visible recording state, source-change handling, and a generic browser recording boundary.

The Talk communication track is not the PoRE recording master. The Talk connector observes the selected local microphone only to establish the PoRE capture source and detect source replacement. The browser V1 path currently produces a browser recording artifact through `MediaRecorder`; integration with the authoritative PoRE `RecordingArtifact`/persistence lifecycle remains the next explicit boundary.

The next major work packages are the Talk artifact boundary, reconciliation with the distributed recording protocol, production Talk UI integration, and real browser/runtime validation.
