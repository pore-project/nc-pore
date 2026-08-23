# ADR-037: Audio Delivery Format and FLAC Default

## Status

Accepted

## Date

2026-08-23

## Decision Type

Architecture / Audio

---

# Deutsch

## Kontext

ADR-002 definiert bereits die Trennung von Aufnahme, Transport und Persistenz und legt fest, dass das Audio-Qualitätsprofil durch den Host bzw. Betreiber einer Session konfigurierbar ist. Die konkrete Codec-/Containerwahl blieb dort bewusst offen.

Die technische Umsetzung hat inzwischen gezeigt, dass NC-PoRE echte Audiodaten zunächst als Capture-Payload übernehmen und anschließend als Recording Artifact persistieren kann. Für die weitere Umsetzung benötigen wir nun eine Default-Entscheidung für Sessions, bei denen der Host kein konkretes Delivery-Format vorgibt.

Dabei soll insbesondere vermieden werden, Audio unnötig verlustfrei zu übertragen, wenn der Host ausdrücklich eine verlustbehaftete Zielqualität verlangt. Umgekehrt sollen Informationen nicht ohne ausdrückliche Vorgabe frühzeitig verloren gehen.

## Entscheidung

1. Das **Audio-Delivery-Format ist eine Host-/Session-Konfiguration**.
2. Gibt der Host ein konkretes Delivery-Format bzw. Qualitätsprofil vor, ist dieses die fachliche Vorgabe für die technische Delivery-Repräsentation.
3. Gibt der Host kein konkretes Delivery-Format vor, verwendet NC-PoRE **FLAC als Default**.
4. FLAC wird als verlustfreies, offenes und komprimierbares Standardformat für den Default-Delivery-Weg verwendet.
5. Eine verlustbehaftete Delivery-Repräsentation (z. B. MP3 mit vorgegebenem Bitrate-Profil) darf auf ausdrückliche Host-Vorgabe bereits vor bzw. während der Delivery-Erzeugung verwendet werden, um unnötiges Datenvolumen und Übertragungszeit zu vermeiden.
6. Die Verwendung eines späteren unkomprimierten oder technisch höherwertigen Containers stellt durch eine vorherige verlustbehaftete Kodierung entfernte Informationen nicht wieder her.
7. **Capture-Format und Delivery-Format werden nicht gleichgesetzt.** Das technische Capture-Format bleibt eine separate Implementierungsentscheidung und darf von der gewünschten Delivery-Repräsentation abweichen.
8. Das tatsächlich erzeugte Delivery-Format muss in den technischen Recording-/Artifact-Metadaten eindeutig beschrieben werden.

## Begründung für FLAC als Default

FLAC erhält die PCM-Audiodaten verlustfrei und reduziert gleichzeitig das Datenvolumen gegenüber unkomprimiertem PCM. Damit bietet es einen sinnvollen Default, wenn der Host keine verlustbehaftete Zielqualität vorgibt.

FLAC erlaubt außerdem eine spätere verlustfreie Rückwandlung nach PCM/WAV. Eine Konvertierung

`FLAC -> WAV`

verliert keine Audioinformationen, sofern das WAV dieselbe Sample-/Kanal-/Bit-Tiefe repräsentieren kann.

Dagegen kann eine Konvertierung

`MP3 64 kbit/s -> WAV`

zwar technisch ein WAV erzeugen, die durch MP3 entfernten Informationen werden dadurch jedoch nicht wiederhergestellt.

## Konsequenzen

- Der Host kann Datenvolumen und Qualität an seinen tatsächlichen Bedarf anpassen.
- FLAC bietet einen sicheren lossless Default, ohne unkomprimierte WAV-Daten erzwingen zu müssen.
- Verlustbehaftete Formate werden nicht grundsätzlich ausgeschlossen, sondern bewusst als Host-Vorgabe behandelt.
- Capture und Delivery können unabhängig weiterentwickelt werden.
- Codec-/Containerimplementierungen müssen weiterhin auf Browser-/Runtime-Unterstützung, CPU-/Speicherbedarf und chunkweise Verarbeitung geprüft werden.

## Nicht entschieden

Diese ADR legt noch nicht fest:

- konkrete Sample-Rate des Capture-Wegs
- konkrete Bit-Tiefe des Capture-Wegs
- Mono-/Stereo-/Mehrkanalstrategie des technischen Capture-Wegs
- konkrete Encoderparameter für FLAC
- konkrete Implementierungen weiterer Delivery-Formate
- welche Host-Qualitätsprofile in einer späteren UI angeboten werden

Diese Punkte werden in den jeweiligen technischen Arbeitsschritten entschieden.

## Beziehung zu ADR-002

ADR-037 konkretisiert die in ADR-002 bewusst offen gelassene technische Codec-/Containerfrage für den Fall ohne explizite Host-Vorgabe. Die grundlegende Trennung von Produktions-/Transportqualität und Archiv-/Persistenzqualität aus ADR-002 bleibt bestehen.

---

# English Version

## Context

ADR-002 already establishes the separation of capture, transport and persistence and defines the audio quality profile as configurable by the session owner/operator. The concrete codec/container choice was intentionally left open.

The technical implementation has now demonstrated that NC-PoRE can accept real captured audio as capture payload and persist it as a Recording Artifact. The implementation therefore needs a default delivery decision for sessions where the host does not specify a concrete delivery format.

The system should avoid transferring lossless audio unnecessarily when the host explicitly requests a lossy target quality. Conversely, information should not be discarded early without an explicit requirement.

## Decision

1. The **audio delivery format is a host/session configuration**.
2. If the host specifies a concrete delivery format or quality profile, that specification is the functional requirement for the delivery representation.
3. If the host does not specify a concrete delivery format, NC-PoRE uses **FLAC as the default**.
4. FLAC is used as the default because it is lossless, open and compressible.
5. A lossy delivery representation (for example MP3 with a specified bitrate profile) may be selected explicitly by the host in order to avoid unnecessary data volume and transfer time.
6. Storing a previously lossy-encoded signal in an uncompressed or technically higher-grade container does not restore information removed by the earlier lossy encoding.
7. **Capture format and delivery format are not the same architectural concept.** The technical capture format remains a separate implementation decision and may differ from the desired delivery representation.
8. The actual delivery format must be explicitly represented in the technical recording/artifact metadata.

## Rationale for FLAC as Default

FLAC preserves PCM audio losslessly while reducing data volume compared with uncompressed PCM. It is therefore a suitable default when the host does not specify a lossy target quality.

FLAC can also be decoded back to PCM/WAV without audio loss, provided the WAV representation supports the same sample rate, channel count and bit depth.

By contrast, converting `MP3 64 kbit/s -> WAV` can produce a technically valid WAV file, but cannot restore information removed by the MP3 encoding step.

## Consequences

- The host can adapt quality and data volume to the actual use case.
- FLAC provides a lossless default without requiring uncompressed WAV transfer.
- Lossy formats remain possible but require an explicit host requirement.
- Capture and delivery can evolve independently.
- Codec/container implementations still need evaluation for browser/runtime support, CPU/memory requirements and chunked processing.

## Not Decided

This ADR does not yet decide:

- the concrete capture sample rate
- the concrete capture bit depth
- the technical mono/stereo/multichannel capture strategy
- concrete FLAC encoder parameters
- concrete implementations of other delivery formats
- which host quality profiles will later be exposed in the UI

These points remain separate technical decisions.

## Relationship to ADR-002

ADR-037 concretizes the codec/container question intentionally left open by ADR-002 for sessions without an explicit host requirement. The fundamental separation between production/transport quality and archive/persistence quality from ADR-002 remains unchanged.
