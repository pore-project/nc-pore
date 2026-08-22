# NC-PoRe Implementation Plan

## Deutsche Version

---

# Zweck

Dieses Dokument beschreibt den Weg von der technischen Grundlage zur ersten öffentlich nutzbaren NC-PoRe-V1.

ADRs beantworten, **warum** eine Entscheidung getroffen wurde. Dieser Plan beschreibt, **was als Nächstes fertig werden muss**.

---

# Aktueller Stand

Die technische Basis ist weit fortgeschritten. Implementiert sind insbesondere:

- Core-Domain und Lifecycle-Modelle
- Recorder-Capture-Boundary und Workflow-Koordination
- vollständige lokale `RecordingArtifact`-Repräsentation
- lokale Persistenz und Recovery
- Artifact Registry und Processing
- vendor-neutrale Synchronisationswarteschlange
- deterministische Synchronisationsorchestrierung
- `ArtifactTransfer`-Provider-Grenze
- Nextcloud WebDAV Connector
- App-Password-Authentifizierung
- Idempotenz und Versionskonflikte
- Nextcloud Chunked Upload für große Payloads

NC-PoRe befindet sich damit in der **V1-Härtung und Produktisierung**.

---

# V1-Arbeitsplan

## V1-1 Repository- und Dokumentationskonsolidierung

Code, Branch-Zustand und Dokumentation müssen denselben Projektstand beschreiben. Historische Issues werden nachvollziehbar konsolidiert, ohne die Git-Historie umzuschreiben.

## V1-2 Legacy- und Dead-Code-Audit

Zu untersuchen sind insbesondere:

- `application/src/lib_source.rs`
- historische Recorder-Module
- Kompatibilitäts- und Übergangsexporte
- Feasibility-/Proof-of-Concept-Code
- scheinbar unreferenzierte öffentliche APIs

Nichts wird allein wegen seines Alters gelöscht. Verwendung und architektonische Funktion müssen vorher geklärt werden.

## V1-3 Synchronisationshärtung

- Aufnahmezeit und Display Name über die Application-/Transfer-Grenze an den Nextcloud-Connector weiterreichen
- Provider-spezifische Metadaten und Transportdetails aus dem Core heraushalten
- Remote-Payload-Integrität vor `Succeeded` beziehungsweise `AlreadySynchronized` nachweisen
- Idempotenz und Konfliktsemantik erhalten
- Nextclouds Upload-Semantik nutzen, statt ein eigenes Resume-Protokoll zu bauen

Für V1 gilt zunächst: **restart-safe reicht**, sofern reale Nextcloud-Tests keinen relevanten Bedarf für echtes Fortsetzen ab dem letzten bestätigten Chunk zeigen.

## V1-4 Reale Nextcloud-Verifikation

Reproduzierbarer Smoke-/Integrationstest:

```text
lokales RecordingArtifact
        ↓
Persistenz
        ↓
SynchronizationWork
        ↓
Nextcloud Connector
        ↓
Remote Artifact
        ↓
Manifest + Payload verifizieren
        ↓
identischer zweiter Transfer
        ↓
AlreadySynchronized
```

Zusätzlich: unterbrochener Upload, Konflikt, fehlende/beschädigte Payload, falscher Manifest-Hash, ungültige Credentials und HTTPS-only.

## V1-5 Erster nutzbarer Client

Der bestehende `ClientSessionService` ist die Application-Grenze. Der HTTP-Feasibility-Harness ist ausdrücklich kein Produktionsclient.

Der erste Client muss mindestens diesen vertikalen Workflow abbilden:

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

Eine öffentliche V1 wird erst freigegeben, wenn Build und Tests reproduzierbar grün sind, der reale Nextcloud-Smoke-Test funktioniert, HTTPS-only durchgesetzt ist, Credentials sicher behandelt werden, Remote-Integrität geprüft wird, Recovery/Retry getestet sind, der Minimalworkflow im Client funktioniert und Installation/Betrieb dokumentiert sind.

---

# Bewusste V1-Abgrenzung

Nicht Teil der V1 sind:

- weitere Remote-Provider
- eigene Remote-Storage-Infrastruktur
- Track-/Chunk-Level-Synchronisation als fachliche Einheit
- Delta-Synchronisation
- automatische Konfliktauflösung
- OIDC als V1-Pflicht
- Performance-Optimierung ohne Messdaten

---

# English Version

NC-PoRe is now in the **V1 hardening and productization phase**.

The implementation sequence is:

1. repository and documentation consolidation
2. legacy/dead-code audit
3. synchronization hardening
4. reproducible real-Nextcloud verification
5. completion of the first usable client
6. public V1 release readiness

For V1, Nextcloud remains the only productive provider, HTTPS is mandatory for public use, and provider-specific transport behavior remains outside the Core.
