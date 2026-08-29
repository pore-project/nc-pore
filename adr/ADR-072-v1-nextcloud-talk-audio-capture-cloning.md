# ADR-072: V1 Nextcloud Talk Audio Capture Cloning

## Status

Angenommen

## Datum

2026-08-29

---

<a id="deutsch"></a>

# Deutsch

## Kontext

NC-PoRe V1 verwendet Nextcloud Talk für die Teilnehmerkommunikation. Talk bleibt für Call, WebRTC, HPB und die Verteilung von Audio und Video zuständig. NC-PoRe muss gleichzeitig jede eigene Audiospur lokal beim jeweiligen Teilnehmer aufzeichnen.

Für diese lokale Aufnahme benötigt NC-PoRe das lokale Mikrofon-Audiosignal. Es darf daher nicht erst aus einem von Talk bereits verarbeiteten Audio-Track, aus einem serverseitigen Audiopfad oder aus einer späteren Talk-Schnittstelle gewonnen werden.

NC-PoRe verfügt bereits über einen allgemeinen Recording-Pfad mit Capture-Grenze, Recording-Lifecycle, Artifact-Erzeugung, Persistenz und Synchronisation. Die Talk-Integration soll diesen vorhandenen Weg erweitern und keinen zweiten Recording-Stack schaffen.

## Entscheidung

NC-PoRe verwendet für V1 **einen gemeinsamen browserseitigen Mikrofon-Capture und klont den lokalen Mikrofon-Audio-Track unmittelbar nach dessen Erzeugung durch `getUserMedia()`**.

Der Clone-Punkt liegt damit **vor jeglicher Nextcloud-Talk-eigenen Audioverarbeitung**.

Der erzeugte Clone wird als Capture-Quelle in den bestehenden generischen NC-PoRe-Recording-Pfad überführt. Die Talk-Integration ist damit eine weitere Capture-Quelle an der bestehenden Capture-Grenze und keine eigenständige Recording-Architektur.

```text
Mikrofon / Browser-Capture-Quelle
             │
             ▼
        getUserMedia()
             │
             ▼
      gemeinsame Media Source
             │
       ┌─────┴─────┐
       │           │
       ▼           ▼
  Talk Track   PoRE Clone
       │           │
       ▼           ▼
 Talk-Verarbeitung   bestehender
 WebRTC / HPB        PoRE Capture-
                     und Recording-Pfad
```

Talk und NC-PoRe erhalten jeweils einen eigenen Track derselben Capture-Source. Es gibt keinen zweiten unabhängigen Mikrofon-Capture durch NC-PoRe.

### Geräteunabhängigkeit

NC-PoRe bindet sich **nicht an ein bestimmtes Mikrofon, Audiointerface, Headset oder eine bestimmte `deviceId`**. Das heute validierte RØDECaster Pro ist ausschließlich ein Testgerät.

Jede Audioquelle, die der Browser über `getUserMedia()` als Audio-Track bereitstellt, wird grundsätzlich gleich behandelt. Die Auswahl und konkrete Bereitstellung des Aufnahmegeräts verbleibt bei Browser und Talk.

Damit ist die Capture-Architektur unabhängig davon, ob beispielsweise ein internes Mikrofon, USB-Mikrofon, Headset, Bluetooth-Gerät, Audiointerface oder eine andere vom Browser unterstützte Audioquelle verwendet wird.

### Browserunabhängigkeit

Die V1-Architektur basiert auf standardisierter Web-API-Semantik und ist **nicht an Firefox gebunden**. Zielbrowser sind gleichberechtigt:

- Firefox
- Chromium-basierte Browser
- Safari/WebKit

Browser-spezifische Unterschiede dürfen nicht in den bestehenden PoRE-Recording-Pfad hineinragen. Sie werden ausschließlich in der Browser-Integrationsschicht behandelt.

`MediaStream`, `MediaStreamTrack` und insbesondere `MediaStreamTrack.clone()` bilden die gemeinsame technische Grundlage. Die praktische Funktionsfähigkeit wird für die Zielbrowser jeweils durch Live-Integration validiert.

### Verantwortungsgrenzen

**Nextcloud Talk** bleibt vollständig verantwortlich für seinen Audio-Track und dessen weitere Verarbeitung, einschließlich WebRTC und HPB.

**NC-PoRe** verwendet ausschließlich seinen eigenen Clone für die lokale Aufnahme und darf diesen Track unabhängig für eigene Aufnahme- und Verarbeitungsanforderungen verwenden.

NC-PoRe darf insbesondere nicht davon abhängig sein, dass Talk einen bereits verarbeiteten Audio-Track an einer späteren Stelle zur Verfügung stellt.

### Rohsignal

Der NC-PoRe-Clone entsteht vor Talk-eigenem Noise Suppression, Gain-/Audio-Processing oder sonstiger Talk-Verarbeitung. Für V1 ist damit das **lokale Mikrofon-Capture als gemeinsame Quelle** die Aufnahmequelle von NC-PoRe.

Die Entscheidung legt keine bestimmte physikalische Geräteklasse fest und verlangt keine bitweise identische Ausgabe verschiedener Browser.

### Audio und Video

Diese Entscheidung betrifft den Audio-Capture. Video bleibt für V1 bei Nextcloud Talk. Talk bleibt für Teilnehmerkommunikation und Videoübertragung zuständig.

### Bestehender PoRE-Recording-Pfad

Nach dem Clone übernimmt NC-PoRe den bereits vorhandenen generischen Recording-Weg. Insbesondere werden keine parallelen Talk-spezifischen Lifecycle-, Artifact- oder Persistenzmodelle eingeführt.

Die bestehende Trennung bleibt erhalten:

1. Capture-Quelle liefert Audio.
2. Der bestehende Recording-Lifecycle steuert Start, Bereitschaft und Stop.
3. Der bestehende Artifact-Pfad verarbeitet das abgeschlossene Recording.
4. Bestehende Persistenz und Synchronisation bleiben zuständig.

Der Browser/Talk-Adapter ist damit auf die Capture-Integration begrenzt.

### Signet

Der gemeinsame Recording-Ablauf aus ADR-068 bleibt bestehen:

1. Host sendet Start.
2. Berechtigte Clients starten ihre lokale Aufnahme.
3. Jeder Client meldet `READY`, sobald die lokale Aufnahme tatsächlich läuft.
4. Erst wenn alle erforderlichen Teilnehmer `READY` gemeldet haben, sendet der Host das digitale Signal für das Opening Sync Signet.
5. Jeder Client erzeugt das Signet **lokal** und bringt es in seine lokale NC-PoRe-Aufnahmespur ein.

Das Signet muss nicht als vom Host stammendes Audiosignal übertragen werden.

## Live-Validierung

Am 29.08.2026 wurde die Clone-Entscheidung in einer realen lokalen Nextcloud-Talk-Installation validiert.

**Testumgebung:**

- Nextcloud 34.0.3 als Docker-Instanz
- Nextcloud Talk aktiviert
- Firefox 154.0
- HTTPS über Caddy mit lokalem `mkcert`-Zertifikat
- LAN-Zugriff über `A-desktop-G.local` / `192.168.192.126`
- reales Audioeingabegerät: RØDECaster Pro Analoges Stereo
- NC-PoRe App 0.1.0 mit frühem `getUserMedia()`-Hook

**Beobachtung:**

Talks reale `getUserMedia()`-Aufrufe wurden durch den NC-PoRe-Hook abgefangen. Für Audio-Aufrufe wurde jeweils ein eigener `MediaStreamTrack` erfolgreich mit `MediaStreamTrack.clone()` erzeugt. Der ursprüngliche Stream wurde unverändert an Talk zurückgegeben.

Der Clone wurde anschließend unabhängig über `MediaStream` und `MediaRecorder` aufgezeichnet. Die resultierende Testdatei `pore-talk-clone-test.ogx` enthielt 52,54 Sekunden reales Audio:

- Ogg Container
- Opus Codec
- Stereo
- 48 kHz Input Sample Rate
- ca. 130 kbit/s
- Encoder: Mozilla Firefox 154.0

Damit ist der primäre Capture-/Clone-Mechanismus unter realen Nextcloud-Talk-Bedingungen technisch nachgewiesen. Der Test beweist noch nicht die produktive Einbindung in den bestehenden PoRE-Recording-Lifecycle, Chunk-Persistenz, Signet-Synchronisation oder die spätere Artifact-/Transportintegration.

Der Test validiert außerdem **keine geräte- oder browserübergreifende Gleichheit**. Die Architektur ist dafür standardbasiert ausgelegt; Firefox ist der erste erfolgreich validierte Zielbrowser. Chromium und Safari/WebKit müssen in der Live-Integration separat validiert werden.

## Integrationspriorität

1. **Primär:** browserseitiger Audio-Capture → Track Clone unmittelbar nach `getUserMedia()` → Talk und NC-PoRe als getrennte Consumer → bestehender PoRE-Recording-Pfad.
2. Browser-spezifische Anpassungen bleiben auf die Integrationsschicht begrenzt und verändern den bestehenden Recording-/Artifact-Pfad nicht.
3. Ein alternativer Capture-Weg wird erst betrachtet, wenn der standardbasierte gemeinsame Capture für einen Zielbrowser technisch nicht erreichbar ist.

Die bisher dokumentierten Fallbacks über einen von PoRE vollständig übernommenen Talk-Capture oder einen separaten PoRE-Browserclient werden **nicht als regulärer V1-Pfad verfolgt**, solange der primäre gemeinsame Capture-Weg implementierbar ist.

## Konsequenzen

- NC-PoRe erhält das lokale Mikrofon-Audio vor Talk-eigener Audioverarbeitung.
- Talk kann seine vorhandene Audio-, Video-, WebRTC- und HPB-Infrastruktur weiterverwenden.
- NC-PoRe kann lokal aufzeichnen, ohne auf serverseitiges Recording angewiesen zu sein.
- Audio und Video können für V1 getrennt behandelt werden.
- Die konkrete Audio-Hardware ist für PoRE irrelevant.
- Firefox, Chromium und Safari/WebKit werden durch dieselbe standardbasierte Capture-Architektur adressiert.
- Browser-spezifische Implementierungsdetails bleiben außerhalb des bestehenden Recording-/Artifact-Pfads.
- Der bereits vorhandene PoRE-Recording-Weg wird wiederverwendet und nicht dupliziert.

## Abgrenzung

Diese ADR legt nicht fest, welche konkrete Talk-Frontend-Klasse oder welches Integrationsmodul den frühen Capture bereitstellt. Sie legt auch weder das Aufnahmecontainerformat noch spätere automatische Synchronisations- oder Driftkorrekturverfahren fest.

Die technische Integrationsstelle muss jedoch die zentralen Entscheidungen erfüllen:

- Der NC-PoRe-Clone wird unmittelbar am browserseitigen Audio-Capture und vor Talk-eigener Audioverarbeitung erzeugt.
- Der Capture-Weg ist geräteunabhängig.
- Der Capture-Weg ist für Firefox, Chromium-basierte Browser und Safari/WebKit ausgelegt.
- Der Clone wird in den bestehenden generischen PoRE-Recording-Pfad überführt.

---

<a id="english-version"></a>

# English Version

## Context

NC-PoRe V1 uses Nextcloud Talk for participant communication. Talk remains responsible for the call, WebRTC, HPB, and audio/video distribution. NC-PoRe must record each participant's own audio locally.

NC-PoRe requires the local microphone audio signal. It must not depend on a Talk-processed audio track, a server-side audio path, or a later Talk integration point.

NC-PoRe already has a generic recording path with a capture boundary, recording lifecycle, artifact creation, persistence, and synchronization. The Talk integration extends this existing path rather than creating a second recording stack.

## Decision

For V1, NC-PoRe uses **one browser-side microphone capture and clones the local microphone audio track immediately after it is created by `getUserMedia()`**.

The clone point is therefore **before any Nextcloud Talk audio processing**.

The resulting clone is passed into the existing generic NC-PoRe recording path as another capture source. Talk integration is therefore an additional capture source at the existing capture boundary, not a separate recording architecture.

```text
Microphone / browser capture source
              │
              ▼
         getUserMedia()
              │
              ▼
       shared Media Source
              │
        ┌─────┴─────┐
        │           │
        ▼           ▼
   Talk Track   PoRE Clone
        │           │
        ▼           ▼
 Talk processing    existing
 WebRTC / HPB       PoRE capture
                    and recording path
```

Talk and NC-PoRe each receive their own track from the same capture source. NC-PoRe does not perform a second independent microphone capture.

### Device Independence

NC-PoRe is **not tied to a specific microphone, audio interface, headset, or `deviceId`**. The RØDECaster Pro used in the current test is test hardware only.

Any audio source that the browser exposes through `getUserMedia()` as an audio track is treated the same way. Device selection and concrete device provisioning remain the responsibility of the browser and Talk.

The architecture is therefore independent of whether the user has an internal microphone, USB microphone, headset, Bluetooth device, audio interface, or another browser-supported audio source.

### Browser Independence

The V1 architecture is based on standardized Web API semantics and is **not Firefox-specific**. Target browsers are equal:

- Firefox
- Chromium-based browsers
- Safari/WebKit

Browser-specific differences must not leak into the existing PoRE recording path. They remain confined to the browser integration layer.

`MediaStream`, `MediaStreamTrack`, and in particular `MediaStreamTrack.clone()` provide the common technical basis. Actual behavior is validated through live integration for each target browser.

### Responsibility Boundaries

**Nextcloud Talk** remains fully responsible for its audio track and subsequent processing, including WebRTC and HPB.

**NC-PoRe** uses only its own clone for local recording and may process that clone independently for its own recording requirements.

NC-PoRe must not depend on Talk exposing an already processed audio track at a later integration point.

### Raw Signal

The NC-PoRe clone is created before Talk-specific noise suppression, gain/audio processing, or other Talk processing. For V1, the **local microphone capture as the shared source** is therefore NC-PoRe's recording source.

The decision does not prescribe a particular physical device class and does not require bit-identical output across browsers.

### Audio and Video

This decision concerns audio capture. Video remains with Nextcloud Talk for V1. Talk remains responsible for participant communication and video transport.

### Existing PoRE Recording Path

After cloning, NC-PoRe uses the existing generic recording path. No parallel Talk-specific lifecycle, artifact, or persistence model is introduced.

The existing separation remains intact:

1. The capture source provides audio.
2. The existing recording lifecycle controls start, readiness, and stop.
3. The existing artifact path processes the completed recording.
4. Existing persistence and synchronization remain responsible for storage and transfer.

The browser/Talk adapter is therefore limited to capture integration.

### Signet

The shared recording flow from ADR-068 remains in force:

1. Host sends Start.
2. Eligible clients start their local recording.
3. Each client reports `READY` once local recording is actually active.
4. Only after all required participants are ready does the Host send the digital Opening Sync Signet command.
5. Each client generates the signet **locally** into its own NC-PoRe recording track.

The signet does not need to be transmitted as host-originated audio.

## Live Validation

On 2026-08-29, the clone decision was validated in a real local Nextcloud Talk installation.

**Test environment:**

- Nextcloud 34.0.3 running in Docker
- Nextcloud Talk enabled
- Firefox 154.0
- HTTPS through Caddy with a local `mkcert` certificate
- LAN access through `A-desktop-G.local` / `192.168.192.126`
- real audio input device: RØDECaster Pro Analog Stereo
- NC-PoRe app 0.1.0 with the early `getUserMedia()` hook

**Observation:**

Real Talk `getUserMedia()` calls were intercepted by the NC-PoRe hook. For audio calls, an independent `MediaStreamTrack` was successfully created using `MediaStreamTrack.clone()`. The original stream was returned unchanged to Talk.

The clone was then recorded independently through `MediaStream` and `MediaRecorder`. The resulting test file `pore-talk-clone-test.ogx` contained 52.54 seconds of real audio:

- Ogg container
- Opus codec
- stereo
- 48 kHz input sample rate
- approximately 130 kbit/s
- encoder: Mozilla Firefox 154.0

This establishes the primary capture/clone mechanism under real Nextcloud Talk conditions. It does not yet validate production integration with the existing PoRE recording lifecycle, chunk persistence, signet synchronization, or later artifact/transport integration.

The test also does **not** establish cross-browser or cross-device equivalence. The architecture is designed for that through standard APIs; Firefox is the first successfully validated target browser. Chromium and Safari/WebKit must be validated separately during live integration.

## Integration Priority

1. **Primary:** browser-side audio capture → immediate track clone after `getUserMedia()` → Talk and NC-PoRe as separate consumers → existing PoRE recording path.
2. Browser-specific adaptations remain confined to the integration layer and do not alter the existing recording/artifact path.
3. An alternative capture path is considered only if the standard shared capture cannot be reached technically for a target browser.

The previously documented fallback paths in which PoRE fully owns Talk capture or uses a dedicated PoRE browser client are **not pursued as regular V1 paths** while the primary shared capture path remains implementable.

## Consequences

- NC-PoRe receives local microphone audio before Talk-specific audio processing.
- Talk can continue using its existing audio, video, WebRTC, and HPB infrastructure.
- NC-PoRe can record locally without relying on server-side recording.
- Audio and video can be handled separately for V1.
- The concrete audio hardware is irrelevant to PoRE.
- Firefox, Chromium-based browsers, and Safari/WebKit are addressed through the same standards-based capture architecture.
- Browser-specific implementation details remain outside the existing recording/artifact path.
- The existing PoRE recording path is reused rather than duplicated.

## Boundary

The exact Talk frontend class or integration module providing early capture is an implementation detail. The technical integration point must satisfy the following architectural decisions:

- The NC-PoRe clone is created immediately at browser-side audio capture and before Talk-specific audio processing.
- The capture path is device-independent.
- The capture path targets Firefox, Chromium-based browsers, and Safari/WebKit.
- The clone is passed into the existing generic PoRE recording path.
