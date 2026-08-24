# ADR-070: Audio Delivery Format and FLAC Default

## Status

Accepted

## Date

2026-08-23

## Decision Type

Architecture / Audio

## Supersedes

ADR-002 — limited to the audio format and delivery-format decision.

The track concept and other architectural aspects of ADR-002 remain valid unless explicitly superseded by a later decision.

---

# Deutsch

## Kontext

ADR-002 established the separation of capture, transport and persistence and made the audio quality profile configurable by the host or session operator. The concrete codec/container choice was intentionally left open.

The implementation work has now reached the point where NC-PoRE can capture real audio and persist the resulting Recording Artifact. For the next implementation step we need a defined delivery-format policy.

The central requirement is that the host remains in control of the required delivery quality. If a host explicitly requests a lossy target such as MP3 at a defined bitrate, NC-PoRE should not unnecessarily transfer a larger lossless representation first. Conversely, NC-PoRE should not discard audio information merely because no delivery requirement was specified.

## Entscheidung

1. Das **Audio-Delivery-Format ist Bestandteil der Host-/Session-Konfiguration**.
2. Gibt der Host ein konkretes Delivery-Format oder Qualitätsprofil vor, ist diese Vorgabe maßgeblich für die zu erzeugende Delivery-Repräsentation.
3. Gibt der Host kein konkretes Delivery-Format vor, verwendet NC-PoRE **FLAC als Default**.
4. FLAC ist der Default, weil es verlustfrei, offen standardisiert und komprimierbar ist.
5. Verlustbehaftete Delivery-Formate (z. B. MP3 mit vorgegebener Bitrate) sind ausdrücklich zulässig, wenn sie vom Host verlangt werden. In diesem Fall darf NC-PoRE die verlustbehaftete Repräsentation direkt für die Delivery erzeugen, um Datenvolumen und Übertragungszeit zu reduzieren.
6. Eine spätere Konvertierung eines bereits verlustbehafteten Formats in ein unkomprimiertes oder höherwertig erscheinendes Containerformat stellt entfernte Informationen nicht wieder her.
7. **Capture-Format und Delivery-Format sind getrennte Architekturbegriffe.** Das technische Capture-Format wird durch diese ADR nicht festgelegt und darf von der gewünschten Delivery-Repräsentation abweichen.
8. Das tatsächlich erzeugte Delivery-Format wird eindeutig in den technischen Recording-/Artifact-Metadaten beschrieben.

## Warum FLAC als Default

FLAC bewahrt die zugrunde liegenden PCM-Audiodaten verlustfrei und reduziert gegenüber unkomprimiertem PCM das Datenvolumen. Es ist deshalb ein sinnvoller Default, wenn der Host keine verlustbehaftete Zielqualität vorgibt.

Eine Rückwandlung von

`FLAC -> WAV`

kann ohne zusätzlichen Audioverlust erfolgen, sofern das WAV dieselbe Sample-Rate, Kanalzahl und Bit-Tiefe abbilden kann.

Dagegen kann

`MP3 64 kbit/s -> WAV`

zwar technisch ein gültiges WAV erzeugen, die durch die MP3-Kodierung entfernten Informationen sind damit jedoch endgültig verloren.

## Konsequenzen

- Hosts können Qualität, Datenvolumen und Übertragungszeit an ihren konkreten Bedarf anpassen.
- Ohne ausdrückliche Vorgabe erhalten wir einen verlustfreien Default.
- WAV wird nicht zum Default, nur weil es unkomprimiert ist.
- Verlustbehaftete Formate werden nicht ausgeschlossen, sondern bewusst aufgrund einer Host-Vorgabe eingesetzt.
- Capture und Delivery können unabhängig voneinander weiterentwickelt werden.
- Die konkrete Implementierung von FLAC und weiteren Delivery-Formaten bleibt ein nachgelagerter technischer Schritt.

## Nicht durch diese ADR entschieden

Diese ADR legt insbesondere **nicht** fest:

- die Capture-Sample-Rate
- die Capture-Bit-Tiefe
- Mono-, Stereo- oder Mehrkanal-Capture
- das konkrete interne Sample-Format
- konkrete FLAC-Kompressionsparameter
- konkrete Encoder für MP3 oder andere Delivery-Formate
- welche Qualitätsprofile ein Host später in einer Benutzeroberfläche auswählen kann

## Beziehung zu ADR-002

Diese ADR ersetzt ADR-002 **hinsichtlich der dort offen gelassenen Audioformat-/Delivery-Format-Entscheidung**.

Die in ADR-002 definierte Trennung von Capture, Transport und Persistenz sowie das dort beschriebene Track-Konzept bleiben bestehen.

---

# English Version

## Context

ADR-002 established the separation of capture, transport and persistence and made the audio quality profile configurable by the host or session operator. The concrete codec/container choice was intentionally left open.

Implementation work has now reached the point where NC-PoRE can capture real audio and persist the resulting Recording Artifact. The next implementation step therefore requires a defined delivery-format policy.

The central requirement is that the host remains in control of the required delivery quality. If a host explicitly requests a lossy target such as MP3 at a defined bitrate, NC-PoRE should not unnecessarily transfer a larger lossless representation first. Conversely, NC-PoRE should not discard audio information merely because no delivery requirement was specified.

## Decision

1. The **audio delivery format is part of the host/session configuration**.
2. If the host specifies a concrete delivery format or quality profile, that specification governs the delivery representation to be produced.
3. If the host does not specify a concrete delivery format, NC-PoRE uses **FLAC as the default**.
4. FLAC is the default because it is lossless, openly standardized and compressible.
5. Lossy delivery formats (for example MP3 with a specified bitrate) are explicitly allowed when requested by the host. In that case NC-PoRE may generate the lossy representation directly for delivery in order to reduce data volume and transfer time.
6. Converting an already lossy representation into an uncompressed or apparently higher-grade container does not restore information removed by the earlier lossy encoding.
7. **Capture format and delivery format are separate architectural concepts.** This ADR does not define the technical capture format, which may differ from the desired delivery representation.
8. The actual delivery format must be explicitly represented in the technical recording/artifact metadata.

## Why FLAC as Default

FLAC preserves the underlying PCM audio losslessly while reducing data volume compared with uncompressed PCM. It is therefore a suitable default when the host does not specify a lossy target quality.

A conversion from

`FLAC -> WAV`

can be performed without additional audio loss, provided the WAV representation supports the same sample rate, channel count and bit depth.

By contrast,

`MP3 64 kbit/s -> WAV`

can produce a technically valid WAV file, but information removed by the MP3 encoding step is permanently lost.

## Consequences

- Hosts can adapt quality, data volume and transfer time to their actual requirements.
- Without an explicit host requirement, the system has a lossless default.
- WAV is not the default merely because it is uncompressed.
- Lossy formats remain available, but are deliberately selected based on an explicit host requirement.
- Capture and delivery can evolve independently.
- Concrete FLAC and other delivery-format implementations remain a subsequent technical step.

## Not Decided by this ADR

This ADR does not decide:

- capture sample rate
- capture bit depth
- mono, stereo or multichannel capture
- concrete internal sample format
- concrete FLAC compression parameters
- concrete encoders for MP3 or other delivery formats
- which quality profiles hosts will later be able to select in a user interface

## Relationship to ADR-002

This ADR supersedes ADR-002 **with regard to the audio-format and delivery-format decision that ADR-002 intentionally left open**.

The separation of capture, transport and persistence established by ADR-002, as well as its track concept, remains in force.
