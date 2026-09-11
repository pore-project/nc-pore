# ADR-082: Browser-to-Application Artifact Transport Boundary

- Status: Accepted
- Date: 2026-09-05
- Decision Type: Architecture

---

<a id="deutsch"></a>

# Deutsch

## Kontext

NC-PoRe besitzt inzwischen eine browserseitige Grenze für die lokale Finalisierung von Aufnahmen und einen Application-seitigen Adapter, der das finalisierte Browser-Payload in den bestehenden `RecordingArtifactProcessor`-Pfad überführen kann.

Das Repository enthält derzeit keinen Server-Runtime für den Rust-`application`-Crate. Die Nextcloud-App ist eine PHP-Host-Integration, während der Rust-Workspace Domain-, Application-, Recorder- und Infrastructure-Bibliotheken enthält. Deshalb darf ein Browser-HTTP-Endpunkt nicht so implementiert werden, als wäre die Rust-Application-Library bereits ein HTTP-Server.

Die bestehende Persistenz- und Synchronisationsarchitektur ist bereits festgelegt und muss der einzige Artifact-Pfad bleiben:

`Browser capture -> Application handoff -> RecordingArtifact -> PersistenceProvider -> SynchronizationWork -> ArtifactTransfer -> Nextcloud`

---

## Entscheidung

Die Übergabe vom Browser zur Application bildet eine eigene Transportgrenze.

Der Transport überträgt:

- `Core.ProductionId`
- `Core.RecordingId`
- `Recorder.RecordingSessionId`
- technische Capture-Identität des Browsers
- Track-/Capture-Metadaten
- das finalisierte Payload

Der Transport DARF NICHT:

- direkt nach WebDAV schreiben
- einen zweiten Persistence Store erzeugen
- einen zweiten Recording-Lifecycle erzeugen
- `ProductionId` oder `RecordingId` als `ArtifactId` verwenden
- den Browser von Recorder-Persistenz oder Nextcloud-WebDAV-Details abhängig machen

Der Rust-`application`-Crate bleibt transportneutral. Ein konkreter HTTP-Runtime kann später als Composition Root eingeführt werden, die die bestehende Application-Grenze aufruft. Die Nextcloud-PHP-Anwendung kann Host-Authentifizierung und Session-Integration bereitstellen, darf aber keine zweite Recording-Persistenzimplementierung werden.

Solange eine solche Runtime nicht existiert, gilt kein Endpoint als annehmender Persistence-Endpoint. Eine Route, die eine erfolgreiche Annahme meldet, ohne die Application-Grenze aufzurufen, ist ausdrücklich verboten.

---

## Konsequenzen

Der Browser-Transportvertrag kann unabhängig von Deployment- und Runtime-Fragen implementiert und getestet werden.

Eine konkrete Runtime muss die folgende Kette Ende-zu-Ende nachweisen, bevor der Browser-Transport als vollständig betrachtet werden kann:

1. authentifizierte Browser-Anfrage
2. Transport-Decoding und Validierung
3. Application `BrowserRecordingArtifact`
4. bestehender `RecordingArtifactProcessor`
5. bestehender `PersistenceProvider`
6. Einreihung in die Synchronisation
7. bestehender `ArtifactTransfer`
8. autoritatives Recording Completion

Recorder und Nextcloud Provider benötigen für diese Grenze keine architektonische Änderung.

---

## Nicht-Entscheidungen

Diese ADR legt nicht fest:

- ein Rust-HTTP-Framework
- ein Prozessmodell oder Deployment-Topologie
- ob Nextcloud an einen separaten PoRE-Service proxied
- REST gegenüber einem anderen Transportprotokoll
- das Format von Authentifizierungstokens
- die Semantik entfernter Artifact-Pfade

Diese Entscheidungen gehören in den konkreten Runtime-/Composition-Slice und müssen vor der Implementierung eines tatsächlich annehmenden HTTP-Endpoints getroffen werden.

---

<a id="english-version"></a>

# English Version

## Context

NC-PoRE now has a browser-local recording finalization boundary and an application-side adapter that can transform the finalized browser payload into the existing `RecordingArtifactProcessor` path.

The repository does not currently contain a server runtime for the Rust `application` crate. The Nextcloud app is a PHP host integration, while the Rust workspace contains domain, application, recorder, and infrastructure libraries. Therefore a browser HTTP endpoint must not be implemented as if the Rust application library were already an HTTP server.

The existing persistence and synchronization architecture is already established and must remain the only artifact path:

`Browser capture -> Application handoff -> RecordingArtifact -> PersistenceProvider -> SynchronizationWork -> ArtifactTransfer -> Nextcloud`

---

## Decision

The browser-to-application handoff is a distinct transport boundary.

The transport carries:

- `Core.ProductionId`
- `Core.RecordingId`
- `Recorder.RecordingSessionId`
- browser technical capture identity
- track/capture metadata
- the finalized payload

The transport MUST NOT:

- write directly to WebDAV
- create a second persistence store
- create a second recording lifecycle
- use `ProductionId` or `RecordingId` as an `ArtifactId`
- make the browser depend on recorder persistence or Nextcloud WebDAV details

The Rust `application` crate remains transport-neutral. A concrete HTTP runtime may later be introduced as a composition root that invokes the existing application boundary. The Nextcloud PHP application may provide host authentication and session integration, but it must not become a second recording persistence implementation.

Until such a runtime exists, no endpoint is considered an accepting persistence endpoint. A route that reports successful acceptance without invoking the application boundary is explicitly forbidden.

---

## Consequences

The browser transport contract can be implemented and tested independently from deployment and runtime concerns.

A concrete runtime must prove the following chain end-to-end before the browser transport is considered complete:

1. authenticated browser request
2. transport decoding and validation
3. application `BrowserRecordingArtifact`
4. existing `RecordingArtifactProcessor`
5. existing `PersistenceProvider`
6. synchronization enqueue
7. existing `ArtifactTransfer`
8. authoritative recording completion

The existing recorder and Nextcloud provider require no architectural change for this boundary.

---

## Non-Decisions

This ADR does not choose:

- a Rust HTTP framework
- a process model or deployment topology
- whether Nextcloud proxies to a separate PoRE service
- REST versus another transport protocol
- authentication token format
- remote artifact path semantics

Those decisions belong to the concrete runtime/composition slice and must be made before implementing a real accepting HTTP endpoint.
