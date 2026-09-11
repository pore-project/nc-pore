# NC-PoRe Implementation Plan

## Deutsche Version

---

# Zweck

Dieses Dokument beschreibt den aktuellen Weg zur ersten öffentlich nutzbaren NC-PoRe-V1.

ADRs beantworten, **warum** eine Architekturentscheidung getroffen wurde. Dieser Plan beschreibt, **welche Arbeiten für den aktuellen V1-Stand noch erforderlich sind**.

---

# Aktueller Stand

Die technische Basis umfasst insbesondere:

- Core-Domain und Lifecycle-Modelle
- Recorder-Capture-Boundary und Workflow-Koordination
- `RecordingArtifact`-Repräsentation
- lokale Persistenz und Recovery
- Artifact Registry und Processing
- Synchronisations- und Transfergrenzen
- Browser-Artefaktpersistenz und Wiederherstellung
- Nextcloud-Integration über die bestehende Application-/OCS-/Files-API-Grenze
- Integritätsprüfung und idempotente Übergabe

NC-PoRe befindet sich in der **V1-Härtung und Produktisierung**.

---

# V1-Arbeitsplan

## V1-1 Repository- und Dokumentationskonsolidierung

Code, Branch-Zustand und Dokumentation müssen denselben Projektstand beschreiben. Historische Informationen werden nachvollziehbar konsolidiert, ohne die Git-Historie umzuschreiben.

## V1-2 Legacy- und Dead-Code-Audit

Zu untersuchen sind insbesondere:

- historische oder unreferenzierte Module
- Kompatibilitäts- und Übergangsexporte
- Feasibility-/Proof-of-Concept-Code
- scheinbar unreferenzierte öffentliche APIs

Nichts wird allein wegen seines Alters gelöscht. Verwendung und architektonische Funktion müssen vorher geklärt werden.

## V1-3 Synchronisations- und Transferhärtung

- Aufnahmemetadaten korrekt über die Application-/Transfer-Grenze führen
- Provider-spezifische Metadaten und Transportdetails aus dem Core heraushalten
- Remote-Payload-Integrität vor erfolgreichem Abschluss nachweisen
- Idempotenz und Konfliktsemantik erhalten
- vorhandene Nextcloud-Upload-Semantik nutzen, statt ein paralleles Resume-Protokoll zu bauen

Für V1 gilt zunächst: **restart-safe reicht**, sofern reale Tests keinen relevanten Bedarf für echtes Fortsetzen ab dem letzten bestätigten Teilstück zeigen.

## V1-4 Reale Nextcloud-Verifikation

Reproduzierbarer Smoke-/Integrationstest:

```text
lokales RecordingArtifact
        ↓
Persistenz / Browser-Recovery
        ↓
SynchronizationWork / Transfer
        ↓
Nextcloud Application-Grenze
        ↓
Nextcloud Files
        ↓
Manifest + Payload verifizieren
        ↓
identischer zweiter Transfer
        ↓
AlreadySynchronized
```

Zusätzlich werden unterbrochene Übergaben, Konflikte, fehlende oder beschädigte Payloads, falsche Integritätsdaten, ungültige Credentials und HTTPS-only geprüft.

## V1-5 Erster nutzbarer Client

Der bestehende `ClientSessionService` bildet die Application-Grenze. Feasibility-Harnesses sind keine Produktionsclients.

Der erste nutzbare Client muss mindestens diesen vertikalen Workflow abbilden:

```text
Production Session erstellen
        ↓
Teilnehmer verwalten
        ↓
Session starten
        ↓
lokal aufnehmen
        ↓
Artifact persistieren
        ↓
synchronisieren
        ↓
Synchronisationsstatus anzeigen
        ↓
Session abschließen
```

Transport, Serialisierung, Authentifizierung und UI bleiben außerhalb des Domain-Core.

## V1-6 Öffentliche V1-Freigabe

Eine öffentliche V1 wird erst freigegeben, wenn Build und Tests reproduzierbar grün sind, der reale Nextcloud-Test funktioniert, HTTPS-only durchgesetzt ist, Credentials sicher behandelt werden, Remote-Integrität geprüft wird, Recovery/Retry getestet sind, der Minimalworkflow im Client funktioniert und Installation/Betrieb dokumentiert sind.

---

# Bewusste V1-Abgrenzung

Nicht Teil der V1 sind insbesondere:

- zusätzliche Remote-Provider
- eigene Remote-Storage-Infrastruktur
- Track-/Chunk-Level-Synchronisation als fachliche Einheit
- Delta-Synchronisation
- automatische Konfliktauflösung
- OIDC als V1-Pflicht
- Performance-Optimierung ohne Messdaten

---

# English Version

This document describes the current path toward the first publicly usable NC-PoRe V1.

ADRs explain **why** an architectural decision was made. This plan describes **which work remains for the current V1 state**.

## Current State

The technical foundation includes in particular:

- Core domain and lifecycle models
- recorder capture boundary and workflow coordination
- `RecordingArtifact` representation
- local persistence and recovery
- artifact registry and processing
- synchronization and transfer boundaries
- browser artifact persistence and recovery
- Nextcloud integration through the existing Application/OCS/Files API boundary
- integrity verification and idempotent handoff

NC-PoRe is in the **V1 hardening and productization phase**.

## V1 Work Plan

1. repository and documentation consolidation
2. legacy/dead-code audit
3. synchronization and transfer hardening
4. reproducible real-Nextcloud verification
5. completion of the first usable client
6. public V1 release readiness

For V1, existing Nextcloud upload semantics are used and provider-specific transport behavior remains outside the Core.

## Explicit V1 Boundary

The V1 scope does not include in particular:

- additional remote providers
- custom remote-storage infrastructure
- track/chunk-level synchronization as the domain unit
- delta synchronization
- automatic conflict resolution
- OIDC as a V1 requirement
- premature performance optimization without measurements
