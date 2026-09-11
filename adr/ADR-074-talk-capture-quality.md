# ADR-074: Talk Capture Quality Boundary

- Status: Accepted
- Date: 2026-08-30
- Scope: V1 Talk integration

---

<a id="deutsch"></a>

# Deutsch

## Kontext

NC-PoRe ist für professionelle Podcast-Produktion und für Aufnahmen vorgesehen, die an Rundfunkanstalten oder andere nachgelagerte Nutzer weitergegeben werden können. Der Recording-Pfad soll das Eingangssignal daher so originalgetreu wie technisch möglich erhalten, vorzugsweise verlustfrei, statt lediglich verständliche Sprache für eine Kommunikation zu erzeugen.

Nextcloud Talk ist eine Kommunikationsschicht. Sein WebRTC-Audiopfad kann Browser-/WebRTC-Audioverarbeitung anwenden und transportiert die Gesprächsaudio-Daten über einen verlustbehafteten Codec wie Opus. Ein von Talk bereitgestellter `MediaStreamTrack` kann daher nicht als gleichwertiger Ersatz für einen nativen PoRE-Capture-Pfad behandelt werden: Eine Codec-Konvertierung kann bereits verworfene Informationen nicht wiederherstellen, und die Qualität von Mikrofon und Aufnahmeumgebung bleibt eine begrenzende Größe.

Der V1-Prototyp hat gezeigt, dass PoRE den von Talk bereitgestellten Audiotrack aufnehmen kann und dabei einen gültigen 48-kHz-Stereo-Opus-Stream erzeugt. Die erzeugte Datei war decodierbar, ihre Qualitätsmerkmale entsprechen jedoch dem Talk-/WebRTC-Pfad und nicht einer verlustfreien PoRE-Masteraufnahme.

## Entscheidung

PoRE trennt Kommunikation und Aufnahme.

- Talk bleibt für den Kommunikationsstream verantwortlich.
- PoRE darf den von Talk/WebRTC kodierten Ausgang nicht als professionellen Recording-Master verwenden, wenn ein direkter lokaler Capture-Pfad verfügbar ist.
- PoRE soll die lokale Eingangsquelle unabhängig und vor der WebRTC-Verarbeitung und dem Codec-Pfad von Talk erfassen und dabei die beste tatsächlich vom gewählten Gerät unterstützte Qualität verwenden.
- Der native PoRE-Capture bleibt der Referenz-/Master-Recording-Pfad und soll das Eingangssignal so originalgetreu wie technisch möglich, vorzugsweise verlustfrei, erhalten.
- Talk-Capture kann als Kompatibilitäts- oder Komfortpfad verfügbar bleiben, darf aber nicht als qualitativ gleichwertig mit nativem PoRE-Capture dargestellt werden.

## Gerätefähigkeiten

Vor der Aufnahme soll PoRE die Fähigkeiten des gewählten Eingabegeräts ermitteln, statt ein festes Aufnahmeformat anzunehmen. Die Capture-Konfiguration soll die höchste geeignete Qualität verwenden, die das Gerät tatsächlich unterstützt.

PoRE darf keine Qualitätsmerkmale erzeugen, die die Quelle nicht bereitstellt. Eine Monoquelle darf beispielsweise nicht allein durch Kopieren des Kanals als echte Stereoquelle dargestellt werden.

## Gerätewechsel während der Aufnahme

Änderungen des Eingabegeräts sind Bestandteil des Recording-Lifecycles und müssen ausdrücklich behandelt werden.

Ein Wechsel vor Beginn der Aufnahme wählt einfach das neue Eingabegerät für die Aufnahme.

Ein Wechsel während einer aktiven Aufnahme darf die Quelle nicht stillschweigend ersetzen, während gleichzeitig der Eindruck entsteht, die vollständige Aufnahme stamme von einem unveränderten Gerät. Der Wechsel soll eine technische Capture-Grenze erzeugen, sodass das resultierende RecordingArtifact den Quellenwechsel darstellen und die Herkunft erhalten kann.

Das bestehende RecordingArtifact-Modell mit Tracks/Chunks und Sample-Offsets ist der vorgesehene Ort, um solche Capture-Kontinuität und -Grenzen abzubilden.

## Konsequenzen

Diese Entscheidung akzeptiert bewusst, dass ein entfernter Talk-Teilnehmer möglicherweise nicht dieselbe technische Qualität liefert wie ein lokal aufgenommenes professionelles Mikrofon. PoRE kann das beste Signal erhalten, das das tatsächliche Eingabegerät des Teilnehmers liefert, kann aber keine Informationen rekonstruieren, die durch ungeeignete Hardware, Raumakustik, Browserverarbeitung oder verlustbehaftete Netzwerkübertragung verloren gegangen sind.

Die Entscheidung vermeidet außerdem eine nachträgliche WebM/Opus-zu-WAV-Konvertierung allein mit dem Ziel, einen verlustbehafteten Talk-Stream wie einen verlustfreien Master erscheinen zu lassen. Eine solche Konvertierung ändert Container bzw. Repräsentation, nicht den zugrunde liegenden Informationsgehalt.

## Nicht-Ziele

- den Kommunikationscodec von Talk zu ersetzen
- WebRTC/Opus als verlustfreies Aufnahmeformat zu behandeln
- unabhängig von Mikrofon oder Aufnahmeumgebung eine Broadcast-Qualität zu versprechen

---

<a id="english-version"></a>

# English Version

## Context

NC-PoRE is intended for professional podcast production and for recording material that may be delivered to broadcasters or other downstream users. The recording path therefore aims to preserve the input signal as faithfully as technically possible, preferably losslessly, rather than merely producing speech that is intelligible in a call.

Nextcloud Talk is a communication layer. Its WebRTC audio path may apply browser/WebRTC audio processing and transports call audio using a lossy codec such as Opus. A Talk-provided `MediaStreamTrack` therefore cannot be treated as an equivalent substitute for a native PoRE capture path: codec conversion cannot restore information already discarded upstream, and microphone and recording-environment quality remain limiting factors.

The V1 prototype demonstrated that PoRE can capture the Talk-provided audio track, producing a valid 48 kHz stereo Opus stream. The resulting file was decodable, but its quality characteristics are those of the Talk/WebRTC path rather than those of a lossless PoRE master recording.

## Decision

PoRE separates communication from recording.

- Talk remains responsible for the communication stream.
- PoRE must not use the Talk/WebRTC encoded output as its professional recording master when a direct local capture path is available.
- PoRE should capture the local input source independently, before Talk's WebRTC processing and codec path, using the best quality actually supported by the selected device.
- Native PoRE capture remains the reference/master recording path and should preserve the input signal as faithfully as technically possible, preferably losslessly.
- Talk capture may remain available as a compatibility or convenience path, but must not be represented as equivalent in quality to native PoRE capture.

## Device Capabilities

Before recording, PoRE should determine the capabilities of the selected input device rather than assuming a fixed recording format. The capture configuration should use the highest appropriate quality that the device actually supports.

PoRE must not manufacture quality characteristics that the source device does not provide. For example, a mono source must not be represented as a genuinely stereo source merely by duplicating channels.

## Device Changes During Recording

Input-device changes are part of the recording lifecycle and must be handled explicitly.

A change before recording starts simply selects the new input device for the recording.

A change during an active recording must not silently replace the source while pretending that the complete recording came from one unchanged device. The change should create a technical capture boundary so that the resulting RecordingArtifact can represent the source transition and preserve provenance.

The existing RecordingArtifact model, with tracks/chunks and sample offsets, is the intended place to represent such capture continuity and boundaries.

## Consequences

This decision deliberately accepts that a remote Talk participant may not provide the same technical quality as a locally captured professional microphone. PoRE can preserve the best signal available from that participant's actual input device, but cannot reconstruct information lost through poor hardware, room acoustics, browser processing, or lossy network encoding.

The decision also avoids adding a post-hoc WebM/Opus-to-WAV conversion merely to make a lossy Talk stream look like a lossless master. Such conversion changes the container/representation, not the underlying information content.

## Non-goals

- Replacing Talk's communication codec.
- Treating WebRTC/Opus as a lossless recording format.
- Promising broadcast-grade quality regardless of the participant's microphone or recording environment.
