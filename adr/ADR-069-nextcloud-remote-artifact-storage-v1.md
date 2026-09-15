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

## Dateiname und Remote-Repräsentation

Die konkrete menschenlesbare Dateibenennung ist Bestandteil der Nextcloud-Remote-Repräsentation und damit Verantwortung des Nextcloud-Connectors.

Für V1 gilt:

- Der Dateiname besteht aus einem menschenlesbaren Teilnehmernamen beziehungsweise Teilnehmer-Label und der Endung `.wav`.
- Existiert kein verwendbarer Teilnehmername, wird als Fallback die technische `captureId` verwendet.
- Für die Teilnehmerbezeichnung gilt diese Priorität:
  1. vorhandener menschenlesbarer Name des Teilnehmers aus dem aktiven Nextcloud-Talk-Kontext,
  2. `Host` für den Host ohne solchen Namen,
  3. `Participant 1`, `Participant 2`, … für weitere Teilnehmer ohne solchen Namen.
- Die Nummerierung der namenlosen Teilnehmer folgt der im Recording-Kontext festgelegten zeitlichen Teilnahme-/Beitrittsreihenfolge und bleibt für den konkreten Capture-Lifecycle stabil.
- Teilnehmernamen werden vor der Verwendung als Dateiname als einzelnes Files-Path-Segment normalisiert; insbesondere Pfadtrenner und Steuerzeichen dürfen keinen Pfadwechsel bewirken.
- Ein vorhandener Dateiname darf **nicht** stillschweigend überschrieben werden.
- Ist die vorhandene Datei inhaltlich identisch, darf sie als bereits vorhandene Repräsentation wiederverwendet werden.
- Bei einer anderen Payload unter demselben menschenlesbaren Namen wird ein deterministischer numerischer Suffix verwendet, beispielsweise `Max Muster (2).wav`.

Die technische Artifact-Identität bleibt unabhängig vom sichtbaren Dateinamen bestehen.

## Speicherziel und Teilnehmer

Das Ziel des Remote-Speichers ist die Files-Ablage des **Owners der Production**. Das Ziel wird serverseitig aus der authentifizierten Erstellung beziehungsweise Materialisierung der Production abgeleitet.

Ein Teilnehmer darf nicht durch ein frei übermitteltes Zielbenutzerfeld bestimmen, in wessen Files-Ablage ein Artifact gespeichert wird. Insbesondere ein Guest-Upload wird deshalb nicht automatisch im Files-Bereich des sendenden Guests abgelegt.

Die Auflösung des Storage-Ziels gehört zur Application-/Provider-Grenze; der sichtbare Teilnehmername des Dateinamens und die Storage-Zielidentität sind voneinander getrennte Informationen.

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

Die in diesem ADR beschriebene Provider-Grenze, die lokale Synchronisationswarteschlange, die Idempotenz-/Konfliktsemantik, die Nextcloud-Dateirepräsentation, die Payload-Integritätsprüfung und die Zuordnung des Remote-Speicherziels zur Production sind im aktuellen V1-Anwendungspfad implementiert.

Die verbleibenden V1-Härtungspunkte sind:

1. Provider-Metadaten zuverlässig durch den vollständigen normalen Synchronisationspfad an den Nextcloud-Connector übergeben.
2. Remote-Payload-Integrität bei der Erkennung bereits vorhandener Artefakte im vollständigen Providerpfad nachweisen.
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

## Filename and Remote Representation

Human-readable filenames are part of the concrete Nextcloud remote representation and are therefore the responsibility of the Nextcloud connector.

For V1:

- The filename consists of a human-readable participant name/label and the `.wav` extension.
- If no usable participant name is available, the technical `captureId` is used as fallback.
- Participant naming priority is: active Nextcloud Talk participant display name; `Host` for the host without such a name; `Participant 1`, `Participant 2`, etc. for other unnamed participants.
- Numbering of unnamed participants follows the recording context's established temporal participation/join order and remains stable for the concrete capture lifecycle.
- Participant names are normalized as a single Files path segment; path separators and control characters must not cause path traversal.
- An existing filename is never silently overwritten.
- An existing file with identical content may be reused as the already-present representation.
- Different payload content under the same human-readable name receives a deterministic numeric suffix such as `Max Muster (2).wav`.

Technical artifact identity remains independent of the visible filename.

## Storage Target and Participants

The remote storage target is the Files area of the **Production owner**. The target is resolved server-side from the authenticated creation/materialization of the Production.

A participant must not be able to choose the destination user's Files area through a freely supplied target-user field. In particular, a guest upload must not automatically be stored in the sending guest's Files area.

Storage-target resolution belongs to the application/provider boundary. The visible participant filename and the storage target identity are separate pieces of information.

## Metadata

Recording start time and human-readable display name are not Core domain fields for synchronization. They may cross the application/transfer boundary as provider-neutral transfer metadata and are translated into the Nextcloud representation only by the connector.

## Authentication and Security

V1 uses Nextcloud App Passwords. For a public V1 release, **HTTPS is mandatory**; HTTP is not a supported productive configuration.

## Implementation Status

The provider boundary, local synchronization queue, idempotency/conflict semantics, Nextcloud file representation, payload integrity checks, and Production-owner storage targeting are implemented in the current V1 application path.

Remaining V1 hardening consists of complete metadata propagation through the normal provider path, remote integrity verification for already-present artifacts in the full provider path, real-Nextcloud interruption testing, a reproducible end-to-end smoke test, and completion of the first usable client.

These remaining tasks do not change the architectural decision recorded here.
