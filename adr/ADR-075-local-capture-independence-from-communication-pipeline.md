# ADR-075: Local Capture Independence from Communication Pipeline

## Status

Proposed

## Date

2026-08-30

---

# Deutsch

## Kontext

NC-PoRe kann als Aufnahmekomponente innerhalb einer Kommunikations- oder Konferenzanwendung eingesetzt werden. Eine solche Anwendung optimiert ihren Audioweg für die Kommunikation. Dazu können unter anderem Sprachverarbeitung, Rauschunterdrückung, automatische Pegelanpassung und verlustbehaftete Übertragung gehören.

Diese Verarbeitung ist für die Kommunikation sinnvoll, definiert aber nicht die gewünschte Qualität des PoRE-Aufnahmematerials.

Für professionelle Produktion und mögliche Weitergabe an externe Abnehmer soll PoRE das lokal verfügbare Eingangssignal so hochwertig wie technisch möglich erhalten. Eine nachträgliche Umwandlung eines zuvor verlustbehaftet verarbeiteten Signals in ein unkomprimiertes Format stellt die verlorene Quellinformation nicht wieder her.

Die konkrete Kommunikationsanwendung ist dabei nicht Teil des allgemeinen Architekturprinzips. Unterschiedliche Hosts können unterschiedliche Media-Pipelines und Qualitätsanforderungen für ihre Kommunikation verwenden.

## Entscheidung

NC-PoRe behandelt **Kommunikationspipeline und Aufnahmepipeline als getrennte Verantwortungsbereiche**.

Wenn eine lokale Eingangsquelle sowohl für eine Kommunikationsanwendung als auch für PoRE verwendet wird, soll PoRE die lokale Quelle möglichst **vor hostseitiger Kommunikationsverarbeitung** und unabhängig vom für die Kommunikation verwendeten Codec oder Transportweg erfassen.

PoRE zeichnet mit der **bestmöglichen tatsächlich verfügbaren Quellqualität** auf. Die technischen Fähigkeiten der konkreten Capture-Quelle sind maßgeblich; PoRE darf keine höhere Quellqualität durch künstliche Parameter oder nachträgliche Konvertierung vortäuschen.

Die Kommunikationsanwendung darf ihre eigene Kopie bzw. ihren eigenen Media-Pfad unabhängig optimieren, komprimieren, verarbeiten und übertragen. Diese Verarbeitung darf die PoRE-Aufnahme nicht verschlechtern.

Die konkrete technische Umsetzung der lokalen Capture-Grenze bleibt Host- und Plattform-spezifisch. Das allgemeine Prinzip gilt unabhängig davon, ob der Host beispielsweise Nextcloud Talk, Jitsi, BigBlueButton oder eine andere Kommunikationsanwendung verwendet.

## Gerätefähigkeiten und Quellenwechsel

PoRE muss die tatsächlich verfügbaren Fähigkeiten der lokalen Capture-Quelle berücksichtigen, insbesondere unterstützte Samplingraten, Kanalzahl und weitere für das gewählte Audio-Qualitätsprofil relevante Eigenschaften.

Ein Wechsel der lokalen Eingangsquelle während einer laufenden Aufnahme darf nicht unbemerkt als unveränderte Quelle behandelt werden. Die Integration muss einen solchen Wechsel erkennen und den bestehenden Recording-/Artifact-Lifecycle entsprechend behandeln.

Die konkrete Modellierung eines Quellenwechsels als neuer Track, Segment oder andere neutrale Artifact-Einheit wird durch die jeweilige technische Implementierung festgelegt, muss aber die Herkunft und zeitliche Grenze des Materials erhalten.

## Abgrenzung

Diese ADR schreibt keinen bestimmten Codec oder Container vor. Insbesondere wird weder WAV noch FLAC noch Opus oder ein anderes Format als projektweites Masterformat durch diese Entscheidung festgelegt.

Sie definiert auch nicht die Qualität der Kommunikationsausgabe. Kommunikationsqualität ist Aufgabe des jeweiligen Hosts; PoRE-Aufnahmequalität ist Aufgabe des PoRE-Capture-Pfades.

## Konsequenzen

### Positive Auswirkungen

- Kommunikationsoptimierung kann unabhängig von der PoRE-Aufnahme erfolgen.
- Verlustbehaftete Kommunikationscodecs müssen nicht zum Aufnahme-Master werden.
- Die bestmögliche lokale Quellqualität kann für professionelle Produktion erhalten werden.
- Das Prinzip ist auf unterschiedliche Host-Anwendungen übertragbar.
- Gerätewechsel können als Qualitäts- und Herkunftsgrenze behandelt werden.
- Die bestehende Trennung von Capture, RecordingArtifact und späterer Produktion bleibt erhalten.

### Negative Auswirkungen

- Eine zusätzliche lokale Capture-Integration kann erforderlich sein.
- Host-spezifische Media-Lifecycles müssen untersucht werden.
- Gerätefähigkeiten und Quellenwechsel müssen zuverlässig erkannt und behandelt werden.
- Die lokale Capture-Pipeline kann technisch komplexer sein als die Übernahme eines bereits verarbeiteten Kommunikations-Tracks.

## Betrachtete Alternativen

### Aufzeichnung des bereits verarbeiteten Kommunikations-Tracks

Verworfen als professioneller PoRE-Masterpfad.

Der Kommunikations-Track kann bereits durch Host-spezifische Verarbeitung und verlustbehaftete Codierung verändert worden sein. Eine spätere Speicherung als WAV oder in einem anderen unkomprimierten Format stellt entfernte Quellinformation nicht wieder her.

### Gemeinsame Capture-Pipeline für Kommunikation und PoRE

Verworfen.

Die Kommunikationsanwendung soll ihre Medien für ihre eigene Aufgabe optimieren dürfen, ohne dadurch die Aufnahmequalität von PoRE zu begrenzen.

### Host-spezifische Sonderregel für eine einzelne Konferenzanwendung

Verworfen.

Die aus einer konkreten Integration gewonnenen Erkenntnisse sollen als allgemeines PoRE-Prinzip gelten. Einzelne Hosts erhalten eigene Connectoren, müssen sich aber an dieselbe neutrale Capture-Verantwortungsgrenze halten.

## Hinweise

Die Entscheidung konkretisiert die Grundsätze aus ADR-001 zur lokalen Aufnahme sowie ADR-002 zur Trennung von Produktions-/Transportqualität und Archiv-/Persistenzqualität.

Eine Kommunikationsanwendung ist damit nicht automatisch eine Aufnahmekomponente. Sie kann Kommunikationsaudio in einer für den Gesprächsbetrieb geeigneten Qualität erzeugen, während PoRE parallel das lokale Eingangssignal für die Produktion erhält.

---

# English Version

## Context

NC-PoRe may be embedded into a communication or conferencing application. Such an application optimizes its audio path for communication. This may include speech processing, noise suppression, automatic gain control, and lossy transmission.

Such processing can be appropriate for communication but does not define the desired quality of PoRE recording material.

For professional production and possible delivery to external recipients, PoRE shall preserve the locally available input signal at the highest technically achievable quality. Converting a previously lossy-processed signal into an uncompressed format does not restore lost source information.

The concrete communication application is not part of this general architectural principle. Different hosts may use different media pipelines and communication-quality requirements.

## Decision

NC-PoRe treats the **communication pipeline and the recording pipeline as separate responsibilities**.

When a local input source is used by both a communication application and PoRE, PoRE shall capture the local source as independently as technically possible **before host-side communication processing**, without depending on the codec or transport path used for communication.

PoRE records at the **best actual source quality available**. The real capabilities of the capture source are authoritative; PoRE must not imply higher source quality through artificial parameters or later conversion.

The communication application may independently optimize, process, compress, and transmit its own copy or media path. Such processing must not degrade the PoRE recording.

The concrete implementation of the local capture boundary remains host- and platform-specific. The principle applies regardless of whether the host is Nextcloud Talk, Jitsi, BigBlueButton, or another communication application.

## Device Capabilities and Source Changes

PoRE must account for the actual capabilities of the local capture source, including supported sample rates, channel count, and other properties relevant to the selected audio quality profile.

A change of the local input source during an active recording must not be silently treated as the unchanged source. The integration must detect such a change and handle the existing recording/artifact lifecycle accordingly.

The concrete modeling of a source change as a new track, segment, or another neutral artifact unit is an implementation decision, but source provenance and the temporal boundary of the material must be preserved.

## Scope and Boundaries

This ADR does not prescribe a specific codec or container. In particular, it does not establish WAV, FLAC, Opus, or any other format as a project-wide master format.

It also does not define communication output quality. Communication quality is the responsibility of the respective host; PoRE recording quality is the responsibility of the PoRE capture path.

## Consequences

### Positive Effects

- Communication processing can operate independently of PoRE recording.
- Lossy communication codecs do not have to become the recording master.
- The best locally available source quality can be preserved for professional production.
- The principle applies to different host applications.
- Device changes can be treated as quality and provenance boundaries.
- The existing separation between capture, RecordingArtifact, and later production remains intact.

### Negative Effects

- An additional local capture integration may be required.
- Host-specific media lifecycles must be investigated.
- Device capabilities and source changes must be detected and handled reliably.
- The local capture pipeline can be technically more complex than recording an already processed communication track.

## Alternatives Considered

### Recording the Already-Processed Communication Track

Rejected as the professional PoRE master path.

The communication track may already have been modified by host-specific processing and lossy encoding. Storing it later as WAV or another uncompressed format does not restore removed source information.

### Shared Capture Pipeline for Communication and PoRE

Rejected.

The communication application must be free to optimize its media for its own purpose without limiting PoRE recording quality.

### Host-Specific Special Rule for a Single Conferencing Application

Rejected.

Findings from a concrete integration should become a general PoRE principle. Individual hosts receive dedicated connectors but must follow the same neutral capture responsibility boundary.

## Notes

This decision refines the principles from ADR-001 on local recording and ADR-002 on the separation of production/transport quality and archive/persistence quality.

A communication application is therefore not automatically a recording component. It may produce communication audio suitable for conversation while PoRE independently captures the local input signal for production.
