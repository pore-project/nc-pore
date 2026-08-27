# ADR-072: V1 Nextcloud Talk Audio Capture Cloning

## Status

Accepted

## Date

2026-08-27

## Decision Type

Architecture

---

# Deutsch

## Kontext

NC-PoRe V1 verwendet Nextcloud Talk für die Teilnehmerkommunikation. Talk bleibt für Call, WebRTC, HPB und die Verteilung von Audio und Video zuständig. NC-PoRe muss gleichzeitig jede eigene Audiospur lokal beim jeweiligen Teilnehmer aufzeichnen.

Für diese lokale Aufnahme benötigt NC-PoRe das unverarbeitete Mikrofon-Audiosignal. Es darf daher nicht erst aus einem von Talk bereits bearbeiteten Audio-Track, aus einem serverseitigen Audiopfad oder aus einer späteren Talk-Schnittstelle gewonnen werden.

## Entscheidung

NC-PoRe verwendet für V1 **einen gemeinsamen browserseitigen Mikrofon-Capture und klont den lokalen Mikrofon-Audio-Track unmittelbar nach dessen Erzeugung durch `getUserMedia()`**.

Der Clone-Punkt liegt damit **vor jeglicher Nextcloud-Talk-eigenen Audioverarbeitung**.

```text
Mikrofon
   │
   ▼
getUserMedia()
   │
   ▼
gemeinsame Media Source
   │
   ├── Talk Audio Track
   │      └── Talk-Verarbeitung → WebRTC / HPB
   │
   └── NC-PoRe Audio Track (Clone)
          └── unverarbeitet → lokale PoRE-Aufnahme
```

Talk und NC-PoRe erhalten jeweils einen eigenen Track derselben Mikrofon-Source. Es gibt keinen zweiten unabhängigen Mikrofon-Capture.

### Verantwortungsgrenzen

**Nextcloud Talk** bleibt vollständig verantwortlich für seinen Audio-Track und dessen weitere Verarbeitung, einschließlich WebRTC und HPB.

**NC-PoRe** verwendet ausschließlich seinen eigenen Clone für die lokale Aufnahme und darf diesen Track unabhängig für eigene Aufnahme- und Verarbeitungsanforderungen verwenden.

NC-PoRe darf insbesondere nicht davon abhängig sein, dass Talk einen bereits verarbeiteten Audio-Track an einer späteren Stelle zur Verfügung stellt.

### Rohsignal

Der NC-PoRe-Clone entsteht vor Talk-eigenem Noise Suppression, Gain-/Audio-Processing oder sonstiger Talk-Verarbeitung. Für V1 ist damit ausdrücklich das **lokale Mikrofon-Rohsignal** die Aufnahmequelle von NC-PoRe.

### Audio und Video

Diese Entscheidung betrifft den Audio-Capture. Video bleibt für V1 bei Nextcloud Talk. Talk bleibt für Teilnehmerkommunikation und Videoübertragung zuständig.

### Signet

Der gemeinsame Recording-Ablauf aus ADR-068 bleibt bestehen:

1. Host sendet Start.
2. Berechtigte Clients starten ihre lokale Aufnahme.
3. Jeder Client meldet `READY`, sobald die lokale Aufnahme tatsächlich läuft.
4. Erst wenn alle erforderlichen Teilnehmer `READY` gemeldet haben, sendet der Host das digitale Signal für das Opening Sync Signet.
5. Jeder Client erzeugt das Signet **lokal** und bringt es in seine lokale NC-PoRe-Aufnahmespur ein.

Das Signet muss nicht als vom Host stammendes Audiosignal übertragen werden.

### Browser

Die V1-Architektur setzt auf die standardisierte MediaStream-/MediaStreamTrack-Semantik der Web-Plattform und berücksichtigt ausdrücklich:

- Firefox
- Chromium-basierte Browser
- Safari/WebKit

Das standardkonforme Verhalten von `MediaStreamTrack.clone()` wird für V1 als Grundlage akzeptiert. Ein separater Clone-/Synchronisations-PoC ist **kein V1-Gate**. Die tatsächliche Funktion wird bei der Live-Integration validiert.

NC-PoRe verlangt keine bitweise identische Container- oder Recorder-Ausgabe. Entscheidend ist, dass beide Tracks aus derselben Capture-Source stammen und als unabhängige Consumer dieser Source verwendet werden können.

## Integrationspriorität

1. **Primär:** Mikrofon-Capture → Track Clone unmittelbar nach `getUserMedia()` → Talk und NC-PoRe als getrennte Consumer.
2. **Fallback:** NC-PoRe übernimmt den lokalen Mikrofon-Capture und stellt Talk einen geeigneten Audio-Track zur Verfügung, falls der frühe gemeinsame Capture in der Talk-Integration nicht erreichbar ist.
3. **Letzter Fallback:** eigener NC-PoRe-Browserclient, falls die gewünschte Talk-Integration nicht sauber realisierbar ist.

Die Fallbacks werden nicht verfolgt, solange der Primärweg umsetzbar ist.

## Konsequenzen

- NC-PoRe erhält das lokale Mikrofon-Audio vor Talk-eigener Audioverarbeitung.
- Talk kann seine vorhandene Audio-, Video-, WebRTC- und HPB-Infrastruktur weiterverwenden.
- NC-PoRe kann lokal aufzeichnen, ohne auf serverseitiges Recording angewiesen zu sein.
- Audio und Video können für V1 getrennt behandelt werden.
- Firefox, Chromium und Safari/WebKit werden durch dieselbe standardbasierte Architektur adressiert.
- Die praktische Validierung erfolgt bei der Live-Integration.

## Abgrenzung

Diese ADR legt nicht fest, welche konkrete Talk-Frontend-Klasse oder welches Integrationsmodul den frühen Capture bereitstellt. Sie legt auch weder das Aufnahmecontainerformat noch spätere automatische Synchronisations- oder Driftkorrekturverfahren fest.

Die technische Integrationsstelle muss jedoch die zentrale Entscheidung erfüllen: **Der NC-PoRe-Clone wird unmittelbar am browserseitigen Mikrofon-Capture und vor Talk-eigener Audioverarbeitung erzeugt.**

---

# English Version

## Context

NC-PoRe V1 uses Nextcloud Talk for participant communication. Talk remains responsible for the call, WebRTC, HPB, and audio/video distribution. NC-PoRe must record each participant's own audio locally.

NC-PoRe requires the unprocessed microphone signal. It must not depend on a Talk-processed audio track, a server-side audio path, or a later Talk integration point.

## Decision

For V1, NC-PoRe uses **one browser-side microphone capture and clones the local microphone audio track immediately after it is created by `getUserMedia()`**.

The clone point is therefore **before any Nextcloud Talk audio processing**.

```text
Microphone
   │
   ▼
getUserMedia()
   │
   ▼
shared Media Source
   │
   ├── Talk Audio Track
   │      └── Talk processing → WebRTC / HPB
   │
   └── NC-PoRe Audio Track (Clone)
          └── unprocessed → local PoRE recording
```

Talk and NC-PoRe each receive their own track from the same microphone source. No second independent microphone capture is used.

Talk remains responsible for its own track, processing, WebRTC, and HPB. NC-PoRe uses only its clone for local recording and may process that clone independently.

For V1, the NC-PoRe clone is the raw local microphone recording source, before Talk-specific noise suppression or other Talk processing. Video remains with Talk.

The recording/signeting flow of ADR-068 remains in force: Host Start → eligible clients record locally → clients report `READY` when recording is actually active → after all required clients are ready the Host sends the digital Opening Sync Signet command → each client generates the signet locally into its own NC-PoRe recording.

The target browsers are Firefox, Chromium-based browsers, and Safari/WebKit. V1 relies on standard `MediaStream`/`MediaStreamTrack` semantics, including `MediaStreamTrack.clone()`. A separate clone/synchronization PoC is not a V1 gate; behavior is validated during live integration.

Bit-identical container output is not required. The requirement is that both tracks originate from the same capture source and can be used as independent consumers of that source.

## Integration priority

1. **Primary:** microphone capture → immediate track clone → Talk and NC-PoRe as separate consumers.
2. **Fallback:** NC-PoRe owns the microphone capture and provides Talk with a suitable audio track if the early shared capture cannot be reached cleanly.
3. **Last fallback:** dedicated NC-PoRe browser client if the desired Talk integration cannot be implemented cleanly.

Fallback paths are not pursued while the primary path remains implementable.

## Boundary

The exact Talk frontend integration point is an implementation detail. The architectural requirement is fixed: **the NC-PoRe clone is created immediately at browser microphone capture and before Talk-specific audio processing.**
