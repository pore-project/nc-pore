# Deutsch ([English version below](#english-version))

## Nextcloud Remote Artifact Storage v1

**Status:** in Umsetzung abgeschlossen

**Scope:** Nextcloud als erster konkreter Remote-Storage-Provider für vollständige, bereits lokal persistierte `RecordingArtifact`s.

### Umgesetzt

- vendor-neutrale Synchronisations-Queue und Transfer-Grenze
- persistierte Synchronisationsarbeit mit Recovery von unterbrochenen Transfers
- Nextcloud-Verbindung mit App-Password-Authentifizierung
- WebDAV-Grundoperationen für Remote-State und Transfer
- konfigurierbarer Remote-Root mit Default `audio/`
- menschenlesbare Remote-Organisation nach konkretem Recording-Datum und -Minute
- optionaler menschenlesbarer Display Name als zusätzliche Identifikation
- technische Artifact-ID bleibt Teil der eindeutigen technischen Identität
- vollständiger Transfer eines lokal persistierten Artifacts einschließlich Manifest, Tracks und Chunks
- idempotente Behandlung bereits identischer Remote-Artefakte
- explizite Konflikterkennung
- Remote-Verifikation vor erfolgreicher Synchronisationsmeldung
- Nextcloud-resumable/chunked upload für große Payloads
- provider-spezifische Fehler bleiben außerhalb der vendor-neutralen Synchronisationsdomäne
- konkrete Nextcloud-Komposition des bestehenden `SynchronizationOrchestrator`
- Synchronisationsarbeit wird erst nach erfolgreicher lokaler Persistenz erzeugt

### Architekturgrenze

Nextcloud ist in v1 der einzige konkrete Provider. Die Synchronisations- und Transfergrenzen bleiben jedoch vendor-neutral. Weitere Provider können später als eigene, gegebenenfalls kostenpflichtige Add-ons ergänzt werden, ohne die Core-Domain mit Provider-spezifischen Typen zu belasten.

Die Anwendung übergibt nur provider-neutrale Informationen. Provider-spezifische Pfad-, Namens-, Transport- und Remote-Metadatenentscheidungen bleiben im jeweiligen Provider-Modul.

### Menschlesbare Remote-Struktur

Der konkrete Nextcloud-Provider organisiert Artefakte standardmäßig unter:

`audio/YYYY/MM/DD/HH-MM - <Display Name> - <Artifact-ID>/`

Fehlt ein Display Name, bleibt die Struktur trotzdem eindeutig und identifizierbar:

`audio/YYYY/MM/DD/HH-MM - <Artifact-ID>/`

Die Aufnahmezeit wird auf die Minute reduziert. Die Artifact-ID bleibt die technische Identität; Datum, Uhrzeit und Display Name dienen der menschlichen Orientierung.

### Abschluss

Die konkrete Nextcloud-Integration aus #159 ist damit implementiert. Die verbleibende Arbeit für spätere Versionen betrifft weitere Provider, nicht eine Änderung der vendor-neutralen Synchronisationsarchitektur.

## English Version

<a id="english-version"></a>

**Status:** implementation completed

**Scope:** Nextcloud as the first concrete remote-storage provider for complete, already persisted `RecordingArtifact`s.

### Implemented

- vendor-neutral synchronization queue and transfer boundary
- persisted synchronization work with recovery of interrupted transfers
- Nextcloud connection with App Password authentication
- WebDAV primitives for remote-state inspection and transfer
- configurable remote root with default `audio/`
- human-readable remote organization by concrete recording date and minute
- optional human-readable display name as additional identification
- technical artifact ID remains part of the unique technical identity
- complete transfer of a locally persisted artifact including manifest, tracks and chunks
- idempotent handling of already identical remote artifacts
- explicit conflict detection
- remote verification before successful synchronization
- Nextcloud resumable/chunked upload for large payloads
- provider-specific errors remain outside the vendor-neutral synchronization domain
- concrete Nextcloud composition of the existing `SynchronizationOrchestrator`
- synchronization work is created only after successful local persistence

### Architecture Boundary

Nextcloud is the only concrete provider in v1. The synchronization and transfer boundaries nevertheless remain vendor-neutral. Additional providers can later be added as separate, potentially paid add-ons without introducing provider-specific types into the Core domain.

The application passes only provider-neutral information. Provider-specific path, naming, transport and remote-metadata decisions remain inside the respective provider module.

### Human-readable Remote Structure

The concrete Nextcloud provider organizes artifacts by default as:

`audio/YYYY/MM/DD/HH-MM - <Display Name> - <Artifact-ID>/`

Without a display name, the structure remains identifiable:

`audio/YYYY/MM/DD/HH-MM - <Artifact-ID>/`

Recording time is reduced to minute precision. The artifact ID remains the technical identity; date, time and display name provide human orientation.

### Completion

The concrete Nextcloud integration from #159 is implemented. Remaining work for later versions concerns additional providers, not a change to the vendor-neutral synchronization architecture.
