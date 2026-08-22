# ADR-069: Nextcloud Remote Artifact Storage v1

## Status

Proposed

## Date

2026-08-22

## Decision Type

Architecture

---

# Deutsch ([English version below](#english-version))

# Kontext

NC-PoRe besitzt mit ADR-068 und der Synchronisationsgrundlage aus #66 einen lokalen Recording-Lifecycle und einen vendor-neutralen `ArtifactTransfer`-Vertrag.

Ein vollständig abgeschlossenes und lokal persistiertes `RecordingArtifact` soll nach der Aufnahme für die weitere Produktion auf einem Remote-Speicher bereitgestellt werden.

Für Version 1 soll dafür ausschließlich **Nextcloud** unterstützt werden.

Die Architektur soll zugleich ermöglichen, spätere Remote-Provider als separate Erweiterungen anzubieten, ohne den Recording-Core oder die bestehende Synchronisationsdomäne erneut an einen konkreten Anbieter zu binden.

Für v1 soll insbesondere vorhandene Nextcloud-Infrastruktur genutzt werden. NC-PoRe soll deshalb keine eigene Remote-Dateiablage, kein eigenes Chunking-Protokoll und keine eigene Sharing- oder Authentifizierungsinfrastruktur aufbauen, sofern Nextcloud die benötigte Funktion bereits bereitstellt.

---

# Entscheidung

NC-PoRe verwendet in v1 **Nextcloud als einzigen produktiven Remote-Provider für `RecordingArtifact`**.

Die Synchronisation beginnt erst, wenn das vollständige `RecordingArtifact` lokal persistiert und für die Synchronisation verfügbar ist.

Das `RecordingArtifact` bleibt die fachliche und technische Synchronisationseinheit. NC-PoRe zerlegt ein fertiges Artifact für v1 nicht in eine eigene verteilte Synchronisationsstruktur.

Der bestehende vendor-neutrale `ArtifactTransfer`-Vertrag bleibt die Provider-Grenze.

Die konkrete Implementierung wird als `NextcloudArtifactTransfer` außerhalb von Core und der vendor-neutralen Synchronisationsdomäne umgesetzt.

Damit ergibt sich für v1:

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

Spätere Provider können dieselbe `ArtifactTransfer`-Grenze implementieren. Eine zusätzliche generische Provider-Abstraktion wird nicht eingeführt, solange der bestehende Vertrag diese Verantwortung bereits ausreichend kapselt.

---

# Remote-Transport

Für v1 wird **Nextcloud WebDAV** als primäre Data Plane verwendet.

Nextcloud stellt über WebDAV die für den Artifact-Transfer benötigten grundlegenden Operationen bereit, insbesondere Datei- und Verzeichniszugriff sowie Metadatenabfragen.

Für große Dateien und unterbrochene Uploads wird die von Nextcloud bereitgestellte resumable/chunked Upload-Funktion verwendet, soweit dies für die konkrete Übertragung erforderlich ist.

NC-PoRe implementiert kein eigenes allgemeines Chunking-Protokoll für Nextcloud.

Nextcloud-spezifische HTTP-, WebDAV- und Upload-Details bleiben vollständig innerhalb des Nextcloud-Providers.

Eine Nutzung weiterer Nextcloud-APIs, beispielsweise OCS, ist möglich, wird aber nur dort eingeführt, wo WebDAV die benötigte Funktion nicht selbst abdeckt.

---

# Artifact-Identität und Remote-Identität

Die lokale NC-PoRe-Identität eines Artifacts bleibt unabhängig von der Identität, die Nextcloud für eine Datei oder ein Verzeichnis vergibt.

Für NC-PoRe bleiben insbesondere maßgeblich:

- `RecordingArtifactId`
- `manifest_hash`

Nextcloud-spezifische Werte wie beispielsweise File-ID oder ETag sind Provider-Metadaten. Sie ersetzen nicht die NC-PoRe-Artifact-Identität.

Damit gilt:

```text
NC-PoRe
ArtifactId + ManifestHash

        ≠

Nextcloud
FileId + ETag + weitere Remote-Metadaten
```

Der Nextcloud-Provider darf diese Informationen zur effizienten Erkennung eines bereits vorhandenen Remote-Zustands verwenden, aber die Synchronisationsdomäne bleibt von ihnen unabhängig.

---

# Vollständiges Artifact als Synchronisationseinheit

Nach erfolgreicher lokaler Persistierung wird das fertige `RecordingArtifact` als Einheit zur Synchronisation bereitgestellt.

Es gibt in v1 keine separate Remote-Synchronisationslogik für einzelne Recording-Chunks oder Tracks.

Die vorhandene technische Darstellung eines Artifacts und seiner Payloads bestimmt, welche lokalen Daten für den Transfer benötigt werden.

Die konkrete Remote-Repräsentation wird durch den Nextcloud-Provider festgelegt und darf keine neue fachliche Identität erzeugen.

Eine spätere Version kann gezielte Chunk-, Track- oder Delta-Synchronisation ergänzen. Diese Möglichkeit wird durch die Provider-Grenze nicht ausgeschlossen, ist aber nicht Bestandteil dieser Entscheidung.

---

# Idempotenz und Versionskonflikte

Der Provider muss den bestehenden Synchronisationsvertrag respektieren.

Für eine bereits vollständig vorhandene und identische Artifact-Version wird:

`AlreadySynchronized`

zurückgegeben.

Wird unter derselben NC-PoRe-Artifact-Identität eine andere Manifest-Version festgestellt, wird:

`Conflict`

zurückgegeben.

Ein bestehender Remote-Zustand darf nicht stillschweigend durch eine andere Artifact-Version überschrieben werden.

Ein erfolgreicher Transfer wird erst dann als `Succeeded` gemeldet, wenn der Provider den erfolgreichen Abschluss und die für NC-PoRe erforderliche Integrität des Remote-Artifacts feststellen kann.

---

# Integrität

Der `manifest_hash` bleibt die NC-PoRe-seitige Identität der konkreten Artifact-Version.

Nextcloud-eigene ETags, Checksums und weitere Integritätsinformationen werden als zusätzliche Provider-Metadaten betrachtet.

Eine Nextcloud-Checksum ersetzt nicht automatisch den NC-PoRe-Manifest-Hash, da beide unterschiedliche fachliche Aussagen treffen können.

Der Provider muss insbesondere verhindern, dass ein unvollständiger oder nachweislich inkonsistenter Upload als erfolgreich synchronisiert gemeldet wird.

---

# Authentifizierung

Für v1 wird die Authentifizierung gegen Nextcloud über **Nextcloud App Passwords** unterstützt.

Das normale Nextcloud-Benutzerpasswort wird nicht als dauerhafte NC-PoRe-Provider-Credential verwendet.

Die Speicherung und Verarbeitung der Zugangsdaten erfolgt außerhalb der fachlichen Synchronisationsmodelle und wird an die dafür vorgesehene Credential-/Secret-Infrastruktur der Anwendung angebunden.

Weitere Authentifizierungsverfahren, insbesondere OIDC-basierte Verfahren, können später ergänzt werden und sind nicht Bestandteil dieser v1-Entscheidung.

---

# Bewusste Abgrenzung

Diese Entscheidung umfasst ausdrücklich nicht:

- weitere Remote-Provider
- S3-, Azure-, Google-Drive- oder Dropbox-Implementierungen
- einen eigenen NC-PoRe-Remote-Server
- Peer-to-Peer-Synchronisation
- Track- oder Chunk-Level-Synchronisation als fachliche Remote-Einheit
- Delta-Synchronisation
- automatische Konfliktauflösung
- OIDC als v1-Pflichtfunktion
- eine eigene Sharing-Infrastruktur
- eine eigene Remote-Storage-Abstraktion oberhalb von `ArtifactTransfer`

Diese Themen können später als eigenständige Architektur- und Produktentscheidungen behandelt werden.

---

# Spätere Provider-Erweiterungen

Weitere Remote-Provider werden nach v1 schrittweise als separate Erweiterungen beziehungsweise kostenpflichtige Add-ons betrachtet.

Die bestehende Architektur soll diese Erweiterungen ermöglichen, ohne den Recording-Core oder die Synchronisationsdomäne an einen konkreten Provider zu binden.

Ein späterer Provider implementiert grundsätzlich dieselbe `ArtifactTransfer`-Grenze, sofern seine technischen Fähigkeiten dies erlauben.

Provider-spezifische Fähigkeiten dürfen innerhalb des jeweiligen Providers genutzt werden. Sie dürfen jedoch nicht als notwendige Abhängigkeit in den vendor-neutralen Core gelangen.

---

# Konsequenzen

## Vorteile

- v1 benötigt nur einen konkreten Remote-Provider.
- Nextcloud übernimmt wesentliche Infrastruktur für Speicherung, Upload, Wiederaufnahme, Metadaten und Berechtigungen.
- NC-PoRe muss kein eigenes allgemeines Remote-Dateiprotokoll entwickeln.
- Der lokale Recording-Lifecycle bleibt vollständig unabhängig vom Remote-Transport.
- `ArtifactTransfer` bleibt die klare technische Provider-Grenze.
- Spätere Provider können ergänzt werden, ohne den Core erneut umzubauen.
- Das fertige `RecordingArtifact` bleibt die verständliche Synchronisationseinheit.

## Nachteile

- v1 ist bewusst von Nextcloud abhängig.
- Nextcloud-spezifische Fähigkeiten können nicht automatisch auf spätere Provider übertragen werden.
- Der erste produktive Provider muss die technischen Besonderheiten von WebDAV und Nextcloud kapseln.
- Provider-spezifische Zusatzfunktionen können später eigene Erweiterungen erfordern.

---

# Implementierungsfolgen

Aus dieser Entscheidung ergeben sich als nächste technische Schritte insbesondere:

1. Festlegung der minimal erforderlichen Nextcloud-Provider-Grenze innerhalb des bestehenden `ArtifactTransfer`-Vertrags.
2. Festlegung der Remote-Repräsentation eines fertigen `RecordingArtifact`.
3. Implementierung der Nextcloud-Verbindung und Authentifizierung.
4. Implementierung des einfachen vollständigen Artifact-Transfers.
5. Nutzung von Nextclouds resumable/chunked Upload für große beziehungsweise unterbrochene Transfers.
6. Verifikation des Remote-Zustands und Abbildung auf `TransferResult`.
7. End-to-End-Tests vom lokal persistierten Artifact bis zum bestätigten Remote-Zustand.

Diese Schritte sind Implementierungsarbeit und werden nicht durch dieses ADR als bereits umgesetzt behauptet.

---

# English Version ([Deutsche Version oben](#deutsch))

# Context

NC-PoRe now has a local recording lifecycle and, through ADR-068 and the synchronization foundation from #66, a vendor-neutral `ArtifactTransfer` contract.

A fully completed and locally persisted `RecordingArtifact` must become available on remote storage after recording for further production work.

For version 1, **Nextcloud is the only supported remote provider**.

The architecture must nevertheless allow later remote providers to be offered as separate extensions without binding the recording core or synchronization domain to a specific vendor.

For v1, existing Nextcloud infrastructure should be reused wherever possible. NC-PoRe should therefore not build its own remote file store, chunking protocol, sharing infrastructure, or authentication infrastructure when Nextcloud already provides the required capability.

---

# Decision

NC-PoRe uses **Nextcloud as the only productive remote provider for `RecordingArtifact` in v1**.

Synchronization starts only after the complete `RecordingArtifact` has been locally persisted and made available for synchronization.

The `RecordingArtifact` remains the technical synchronization unit. NC-PoRe does not introduce a separate distributed representation of individual recording chunks or tracks for v1.

The existing vendor-neutral `ArtifactTransfer` contract remains the provider boundary.

The concrete implementation is provided as `NextcloudArtifactTransfer` outside the core and vendor-neutral synchronization domain.

For v1 this results in:

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

Later providers may implement the same `ArtifactTransfer` boundary. No additional generic provider abstraction is introduced as long as the existing contract already provides sufficient encapsulation.

---

# Remote Transport

**Nextcloud WebDAV** is used as the primary data plane for v1.

Nextcloud provides the file and directory operations and metadata access required for artifact transfer through WebDAV.

For large files and interrupted uploads, the resumable/chunked upload functionality provided by Nextcloud is used where required for the concrete transfer.

NC-PoRe does not implement a separate general-purpose chunking protocol for Nextcloud.

Nextcloud-specific HTTP, WebDAV, and upload details remain entirely inside the Nextcloud provider.

Additional Nextcloud APIs, such as OCS, may be used where WebDAV alone does not provide a required capability, but they are not introduced as a general dependency of the synchronization domain.

---

# Artifact Identity and Remote Identity

The local NC-PoRe identity of an artifact remains independent of the identity assigned to a file or directory by Nextcloud.

The following remain authoritative for NC-PoRe:

- `RecordingArtifactId`
- `manifest_hash`

Nextcloud-specific values such as file IDs or ETags are provider metadata. They do not replace the NC-PoRe artifact identity.

Thus:

```text
NC-PoRe
ArtifactId + ManifestHash

        ≠

Nextcloud
FileId + ETag + other remote metadata
```

The Nextcloud provider may use these values to efficiently detect an existing remote state, but the synchronization domain remains independent of them.

---

# Complete Artifact as the Synchronization Unit

After successful local persistence, the completed `RecordingArtifact` is made available for synchronization as one unit.

v1 does not introduce separate remote synchronization logic for individual recording chunks or tracks.

The existing technical representation of an artifact and its payloads determines which local data is required for transfer.

The concrete remote representation is defined by the Nextcloud provider and must not introduce a new domain identity.

A later version may add targeted chunk-, track-, or delta-level synchronization. The provider boundary does not prevent this, but it is not part of this decision.

---

# Idempotency and Version Conflicts

The provider must respect the existing synchronization contract.

If an identical artifact version is already completely present, the provider returns:

`AlreadySynchronized`

If a different manifest version is found under the same NC-PoRe artifact identity, the provider returns:

`Conflict`

An existing remote state must not be silently overwritten by another artifact version.

A successful transfer is reported as `Succeeded` only after the provider can establish successful completion and the integrity required by NC-PoRe for the remote artifact.

---

# Integrity

`manifest_hash` remains the NC-PoRe identity of the concrete artifact version.

Nextcloud ETags, checksums, and other integrity information are treated as additional provider metadata.

A Nextcloud checksum does not automatically replace the NC-PoRe manifest hash because the two can express different technical properties.

The provider must ensure that an incomplete or demonstrably inconsistent upload is never reported as successfully synchronized.

---

# Authentication

For v1, authentication against Nextcloud uses **Nextcloud App Passwords**.

The normal Nextcloud user password is not used as a persistent NC-PoRe provider credential.

Credential storage and handling remain outside the synchronization domain and are connected to the application's designated credential/secret infrastructure.

Additional authentication mechanisms, including OIDC-based approaches, may be added later and are not required for v1.

---

# Explicit Scope Exclusions

This decision explicitly excludes:

- additional remote providers
- S3, Azure, Google Drive, or Dropbox implementations
- a dedicated NC-PoRe remote server
- peer-to-peer synchronization
- track- or chunk-level synchronization as the remote domain unit
- delta synchronization
- automatic conflict resolution
- OIDC as a v1 requirement
- a custom sharing infrastructure
- another generic remote-storage abstraction above `ArtifactTransfer`

These topics may be addressed through separate architectural and product decisions later.

---

# Later Provider Extensions

Additional remote providers will be considered incrementally as separate extensions or paid add-ons after v1.

The existing architecture is intended to make these extensions possible without binding the recording core or synchronization domain to a specific provider.

A later provider generally implements the same `ArtifactTransfer` boundary where its technical capabilities allow it.

Provider-specific capabilities may be used inside the respective provider, but they must not become required dependencies of the vendor-neutral core.

---

# Consequences

## Advantages

- v1 requires only one concrete remote provider.
- Nextcloud provides substantial infrastructure for storage, upload, resumption, metadata, and permissions.
- NC-PoRe does not need to implement its own general-purpose remote file protocol.
- The local recording lifecycle remains fully independent of remote transport.
- `ArtifactTransfer` remains the clear technical provider boundary.
- Later providers can be added without redesigning the core.
- The completed `RecordingArtifact` remains the understandable synchronization unit.

## Disadvantages

- v1 deliberately depends on Nextcloud.
- Nextcloud-specific capabilities cannot automatically be assumed by later providers.
- The first productive provider must encapsulate WebDAV and Nextcloud-specific behavior.
- Provider-specific extensions may require additional work later.

---

# Implementation Consequences

This decision leads to the following technical work:

1. Define the minimal Nextcloud provider boundary within the existing `ArtifactTransfer` contract.
2. Define the remote representation of a completed `RecordingArtifact`.
3. Implement the Nextcloud connection and authentication.
4. Implement the basic complete artifact transfer.
5. Use Nextcloud resumable/chunked upload for large or interrupted transfers.
6. Verify remote state and map it to `TransferResult`.
7. Add end-to-end tests from locally persisted artifact to confirmed remote state.

These are implementation tasks and are not claimed as already implemented by this ADR.

---
