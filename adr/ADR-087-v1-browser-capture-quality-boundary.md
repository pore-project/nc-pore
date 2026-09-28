# ADR-087 V1 Browser Capture Quality Boundary

- Status: Accepted
- Date: 2026-09-27
- Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))
## Kontext

NC-PoRe soll die lokale Aufnahmequalität unabhängig von der Kommunikations- oder Host-Pipeline bewahren. Der Browser stellt dafür die konkrete Capture-Quelle bereit. Diese Quelle kann sich hinsichtlich Sample-Rate, Kanalzahl und Audio-Processing je nach Browser, Betriebssystem, Gerät und Laufzeitumgebung unterscheiden.

Für V1 ist deshalb eine eindeutige technische Grenze erforderlich: PoRE muss einen eigenen lokalen Preservation-Master erzeugen und darf weder eine Host-/Talk-Kommunikationsspur noch unbemerkt durch die Web-Audio-Laufzeit veränderte Kanal- oder Sample-Rate-Eigenschaften als unveränderte Capture-Wahrheit ausgeben.

## Entscheidung

### 1. PoRE besitzt den lokalen Preservation-Master

Der PoRE-Browserpfad verwendet die von PoRE angeforderte und erhaltene lokale `MediaStreamTrack` als Ausgangspunkt für den Preservation-Pfad.

Die Kommunikations- bzw. Host-Pipeline ist nicht der Preservation-Master.

Die Architektur bleibt:

```text
Mikrofon
   |
   v
PoRE Local Capture (Master)
   |
   +----> Preservation
   |
   +----> Clone/Adapter für Host-Kommunikation
```

### 2. V1 Capture-Constraints sind Präferenzen, keine unbegründeten Garantien

PoRE fordert für den lokalen Capture-Pfad mindestens:

```text
echoCancellation: false
noiseSuppression: false
autoGainControl: false
channelCount: { ideal: 1 }
sampleRate: { ideal: 48000 }
```

`ideal` wird bewusst verwendet, weil die lokale Capture-Umgebung diese Werte nicht in jedem Fall exakt bereitstellen muss.

Die tatsächlich von der Browser-API gemeldeten Capture-Eigenschaften sind maßgeblich, soweit sie verfügbar und belastbar auslesbar sind. Nicht verfügbare Eigenschaften werden nicht durch Annahmen ersetzt.

### 3. Aktives Browser-Audio-Processing disqualifiziert den Preservation-Master

Nach erfolgreichem `getUserMedia()` prüft PoRE die tatsächlich gemeldeten Track-Einstellungen.

Wenn der Browser ausdrücklich meldet:

```text
echoCancellation === true
noiseSuppression === true
autoGainControl === true
```

darf diese Spur nicht als Preservation-Master verwendet werden.

PoRE beendet in diesem Fall den erhaltenen Capture-Stream kontrolliert und meldet den Capture-Fehler weiter.

Ein nicht verfügbarer Wert wird dagegen nicht automatisch als `true` oder `false` interpretiert.

### 4. Mono wird an der Preservation-Grenze tatsächlich hergestellt

Der Preservation-`AudioWorkletNode` wird mit:

```text
channelCount: 1
channelCountMode: "explicit"
channelInterpretation: "speakers"
```

konfiguriert.

Damit besitzt die Preservation-Kette genau einen Eingangskanal.

Eine mehrkanalige Quelle darf nicht dadurch zu Mono werden, dass das Worklet stillschweigend nur Kanal 0 liest. Die Kanalreduktion erfolgt vorher über die standardisierte Speaker-Downmix-Regel der Web Audio API.

Für den relevanten Stereo→Mono-Fall ist die standardisierte Mischung:

```text
mono = 0.5 * (left + right)
```

Damit bleiben beide vorhandenen Stereo-Kanäle Bestandteil der Mono-Preservation.

### 5. Die tatsächliche bekannte Capture-Sample-Rate bestimmt den AudioContext

Ein `AudioContext` besitzt eine feste Sample-Rate. Wenn die Sample-Rate eines eingebundenen `MediaStreamTrack` von der Context-Rate abweicht, kann die Web Audio API das Track-Signal auf die Context-Rate resamplen.

Um eine unbeabsichtigte Konvertierung zu vermeiden, verwendet PoRE für eine bekannte, gültige Track-Sample-Rate dieselbe Rate beim Erzeugen des Preservation-`AudioContext`.

Daraus folgt:

```text
bekannte Track-Rate
      |
      v
AudioContext(sampleRate = Track-Rate)
      |
      v
Preservation
```

Für den V1-Qualitätsdefault gilt:

- 48 kHz ist die bevorzugte Capture-Anforderung.
- 44,1 kHz ist ein zulässiger realer Capture-Fallback.
- Eine tatsächlich gelieferte 44,1-kHz-Quelle wird nicht allein deshalb auf 48 kHz resampelt, um den Produktdefault nachträglich zu erzwingen.

Wenn der Browser die tatsächliche Track-Sample-Rate nicht verfügbar macht, kann PoRE keine stärkere Aussage treffen als die API zulässt. Dieser Fall bleibt Teil der realen Browser-/Runtime-Validierung.

### 6. Ein Sample-Rate-Wechsel während derselben Preservation-Aufnahme wird nicht still konvertiert

Wenn während einer laufenden Aufnahme eine neue Capture-Quelle übernommen werden soll und deren bekannte Sample-Rate von der bereits verwendeten Preservation-Sample-Rate abweicht, wird dieser Wechsel nicht stillschweigend in den bestehenden AudioContext eingespeist.

Der Wechsel wird abgelehnt.

Damit entsteht keine unbemerkte Sample-Rate-Konvertierung innerhalb eines bereits laufenden Preservation-Artefakts.

Ein solcher Quellwechsel muss von der darüberliegenden Aufnahmelogik als technischer Fehler behandelt werden; der bestehende gültige Capture-Pfad bleibt davon unberührt.

### 7. V1-Preservation-Repräsentation

Die V1-Preservation-Repräsentation ist:

```text
PCM
Mono
24 Bit
48 kHz bevorzugt
44,1 kHz bei tatsächlichem Capture-Fallback zulässig
```

Die Angabe `24 Bit` beschreibt die Preservation-Repräsentation des erzeugten PCM-Artefakts.

Sie ist kein Nachweis, dass Mikrofon, Betriebssystem oder Browser tatsächlich mit nativer 24-Bit-Quantisierung geliefert haben.

Die technischen Capture-Eigenschaften und die Preservation-Eigenschaften bleiben daher getrennte Fakten.

## Konsequenzen

- Der PoRE-Master bleibt unabhängig von der Host-Kommunikationspipeline.
- Mono wird nicht durch stilles Verwerfen des rechten Kanals erzeugt.
- Bekannte Sample-Rate-Unterschiede zwischen Capture und Preservation werden nicht unbemerkt in das bestehende Artefakt resampelt.
- Tatsächlich aktive Browser-Verarbeitung wird nicht als unverarbeitete Preservation ausgegeben.
- Die V1-Preservation bleibt ein nachvollziehbares 24-Bit-PCM-Mono-Artefakt.
- Browser- und Laufzeitunterschiede bleiben reale technische Fakten und müssen durch Runtime-Tests überprüft werden.

## Testing Consequences

Die technische Grenze muss mindestens prüfen:

- angeforderte `getUserMedia()`-Constraints;
- explizit aktive EC/NS/AGC-Einstellungen;
- tatsächliche bekannte Sample-Rate;
- AudioContext-Sample-Rate;
- Preservation-Kanalzahl;
- Downmix-Verhalten für mehrkanalige Quellen;
- Verhalten bei einem bekannten Sample-Rate-Wechsel;
- tatsächliche Preservation-Repräsentation im erzeugten WAV-Artefakt.

Unit- und Contract-Tests können die Konstruktion und Zustandsübergänge deterministisch prüfen. Die tatsächliche Browser-Unterstützung der relevanten MediaTrack-Eigenschaften bleibt eine Runtime-Frage.

## Relationship to Existing Architecture

Diese Entscheidung konkretisiert insbesondere:

- ADR-071 Recording Capture, Preservation and Transport Formats
- ADR-086 Talk Recording Control Surface V1

Die Entscheidung verändert nicht die Trennung zwischen Capture, Preservation und Transport. Sie präzisiert vielmehr die technische Grenze des V1-Browser-Capturepfads.

## Status

Diese Entscheidung gilt für den V1-Browser-Capturepfad als angenommen.

---

# English Version ([Deutsche Version oben](#deutsch))
## Context

NC-PoRe must preserve local recording quality independently of the communication or host pipeline. The browser provides the concrete capture source, whose sample rate, channel count and audio-processing settings may vary across browsers, operating systems, devices and runtime environments.

V1 therefore requires an explicit technical boundary: PoRE must create its own local preservation master and must not treat a host/Talk communication track, or an unnoticed Web Audio conversion, as unchanged capture truth.

## Decision

### 1. PoRE owns the local preservation master

The PoRE browser path uses the local `MediaStreamTrack` requested and obtained by PoRE as the source of the preservation path.

The communication/host pipeline is not the preservation master.

The architecture is:

```text
microphone
    |
    v
PoRE local capture (master)
    |
    +----> preservation
    |
    +----> clone/adapter for host communication
```

### 2. V1 capture constraints are preferences, not unsupported guarantees

The local capture path requests at least:

```text
echoCancellation: false
noiseSuppression: false
autoGainControl: false
channelCount: { ideal: 1 }
sampleRate: { ideal: 48000 }
```

`ideal` is intentional because the local capture environment is not required to provide every requested value exactly.

Actual properties reported by the browser are authoritative where they are reliably exposed. Unavailable properties are not replaced by assumptions.

### 3. Active browser audio processing disqualifies the preservation master

After `getUserMedia()`, PoRE checks the actual track settings.

If the browser explicitly reports any of:

```text
echoCancellation === true
noiseSuppression === true
autoGainControl === true
```

the track must not be used as the preservation master.

PoRE stops the obtained capture stream in this case and reports the capture error.

An unavailable value is not treated as either `true` or `false`.

### 4. Mono is established at the preservation boundary

The preservation `AudioWorkletNode` is configured with:

```text
channelCount: 1
channelCountMode: "explicit"
channelInterpretation: "speakers"
```

The preservation chain therefore has exactly one input channel.

A multichannel source must not become mono by silently keeping only channel 0. Channel reduction occurs first through the Web Audio API's standard speaker downmix.

For the relevant stereo-to-mono case, the standard mix is:

```text
mono = 0.5 * (left + right)
```

Thus both stereo channels remain part of the mono preservation signal.

### 5. The actual known capture rate determines the AudioContext rate

An `AudioContext` has a fixed sample rate. When an attached `MediaStreamTrack` has a different sample rate, the Web Audio API may resample the track signal to the context rate.

To avoid an unintended conversion, PoRE uses a known valid track sample rate when creating the preservation `AudioContext`.

The resulting boundary is:

```text
known track rate
      |
      v
AudioContext(sampleRate = track rate)
      |
      v
preservation
```

For the V1 quality default:

- 48 kHz is the preferred capture request.
- 44.1 kHz is an allowed real capture fallback.
- An actual 44.1-kHz source is not resampled to 48 kHz merely to force the product default after capture.

If the browser does not expose the actual track sample rate, PoRE cannot make a stronger claim than the API permits. That case remains a real browser/runtime validation concern.

### 6. A sample-rate change during one preservation recording is not silently converted

If a new capture source is introduced during an active recording and its known sample rate differs from the preservation sample rate already in use, PoRE does not silently feed it into the existing AudioContext.

The replacement is rejected.

This prevents an unnoticed sample-rate conversion inside an already-running preservation artifact.

### 7. V1 preservation representation

The V1 preservation representation is:

```text
PCM
mono
24 bit
48 kHz preferred
44.1 kHz permitted for an actual capture fallback
```

`24 bit` describes the preservation representation of the generated PCM artifact.

It is not evidence that the microphone, operating system or browser delivered native 24-bit quantization.

Capture properties and preservation properties therefore remain separate facts.

## Consequences

- The PoRE master remains independent of the host communication pipeline.
- Mono is not produced by silently discarding the right channel.
- Known sample-rate differences are not silently resampled into an existing preservation artifact.
- Explicitly active browser processing is not represented as unprocessed preservation.
- V1 preservation remains a traceable 24-bit PCM mono artifact.
- Browser/runtime differences remain real technical facts and must be validated at runtime.

## Testing Consequences

The boundary must test at least:

- requested `getUserMedia()` constraints;
- explicitly active EC/NS/AGC settings;
- actual known sample rate;
- AudioContext sample rate;
- preservation channel count;
- multichannel downmix behavior;
- behavior for a known sample-rate change;
- actual WAV preservation representation.

Unit and contract tests can verify construction and state transitions deterministically. Actual browser support for the relevant MediaTrack properties remains a runtime concern.

## Relationship to Existing Architecture

This decision concretizes in particular:

- ADR-071 Recording Capture, Preservation and Transport Formats
- ADR-086 Talk Recording Control Surface V1

It does not change the separation between capture, preservation and transport. It specifies the technical boundary of the V1 browser capture path.

## Status

This decision is accepted for the V1 browser capture path.
