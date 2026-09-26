# NC-PoRE Project Status

- Version: 3.6
- Date: 2026-09-26

---

# Deutsch (English version below)

---

# Projektphase

## Späte Alpha-Phase / V1-Härtung und Produktisierung

NC-PoRE befindet sich nach der wesentlichen technischen V1-Integrationsarbeit in einem **späten Alpha-Stand**.

Der aktuelle Entwicklungsstand `0.1.0-dev.98` ist CI-verifiziert und bildet die derzeit weitgehend zusammenhängende V1-Basis. Er ist ausdrücklich **noch keine Beta- bzw. App-Store-Freigabe**.

Die fachliche Grundlage, die lokale Aufnahme, die Talk-Integration, die Recording-Koordination, der Artefakt-Transport und die grundlegende Talk-nahe Oberfläche sind vorhanden. Vor einer Beta-Freigabe stehen insbesondere reale Runtime-Validierung, Produktpolitur, Packaging-Bereinigung, Dokumentationsbereinigung und weitere Härtung aus.

---

# Aktueller Implementierungsstand

## Core / Application

Implementiert bzw. verifiziert:

- Production-/Session-Modell und Lifecycle
- Rollen und fachliche Berechtigungen
- Recording-Lifecycle
- Recording-Teilnehmermenge und READY-Barriere
- Opening-Koordination und Opening-Bestätigungen
- Stop-/Stop-Acknowledgement-Lifecycle
- Recording- und Production-Abschlusssemantik
- Late Artifact Delivery ohne erneutes Öffnen der Production
- Application-/Core-Grenzen für browser- und hostseitige Clients

Core bleibt die autoritative Quelle der fachlichen Recording- und Production-Zustände.

## Recorder und lokale Aufnahme

Implementiert:

- generische Capture-Grenze zwischen Recording-Workflow und konkreten CaptureProvidern
- lokale Audioaufnahme
- RecordingArtifact-Erzeugung
- lokale Preservation und Recovery
- Artefakt-Identität und Persistenzgrenzen
- Capture-/Artifact-Lifecycle
- technische Quellenwechselbehandlung
- Übergang von lokaler Preservation in die weitere Verarbeitung

Die lokale Aufnahme bleibt vom Netzwerk unabhängig.

## Nextcloud Talk V1

Im aktuellen `develop`-Stand vorhanden:

- Talk-spezifischer Audio-Capture-Connector
- unabhängige PoRE-Capture-Quelle gegenüber dem Kommunikationspfad
- Talk-Kontext- und Teilnehmerauflösung
- Materialisierung der Production vor dem Recording
- eingefrorene Recording-Teilnehmermenge für den Start
- PoRE-eigene Recording Coordination außerhalb von Talk-Signaling
- kurzer SSE-basierter Coordination-Transport
- authoritative Core-Commands für Begin, Ready, Opening, Stop und Completion
- explizite Host-Steuerung **Aufnahme starten** / **Aufnahme beenden**
- READY-/Opening-Darstellung
- verstrichene Aufnahmezeit
- Production-Status und abschließende Host-Aktion
- dauerhafter Browser-Artefaktstand vor der Remote-Übergabe
- verifizierter Nextcloud-Artefakttransport

## UI / Produktoberfläche

Die Talk-nahe PoRE-Oberfläche ist funktional vorhanden und folgt der in ADR-086 festgelegten Informationshierarchie.

Sie ist weiterhin als **Alpha-Oberfläche** zu betrachten. Insbesondere visuelle Integration, Statuskonsistenz, Fehlerdarstellung und allgemeine Produktpolitur müssen vor einer Beta-/App-Store-Freigabe weiter gehärtet werden.

---

# Validierung

Aktuell verifiziert:

- dev.98: V1 Talk Connector JavaScript erfolgreich
- dev.98: V1 Talk Connector PHP erfolgreich
- dev.98: V1 Talk Connector Rust erfolgreich
- dev.98: allgemeiner CI-Check erfolgreich
- dev.98: 49 JavaScript-Tests bestanden
- dev.98: relevante Rust-Tests erfolgreich
- TEST-04: Host-Eigen-READY führt über authoritative State zu `trigger_opening` und `confirm_opening`
- dev.93 → dev.98: keine unbeabsichtigte Änderung der übrigen Coordination-/Audio-/Server-Architektur festgestellt

Noch **nicht** als erledigt verbucht:

- realer End-to-End-Test mit laufender Nextcloud-/Talk-Instanz auf dem aktuellen dev.98-Stand;
- systematische Browser-/Talk-Laufzeitvalidierung über Firefox, Chromium und Safari/WebKit;
- vollständige Release-Package-Bereinigung;
- Beta-/App-Store-Härtung.

Der reale Mehrpersonen-Test bleibt damit eine Integrationsprüfung und wird nicht durch CI ersetzt.

---

# Architekturzustand

Die aktuelle V1-Architektur folgt insbesondere diesen Prinzipien:

- Core ist autoritativ für fachliche Recording- und Production-Zustände.
- Talk ist Host-/Kommunikationsumgebung, nicht die PoRE-Aufnahmepipeline.
- Host-spezifische Media-Logik bleibt im Connector.
- Kommunikations- und Aufnahmepipeline bleiben getrennt.
- Recording Coordination ist PoRE-eigen und nicht an Talk-Signaling gebunden.
- Signaling transportiert Ereignisse; es ist keine zweite fachliche State-Quelle.
- Lokales Capture ist von Netzwerkverfügbarkeit getrennt.
- Recording, Artifact und Production besitzen getrennte Abschlusssemantiken.
- Ein spät geliefertes Artifact kann ein Recording vervollständigen, darf aber eine bereits geschlossene Production nicht wieder öffnen.
- Nextcloud ist in V1 der produktive Remote-Provider und die autoritative Artefaktablage.
- Remote Completion wird erst nach der vorgesehenen Integritätsprüfung bestätigt.
- Provider-/Host-spezifische Details bleiben außerhalb des host-neutralen PoRE-Kerns.

---

# Abgeschlossene bzw. eingegliederte Arbeiten

- Milestone #55 – Local Technical Recording Pipeline
- Milestone #64 – Recording Lifecycle Foundation
- Milestone #65 – Production Management & Collaboration Foundation
- PR #282 – PoRE-owned, host-neutral recording coordination
- konfigurierbarer Nextcloud-Artefaktpfad und ownergebundene Storage-Zielauflösung
- V1 Production Materialization
- Recording Artifact Aggregation und Production Completion Semantics
- aktuelle V1 Talk Recording UI
- native Capture-Selection und der weitere host-neutrale Recorder-Ausbau in PR #228 / #232 bleiben separate Entwicklungsschritte
---

# Nächste Arbeiten

Die nächsten Schritte sind bewusst in zwei Ebenen getrennt.

## Produkt- und Integrationshärtung

- realer Mehrpersonen-End-to-End-Test auf einer aktuellen Nextcloud-/Talk-Instanz
- gezielte Firefox-/Chromium-/Safari-Validierung
- UI-Politur und konsistente Zustandsdarstellung
- Fehler- und Recovery-Fälle im realen Lauf
- Release-Package so bereinigen, dass Testdateien nicht unnötig in das Produktionsartefakt gelangen
- die separaten Native-Capture-/host-neutralen Recorder-Arbeiten aus PR #228 / #232 weiterführen
- Beta-/App-Store-Releasekriterien definieren und verifizieren

## Repository- und Dokumentationsbereinigung

- offene Issues gegen den aktuellen `develop`-Stand prüfen und erledigte historische Container schließen
- ADR-Nummerierung und Cross-References konsistent halten
- `project-status.md`, README und ADR-Index synchron halten
- veraltete oder doppelte PR-/Branch-Artefakte bereinigen
- verbleibende Architekturentscheidungen gegen die tatsächliche Implementierung abgleichen

---

# Dokumentations-Einstiegspunkte

- `docs/architecture/`
- `docs/ui/`
- `docs/implementation/`
- `docs/project/`
- `docs/milestones/`
- `docs/architecture/adr-index.md`
- `docs/v1/IMPLEMENTATION-NOTE.md`
- `adr/`

---

# English Version (Deutsche Version above)

---

# Project Phase

## Late Alpha / V1 Hardening and Productization

NC-PoRE is now in a **late-alpha development state** after the main V1 technical integration work.

The current development level `0.1.0-dev.98` is CI-verified and forms the current broadly integrated V1 foundation. It is explicitly **not yet a Beta or App-Store release**.

The fachlich foundation, local capture, Talk integration, recording coordination, artifact transport and the basic Talk-like product surface are present. Real runtime validation, product polish, packaging cleanup, documentation reconciliation and further hardening remain before Beta release.

---

# Current Implementation Status

## Core / Application

Implemented and verified:

- Production/session model and lifecycle
- roles and fachliche authorization
- recording lifecycle
- recording participant set and READY barrier
- Opening coordination and Opening confirmations
- stop and stop-acknowledgement lifecycle
- Recording and Production completion semantics
- late Artifact Delivery without reopening a Production
- browser- and host-facing Application/Core boundaries

Core remains authoritative for fachliche Recording and Production state.

## Recorder and Local Capture

Implemented:

- host-neutral capture boundary
- local audio recording
- RecordingArtifact creation
- local preservation and recovery
- artifact identity and persistence boundaries
- native capture selection
- capture/artifact lifecycle
- technical source-change handling
- continuation from local preservation into further processing

Local capture remains independent of network availability.

## Nextcloud Talk V1

The current `develop` line contains:

- Talk-specific audio capture connector
- independent PoRE capture source separate from the communication path
- Talk context and participant resolution
- Production materialization before recording
- fixed recording participant set for the start
- PoRE-owned Recording Coordination outside Talk signaling
- short-lived SSE-based coordination transport
- authoritative Core commands for Begin, Ready, Opening, Stop and Completion
- explicit Host **Start recording** / **Stop recording** controls
- READY/Opening presentation
- elapsed recording time
- Production status and final Host action
- durable browser artifact before remote handoff
- verified Nextcloud artifact transport

## UI / Product Surface

The Talk-like PoRE surface is functionally present and follows the information hierarchy defined by ADR-086.

It is still an **Alpha surface**. Visual integration, state consistency, error presentation and general product polish require further work before a Beta/App-Store release.

---

# Validation

Currently verified:

- dev.98 V1 Talk Connector JavaScript successful
- dev.98 V1 Talk Connector PHP successful
- dev.98 V1 Talk Connector Rust successful
- dev.98 general CI successful
- dev.98 JavaScript suite: 49 passed
- relevant Rust tests successful
- TEST-04: Host self-READY advances from authoritative state to `trigger_opening` and `confirm_opening`
- dev.93 → dev.98 review found no unintended changes to the remaining coordination/audio/server architecture

Not yet recorded as complete:

- real end-to-end test against a running Nextcloud/Talk instance on current dev.98;
- systematic Firefox/Chromium/Safari-WebKit runtime validation;
- complete production package cleanup;
- Beta/App-Store hardening.

The real multi-participant test therefore remains an integration check and is not replaced by CI.

---

# Architecture State

The current V1 architecture follows these principles in particular:

- Core is authoritative for fachliche Recording and Production state.
- Talk is the host/communication environment, not the PoRE recording pipeline.
- Host-specific media logic remains in connectors.
- Communication and recording pipelines remain separate.
- Recording Coordination is PoRE-owned and not coupled to Talk signaling.
- Signaling carries events; it is not a second fachliche state source.
- Local capture is independent of network availability.
- Recording, Artifact and Production have separate completion semantics.
- Late artifact delivery may complete a Recording but never reopens a completed Production.
- Nextcloud is the V1 productive remote provider and authoritative artifact store.
- Remote completion is confirmed only after the defined integrity verification.
- Provider- and host-specific details remain outside the host-neutral PoRE core.

---

# Completed or Integrated Work

- Milestone #55 – Local Technical Recording Pipeline
- Milestone #64 – Recording Lifecycle Foundation
- Milestone #65 – Production Management & Collaboration Foundation
- PR #282 – PoRE-owned, host-neutral recording coordination
- configurable Nextcloud artifact path and Production-owner storage resolution
- V1 Production Materialization
- Recording Artifact Aggregation and Production Completion Semantics
- current V1 Talk Recording UI

---

# Next Work

The next work is deliberately separated into two levels.

## Product and Integration Hardening

- real multi-participant end-to-end test on a current Nextcloud/Talk instance
- targeted Firefox/Chromium/Safari validation
- UI polish and consistent state presentation
- real-world error and recovery cases
- clean the release package so test files are not unnecessarily shipped in the production artifact
- continue the separate native-capture / host-neutral recorder work in PR #228 / #232
- define and verify Beta/App-Store release criteria

## Repository and Documentation Cleanup

- review open issues against current `develop` and close completed historical containers
- keep ADR numbering and cross-references consistent
- keep `project-status.md`, README and ADR index synchronized
- clean remaining obsolete PR and branch artifacts
- reconcile remaining architecture decisions with the actual implementation

---

# Documentation Entry Points

- `docs/architecture/`
- `docs/ui/`
- `docs/implementation/`
- `docs/project/`
- `docs/milestones/`
- `docs/architecture/adr-index.md`
- `docs/v1/IMPLEMENTATION-NOTE.md`
- `adr/`
