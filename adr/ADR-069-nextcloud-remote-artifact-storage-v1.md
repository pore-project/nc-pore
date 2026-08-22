# ADR-069: Nextcloud Remote Artifact Storage v1

## Status

Accepted

## Date

2026-08-22

## Decision Type

Architecture

---

# Deutsch

## Kontext

NC-PoRe besitzt mit ADR-068 und der Synchronisationsgrundlage einen lokalen Recording-Lifecycle und einen vendor-neutralen `ArtifactTransfer`-Vertrag. Für V1 soll ein vollständig lokal persistiertes `RecordingArtifact` anschließend für die weitere Produktion auf Remote-Speicher bereitgestellt werden.

Für V1 wird ausschließlich **Nextcloud** als produktiver Remote-Provider unterstützt.

## Entscheidung

NC-PoRe verwendet in V1 **Nextcloud als einzigen produktiven Remote-Provider für `RecordingArtifact`**.

Die Synchronisation beginnt erst, wenn das vollständige `RecordingArtifact` lokal persistiert und für die Synchronisation verfügbar ist.

Das `RecordingArtifact` bleibt die fachliche und technische Synchronisationseinheit. Der bestehende vendor-neutrale `ArtifactTransfer`-Vertrag bleibt die Provider-Grenze.

Die konkrete Implementierung erfolgt als `NextcloudArtifactTransfer` außerhalb von Core und vendor-neutraler Synchronisationsdomäne.

```text
RecordingArtifact
      ↓
SynchronizationWork
      ↓
SynchronizationOrchestrator
      ↓
ArtifactTransfer
      ↓
NextcloudArtifactTransfer
      ↓
Nextcloud
```

Eine zusätzliche generische Provider-Abstraktion wird nicht eingeführt.

## Remote-Transport

Nextcloud WebDAV ist die primäre Data Plane. Nextcloud-spezifische HTTP-, WebDAV- und Upload-Details bleiben vollständig innerhalb des Providers.

Für große Dateien nutzt der Provider Nextclouds Chunked-/resumable-Upload-Funktion. NC-PoRe definiert dafür kein eigenes allgemeines Remote-Chunking-Protokoll.

Provider-spezifische Fähigkeiten dürfen innerhalb des Connectors genutzt werden, dürfen aber nicht zu notwendigen Abhängigkeiten des Core werden.

## Artifact-Identität und Remote-Identität

Für NC-PoRe bleiben insbesondere `RecordingArtifactId` und `manifest_hash` maßgeblich. Nextcloud File-IDs, ETags und Checksums sind Provider-Metadaten und ersetzen diese Identität nicht.

Ein identischer, vollständig vorhandener Remote-Zustand wird als `AlreadySynchronized` behandelt. Eine andere Manifest-Version unter derselben Artifact-Identität führt zu `Conflict`. Ein bestehender Remote-Zustand wird nicht stillschweigend überschrieben.

## Integrität

Ein Transfer darf nur dann als `Succeeded` beziehungsweise `AlreadySynchronized` gelten, wenn der Provider die erforderliche Vollständigkeit und Integrität des Remote-Artifacts feststellen kann.

Der `manifest_hash` bleibt die NC-PoRe-seitige Identität der konkreten Artifact-Version. Provider-seitige Checksums und ETags sind zusätzliche technische Nachweise.

Die konkrete Prüfung der Remote-Payloads ist Verantwortung des Nextcloud-Connectors. Der Core muss weder WebDAV noch Nextcloud-Checksums kennen.

## Metadaten

Aufnahmezeit und menschenlesbarer Display Name sind **keine fachlichen Bestandteile des Core-Synchronisationsmodells**. Sie dürfen als provider-neutrale Transfer-Metadaten über die Application-/Transfer-Grenze weitergereicht werden und werden erst im Nextcloud-Connector in die konkrete Remote-Repräsentation übersetzt.

Damit bleibt die menschenlesbare Nextcloud-Ordnerstruktur eine Connector-Funktion und keine Core-Verantwortung.

## Authentifizierung und Sicherheit

V1 verwendet Nextcloud App Passwords. Das normale Nextcloud-Benutzerpasswort wird nicht als dauerhafte Provider-Credential verwendet.

Für eine öffentliche V1-Freigabe werden **ausschließlich HTTPS-Endpunkte** unterstützt. HTTP ist kein zulässiger produktiver Konfigurationspfad.

Credential-/Secret-Speicherung bleibt außerhalb des fachlichen Synchronisationsmodells.

## Bewusste Abgrenzung

Nicht Bestandteil dieser V1-Entscheidung sind:

- weitere Remote-Provider
- ein eigener NC-PoRe-Remote-Server
- Peer-to-Peer-Synchronisation
- fachliche Track-/Chunk-Level-Synchronisation
- Delta-Synchronisation
- automatische Konfliktauflösung
- OIDC als V1-Pflichtfunktion
- eigene Sharing-Infrastruktur
- eine weitere generische Remote-Storage-Abstraktion oberhalb von `ArtifactTransfer`

## Implementierungsstand

Die in diesem ADR beschriebene Provider-Grenze, der Nextcloud-Connector, die lokale Synchronisationswarteschlange, Idempotenz-/Konfliktsemantik, WebDAV-Transfer und Chunked Upload sind im Repository implementiert.

Die verbleibenden V1-Härtungspunkte sind:

1. Provider-Metadaten zuverlässig durch den normalen Synchronisationspfad an den Nextcloud-Connector übergeben.
2. Remote-Payload-Integrität nach Transfer und bei der Erkennung bereits vorhandener Artefakte nachweisen.
3. Verhalten bei unterbrochenen Chunked Uploads gegen eine reale Nextcloud-Instanz verifizieren.
4. Einen reproduzierbaren Nextcloud-End-to-End-/Smoke-Test bereitstellen.
5. Den ersten nutzbaren Client fertigstellen.

Diese Punkte ändern die in diesem ADR getroffene Architekturentscheidung nicht.

---

# English Version

## Context

NC-PoRe has a local recording lifecycle and a vendor-neutral `ArtifactTransfer` contract. V1 requires completed, locally persisted `RecordingArtifact` instances to become available on remote storage for further production work.

For V1, **Nextcloud is the only productive remote provider**.

## Decision

NC-PoRe uses **Nextcloud as the only productive remote provider for `RecordingArtifact` in V1**.

Synchronization starts only after the complete artifact has been locally persisted. The completed `RecordingArtifact` remains the synchronization unit and `ArtifactTransfer` remains the provider boundary.

The concrete implementation is `NextcloudArtifactTransfer`, outside the Core and vendor-neutral synchronization domain.

Provider-specific WebDAV, HTTP and upload behavior remains inside the Nextcloud connector. Nextcloud chunked/resumable upload is used where appropriate; NC-PoRe does not define a generic remote chunking protocol.

## Identity and Integrity

`RecordingArtifactId` and `manifest_hash` remain authoritative for NC-PoRe. Nextcloud file IDs, ETags and checksums are provider metadata.

Identical complete remote state maps to `AlreadySynchronized`; a different manifest under the same artifact identity maps to `Conflict`; existing remote state is never silently overwritten.

A transfer is successful only after the provider establishes the required completeness and integrity of the remote artifact. Payload verification is a responsibility of the Nextcloud connector, not the Core.

## Metadata

Recording start time and human-readable display name are not Core domain fields for synchronization. They may cross the application/transfer boundary as provider-neutral transfer metadata and are translated into the Nextcloud representation only by the connector.

## Authentication and Security

V1 uses Nextcloud App Passwords. For a public V1 release, **HTTPS is mandatory**; HTTP is not a supported productive configuration.

## Implementation Status

The provider boundary, Nextcloud connector, local synchronization queue, idempotency/conflict semantics, WebDAV transfer and chunked upload are implemented.

Remaining V1 hardening consists of metadata propagation through the normal transfer path, remote payload integrity verification, real-Nextcloud interruption testing, a reproducible end-to-end smoke test, and completion of the first usable client.

These remaining tasks do not change the architectural decision recorded here.
