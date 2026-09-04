# ADR-072: Host-Integrated Local Audio Capture via Connector

## Status

Angenommen

## Datum

2026-08-29

---

<a id="deutsch"></a>

# Deutsch

## Kontext

NC-PoRe V1 benötigt für lokale Teilnehmeraufnahmen eine Audio-Capture-Quelle aus der Host-Anwendung, in der die Kommunikation stattfindet. Nextcloud Talk ist der erste Host, für den diese Integration umgesetzt wird; später sollen weitere Host-Anwendungen über eigene Connectoren angebunden werden können.

NC-PoRe verfügt bereits über einen generischen Recording-Pfad mit Capture-Grenze, Recording-Lifecycle, Artifact-Erzeugung, Persistenz und Synchronisation. Eine Host-Integration soll diesen vorhandenen Weg erweitern und keinen zweiten Recording-Stack erzeugen.

Die Host-Anwendungen können unterschiedliche Media- und Lifecycle-Modelle verwenden. Deshalb darf der PoRE-Core weder von einer bestimmten Host-Anwendung noch von deren konkreter Media-Implementierung oder von einem bestimmten Aufnahmegerät abhängig werden.

## Entscheidung

NC-PoRe integriert Host-Anwendungen über **host-spezifische Connectoren**. Ein Connector adaptiert die jeweilige lokale Audio-Capture-Architektur der Host-Anwendung an die neutrale Capture-Schnittstelle des bestehenden PoRE-Recording-Pfads.

Der Connector stellt PoRE eine eigene, unabhängige Audio-Capture-Quelle zur Verfügung. Die Host-Anwendung behält ihre eigene Audioverarbeitung und ihren eigenen Media-Lifecycle.

Der PoRE-Core kennt dabei weder die konkrete Host-Anwendung noch deren Media-Interna. Host-spezifische Logik verbleibt vollständig im jeweiligen Connector.

```text
Host-Anwendung
      │
      ▼
Host-spezifische Media-/Capture-Architektur
      │
      ▼
Host Connector
      │
      ▼
neutrale PoRE Capture-Quelle
      │
      ▼
bestehender PoRE Recording-/Artifact-Pfad
      │
      ▼
Persistenz / Synchronisation / Transfer
```

## Verantwortungsgrenzen

**Host und Connector** sind für die Ermittlung und Bereitstellung der geeigneten lokalen Audioquelle verantwortlich.

**NC-PoRe Core** ist für den generischen Recording-Lifecycle, die Aufnahmeverarbeitung, Artifact-Erzeugung, Persistenz und Synchronisation verantwortlich.

Ein Connector darf keine host-spezifischen Lifecycle-, Artifact- oder Persistenzmodelle in den PoRE-Core einführen.

## Geräteunabhängigkeit

NC-PoRe bindet sich nicht an ein bestimmtes Mikrofon, Audiointerface, Headset oder eine bestimmte `deviceId`.

Die konkrete Audiohardware und deren Auswahl verbleiben bei Host-Anwendung und Browser beziehungsweise Betriebssystem. Ein Connector muss jede Audioquelle unterstützen, die die jeweilige Host-Integration als geeignete lokale Capture-Quelle bereitstellt.

Das RØDECaster Pro aus dem ersten Nextcloud-Talk-Reality-Check ist ausschließlich Testhardware und keine Architekturvorgabe.

## Browserunabhängigkeit

Die Connector-Architektur ist nicht an einen einzelnen Browser gebunden. Für browserbasierte Host-Integrationen sind insbesondere Firefox, Chromium-basierte Browser und Safari/WebKit gleichberechtigte Zielplattformen.

Browser-spezifische Unterschiede werden in der jeweiligen Integrationsschicht behandelt und dürfen nicht in den generischen PoRE-Recording-/Artifact-Pfad hineinragen.

Die verwendeten Web-APIs sollen soweit möglich auf standardisierter MediaStream-/MediaStreamTrack-Semantik beruhen. Eine konkrete Host-Integration kann zusätzliche browser- oder host-spezifische Anpassungen benötigen.

## Audio-Capture und unabhängige Aufnahme

Wenn die Host-Anwendung einen lokalen Audio-Track bereitstellt, soll der Connector eine unabhängige PoRE-Capture-Quelle daraus ableiten, ohne den von der Host-Anwendung verwendeten Track zu verändern.

Bei browserbasierten Integrationen kann dies insbesondere über `MediaStreamTrack.clone()` erfolgen. Der konkrete Clone-Punkt und der Umgang mit dem Media-Lifecycle sind jedoch **Connector-spezifische Entscheidungen** und werden nicht durch dieses ADR für alle Hosts vorgegeben.

Damit bleibt insbesondere offen, ob eine Host-Anwendung einen stabilen Track, einen austauschbaren Track, eine eigene Capture-API oder einen anderen geeigneten Integrationspunkt bereitstellt.

## Recording-Pfad

Nach der Übergabe an die neutrale Capture-Grenze übernimmt der bestehende PoRE-Weg:

1. Capture-Quelle liefert Audio.
2. Der bestehende Recording-Lifecycle steuert Start, Bereitschaft und Stop.
3. Der bestehende Artifact-Pfad verarbeitet das abgeschlossene Recording.
4. Bestehende Persistenz und Synchronisation bleiben zuständig.

Die Host-Integration wird damit nicht zu einer zweiten Recording-Architektur.

## Signet

Der gemeinsame Recording-Ablauf aus ADR-068 bleibt bestehen:

1. Host sendet Start.
2. Berechtigte Clients starten ihre lokale Aufnahme.
3. Jeder Client meldet `READY`, sobald die lokale Aufnahme tatsächlich läuft.
4. Erst wenn alle erforderlichen Teilnehmer `READY` gemeldet haben, sendet der Host das digitale Signal für das Opening Sync Signet.
5. Jeder Client erzeugt das Signet lokal in seiner NC-PoRe-Aufnahmespur.

Das Signet ist Bestandteil des generischen PoRE-Recording-Lifecycles und nicht Aufgabe eines einzelnen Host-Connectors.

## Validierung der ersten Host-Integration

Die erste konkrete Host-Integration ist Nextcloud Talk. Am 29.08.2026 wurde dort der grundlegende browserseitige Clone-Mechanismus unter realen Bedingungen validiert.

Testumgebung:

- Nextcloud 34.0.3 als Docker-Instanz
- Nextcloud Talk aktiviert
- Firefox 154.0
- HTTPS über Caddy mit lokalem `mkcert`-Zertifikat
- reales Audioeingabegerät: RØDECaster Pro Analoges Stereo

Der frühe PoC-Hook konnte reale Talk-`getUserMedia()`-Aufrufe abfangen und aus einem Audio-Track erfolgreich einen unabhängigen `MediaStreamTrack` klonen. Eine daraus erzeugte Testaufnahme enthielt 52,54 Sekunden reales Audio als Ogg/Opus, Stereo, 48 kHz und ca. 130 kbit/s.

Dieser Test validiert den technischen Clone-Mechanismus für die erste Host-Integration. Er ist **keine** Festlegung auf die konkrete Talk-Integrationsstelle und kein Nachweis für alle Browser oder alle Geräte.

Die Talk-spezifische Umsetzung ist in der internen ADR-073 beschrieben.

## Konsequenzen

- PoRE erhält eine wiederverwendbare Host-Connector-Architektur.
- Host-spezifische Media-Lifecycle-Logik bleibt außerhalb des PoRE-Core.
- Unterschiedliche Host-Anwendungen können unterschiedliche Capture-Strategien verwenden.
- Geräte- und Browserabhängigkeiten werden an der Integrationsgrenze gekapselt.
- Der vorhandene PoRE-Recording-/Artifact-Pfad wird wiederverwendet.
- Ein Connector kann auf einen von der Host-Anwendung bereits erzeugten Track aufsetzen, ohne die Host-Kommunikation zu übernehmen.
- Weitere Connectoren können später ergänzt werden, ohne den generischen Recording-Pfad neu zu bauen.

## Abgrenzung

Dieses ADR definiert die allgemeine Host-Connector-Architektur. Es legt nicht fest:

- wie eine konkrete Host-Anwendung ihren aktuellen Audio-Track ermittelt,
- wie Track-Replacement behandelt wird,
- welche konkreten Host- oder Browser-APIs ein Connector verwendet,
- welches Aufnahmecontainerformat verwendet wird,
- oder welche konkrete Host-Frontend-Komponente den Connector aufruft.

Diese Entscheidungen gehören in die jeweiligen Connector-ADRs und deren Implementierungen.

---

<a id="english-version"></a>

# English Version

## Context

NC-PoRe V1 requires a local audio capture source from the host application in which the communication takes place. Nextcloud Talk is the first host integration; additional host applications should be connectable through their own connectors later.

NC-PoRe already provides a generic recording path with a capture boundary, recording lifecycle, artifact creation, persistence, and synchronization. A host integration must extend this existing path rather than create a second recording stack.

Host applications may use different media and lifecycle models. PoRE Core must therefore not depend on a particular host application, its internal media implementation, or a particular recording device.

## Decision

NC-PoRe integrates host applications through **host-specific connectors**. A connector adapts the host application's local audio capture architecture to the neutral capture boundary of the existing PoRE recording path.

The connector provides PoRE with its own independent audio capture source. The host application retains ownership of its own audio processing and media lifecycle.

PoRE Core knows neither the concrete host application nor its media internals. Host-specific logic remains entirely within the respective connector.

```text
Host application
      │
      ▼
Host-specific media/capture architecture
      │
      ▼
Host connector
      │
      ▼
neutral PoRE capture source
      │
      ▼
existing PoRE recording/artifact path
      │
      ▼
persistence / synchronization / transfer
```

## Responsibility Boundaries

The **host and its connector** are responsible for discovering and providing the appropriate local audio source.

**NC-PoRE Core** is responsible for the generic recording lifecycle, recording processing, artifact creation, persistence, and synchronization.

A connector must not introduce host-specific lifecycle, artifact, or persistence models into PoRE Core.

## Device Independence

NC-PoRE is not tied to a specific microphone, audio interface, headset, or `deviceId`.

Concrete audio hardware and device selection remain the responsibility of the host application and the browser/operating system. A connector must support any audio source that its host integration provides as the appropriate local capture source.

The RØDECaster Pro used in the first Nextcloud Talk reality check is test hardware only and is not an architectural requirement.

## Browser Independence

The connector architecture is not tied to a single browser. For browser-based host integrations, Firefox, Chromium-based browsers, and Safari/WebKit are equal target platforms.

Browser-specific differences remain within the respective integration layer and must not leak into the generic PoRE recording/artifact path.

Where possible, browser integrations should rely on standardized MediaStream/MediaStreamTrack semantics. A concrete host integration may require additional browser- or host-specific adaptations.

## Audio Capture and Independent Recording

When a host application provides a local audio track, its connector should derive an independent PoRE capture source without modifying the track used by the host application.

For browser-based integrations this may use `MediaStreamTrack.clone()`. The concrete clone point and media lifecycle handling are, however, **connector-specific decisions** and are not prescribed for all hosts by this ADR.

A host may provide a stable track, replace tracks during its lifecycle, expose its own capture API, or provide another suitable integration point.

## Recording Path

Once a source crosses the neutral capture boundary, the existing PoRE path takes over:

1. The capture source provides audio.
2. The existing recording lifecycle controls start, readiness, and stop.
3. The existing artifact path processes the completed recording.
4. Existing persistence and synchronization remain responsible for storage and transfer.

Host integration therefore does not become a second recording architecture.

## Signet

The shared recording flow from ADR-068 remains in force. The host starts the flow, eligible clients start local recording, clients report `READY` once recording is active, and the Opening Sync Signet is generated locally after the required clients are ready.

Signet handling remains part of the generic PoRE recording lifecycle and is not the responsibility of an individual host connector.

## Validation of the First Host Integration

The first concrete host integration is Nextcloud Talk. On 2026-08-29 the fundamental browser-side clone mechanism was validated under real conditions using Nextcloud Talk, Firefox 154, HTTPS, and a RØDECaster Pro input.

The early PoC hook intercepted real Talk `getUserMedia()` calls and successfully created an independent `MediaStreamTrack` clone. A recording made from that clone contained 52.54 seconds of real audio as Ogg/Opus, stereo, 48 kHz, at approximately 130 kbit/s.

This validates the technical clone mechanism for the first host integration. It does **not** prescribe the final Talk integration point and does not establish all-browser or all-device compatibility.

The Talk-specific implementation is defined in internal ADR-073.

## Consequences

- PoRE gains a reusable host connector architecture.
- Host-specific media lifecycle logic remains outside PoRE Core.
- Different host applications may use different capture strategies.
- Device and browser dependencies are encapsulated at the integration boundary.
- The existing PoRE recording/artifact path is reused.
- A connector can consume a track already created by the host without taking over host communication.
- Additional connectors can be added without rebuilding the generic recording path.

## Scope

This ADR defines the general host connector architecture. It does not define how a particular host finds its current audio track, handles track replacement, selects concrete host/browser APIs, chooses an audio container, or invokes the connector from a specific frontend component.

Those decisions belong in the respective connector ADRs and implementations.
