# Deutsch ([English version below](#english-version))

# ADR-064: Reconstructable Capture Artifacts

## Status

Proposed

## Date

2026-08-17

## Decision Type

Architecture

---

# Kontext

Eine Aufnahme auf dem Endgerät eines Teilnehmers kann durch Netzwerkausfälle, Browser- oder Anwendungsabstürze, Geräteprobleme oder einen vorübergehenden Verlust der Verbindung unterbrochen werden.

Der Capture-Prozess kann daher nicht verlangen, zum Zeitpunkt der Aufnahme bereits eine perfekte fertige Mediendatei zu erzeugen.

Der Vergleich mit Ennuicastr macht ein nützliches Prinzip sichtbar: Aufgenommene Daten sollten ausreichend vollständig und zeitlich eindeutig zuordenbar bleiben, damit ein konsistenter Track später rekonstruiert werden kann.

NC-PoRe muss außerdem Quelldaten des Captures von abgeleiteten Produktionsausgaben unterscheiden.

---

# Entscheidung

NC-PoRe behandelt **Roh-Capture-Daten als First-Class Artifact** eines Recordings.

Das Raw Capture Artifact ist die rekonstruierbare Quelle, aus der spätere Production Artifacts rekonstruiert oder neu erzeugt werden können.

Capture-Daten müssen:

* ausreichend vollständig sein, um das aufgezeichnete Material rekonstruieren zu können
* Participant und Session eindeutig zugeordnet sein
* eine zeitliche Positionierung besitzen
* bei Bedarf auf Chunk-/Segmentebene identifizierbar sein
* zwischen vollständigem und unvollständigem Capture-Zustand unterscheiden können
* zwischen vollständigem und beschädigtem Capture-Zustand unterscheiden können
* ihre Integrität auf Chunk-Ebene verifizieren können
* die Integrität des gesamten Artifacts verifizierbar machen
* unabhängig von abgeleiteten Processing-Ausgaben gemäß Storage Policy erhalten werden können

Ein Capture Artifact muss keine fertige WAV-, AIFF-, FLAC- oder andere Produktionsdatei sein.

---

# Architekturprinzip

> Capture muss nicht perfekt sein. Es muss vollständig und rekonstruierbar genug sein.

Processing ist dafür verantwortlich, aus rekonstruierten Capture-Daten Production Artifacts zu erzeugen.

---

# Rejoin und unterbrochene Streams

Eine Unterbrechung darf einen gesamten Participant Track nicht automatisch unbrauchbar machen.

Wo dies technisch und fachlich möglich ist, kann ein nachfolgendes Capture-Segment desselben Participants und derselben Session dem bestehenden Track zugeordnet und zu einer konsistenten Timeline rekonstruiert werden.

Die genaue Rejoin-Semantik bleibt von der Synchronisationsentscheidung und späteren Implementierungsentscheidungen abhängig.

---

# Artifact Layers

Die vorgesehene konzeptionelle Trennung lautet:

```text
Recording
   |
   +-- Raw Capture Artifact(s)
   |       |
   |       +-- chunks / segments
   |       +-- timing information
   |       +-- capture metadata
   |
   +-- Derived Production Artifact(s)
           |
           +-- lossless master candidates
           +-- exports
           +-- processed variants
```

Raw Capture ist die Source of Truth für die Rekonstruktion. Abgeleitete Artifacts können neu erzeugt werden, wenn sich Processing-Regeln oder Exportanforderungen ändern.

---

# Überlegung zum Audioformat

Ein lossless Format wie FLAC ist ein starker Kandidat für ein kanonisches abgeleitetes Audio-Master, weil es verlustfrei ist und gegenüber unkomprimierten PCM-Containern wie WAV oder AIFF den Speicherbedarf reduziert.

Diese ADR schreibt **FLAC ausdrücklich nicht als lokales Capture-Format vor**. Browser- und Recorder-Implementierungen können eine andere Capture-Repräsentation verwenden, solange das daraus entstehende Capture Artifact die Rekonstruktionsanforderungen erfüllt.

---

# Integrität der Capture Artifacts

NC-PoRe verwendet **SHA-256** zur Integritätsprüfung persistierter Capture Artifacts.

Jeder persistierte Capture Chunk erhält einen SHA-256-Hash über seine Payload. Das Artifact erhält zusätzlich einen deterministisch berechneten Integritäts-Hash über seine relevante Struktur, seine relevanten Metadaten und die Hashes seiner Chunks.

Die beiden Ebenen beantworten unterschiedliche Fragen:

* Der **Chunk-Hash** stellt fest, ob die gespeicherten Payload-Bytes eines einzelnen Chunks unverändert sind.
* Der **Artifact-Hash** stellt fest, ob das gesamte Artifact strukturell und inhaltlich noch dasselbe Artifact ist.

Artifact Identity und Integrity Hash sind dabei bewusst unterschiedliche Konzepte:

```text
ArtifactId   = Welches Artifact ist gemeint?
IntegrityHash = Sind dessen relevante Daten unverändert?
```

Die Integritätsinformationen gehören zur Artifact-Semantik und sind nicht provider-spezifisch. Ein Storage Provider darf und soll die Integrität beim Persistieren und Laden verifizieren, ist aber nicht die fachliche Autorität über die Bedeutung der Hashes.

Ein Integritätsfehler wird als **Inconsistent** behandelt. Ein fehlendes oder unvollständiges Artifact bleibt davon unterscheidbar als **Incomplete**; ein nicht vorhandenes Artifact als **NotFound**.

---

# Performance-Anforderung für den Capture-Pfad

Integritätsberechnung darf den eigentlichen Aufnahmevorgang nicht zum Flaschenhals machen.

Die Hash-Berechnung muss daher **streamingfähig** sein und darf keine synchrone Abhängigkeit von der Latenz des Persistence Providers in den Capture-Pfad einführen.

Konzeptionell:

```text
Audio Capture
     |
     +----> Chunk Payload
     |
     +----> SHA-256 update
              |
              +----> Persistence Queue
```

Der Capture-Pfad darf insbesondere nicht auf langsame oder netzwerkbasierte Storage Provider warten müssen, nur um die Aufnahme fortsetzen zu können.

Die konkrete Parallelisierungs-, Buffering- und Backpressure-Strategie bleibt eine spätere Implementierungsentscheidung.

---

# Konsequenzen

## Positive Auswirkungen

* Fehler während des Captures zerstören nicht zwangsläufig die vollständige Aufnahme
* Quelldaten bleiben für erneutes Processing verfügbar
* Production Artifacts können neu erzeugt werden
* Capture- und Production-Formate bleiben entkoppelt
* Recovery-Semantik wird explizit
* Beschädigte Payloads können erkannt und von fehlenden oder unvollständigen Daten unterschieden werden
* Integrität bleibt unabhängig vom konkreten Storage Provider überprüfbar

---

## Negative Auswirkungen

* zusätzliche Metadaten müssen persistiert werden
* vollständige, unvollständige und inkonsistente Artifact-Zustände müssen modelliert werden
* Rekonstruktionslogik wird zu einem eigenen Subsystem
* die Storage-Anforderungen können steigen, wenn Raw Capture erhalten bleibt
* Integritätsberechnung und Verifikation benötigen zusätzliche Rechenarbeit

---

# Betrachtete Alternativen

## Nur die fertige Mediendatei speichern

Verworfen. Eine teilweise geschriebene oder beschädigte fertige Datei kann Recovery unmöglich machen und verhindert zuverlässiges Reprocessing aus den Quelldaten.

---

## Capture direkt im kanonischen Produktionsformat erzeugen

Als allgemeine Architekturvorgabe verworfen. Dadurch würde Capture auf Browser-/Client-Seite an das Produktionsformat gekoppelt und Recovery sowie zukünftige Änderungen am Processing erschwert.

---

## Nur einen Hash für das gesamte Artifact verwenden

Verworfen. Ein einzelner Artifact-Hash kann feststellen, dass ein Artifact verändert oder beschädigt wurde, identifiziert aber nicht zuverlässig den betroffenen Chunk. Chunk-Level-Hashes unterstützen Recovery, Diagnose und Rekonstruktion besser.

---

## Provider-spezifische Prüfsummen verwenden

Verworfen. Integritätsinformationen müssen über verschiedene Storage Provider hinweg dieselbe Bedeutung behalten. Provider-spezifische Checksums können ergänzend genutzt werden, ersetzen aber nicht die provider-unabhängige Artifact-Integrität.

---

# Beziehung zu bestehender Architektur

Diese Entscheidung baut auf ADR-026 (Session Data and Storage Architecture) und ADR-035 (Domain Lifecycle and State Transition Management) auf.

Sie führt eine deutlichere Trennung zwischen Source Capture Artifacts und abgeleiteten Production Artifacts ein, ohne eine konkrete Storage-Technologie oder einen Audio-Codec vorzuschreiben.

Die Integritätsentscheidung ergänzt die bestehende Artifact- und Persistence-Abstraktion und bleibt mit der Provider-Grenze aus ADR-065 kompatibel.

---

# Zukünftige Betrachtungen

Eine spätere Implementierungsentscheidung muss Artifact Identity, Chunk Addressing, konkrete Hash-Repräsentation und Serialisierung, Completion Semantics, Retention und Rekonstruktionsregeln definieren.

Die konkrete Implementierung muss außerdem nachweisen, dass Integritätsberechnung und -prüfung den Echtzeit-Capture-Pfad nicht unzulässig belasten.

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-064: Reconstructable Capture Artifacts

## Status

Proposed

## Date

2026-08-17

## Decision Type

Architecture

---

# Context

A recording captured on a participant device may be interrupted by network failures, browser or application crashes, device problems or temporary loss of connectivity.

The capture process therefore cannot be required to produce a perfect final media file at recording time.

The comparison with Ennuicastr highlights a useful principle: captured data should remain sufficiently complete and temporally identifiable so that a coherent track can be reconstructed later.

NC-PoRe must also distinguish source capture data from derived production outputs.

---

# Decision

NC-PoRe treats **raw capture data as a first-class artifact** of a Recording.

The Raw Capture Artifact is the recoverable source from which later Production Artifacts can be reconstructed or regenerated.

Capture data must:

* be sufficiently complete to reconstruct the recorded material
* be associated with participant and session identity
* contain temporal positioning
* be identifiable at chunk/segment level where required
* distinguish complete from incomplete capture state
* distinguish complete from corrupted capture state
* allow integrity verification at chunk level
* make integrity of the complete Artifact verifiable
* be preservable independently of derived processing outputs according to storage policy

A Capture Artifact is not required to be a finished WAV, AIFF, FLAC or other production file.

---

# Architectural Principle

> Capture does not have to be perfect. It has to be complete and reconstructable enough.

Processing is responsible for turning reconstructed capture data into Production Artifacts.

---

# Rejoin and Interrupted Streams

An interruption must not automatically make the entire participant track unusable.

Where technically and semantically possible, a subsequent capture segment from the same participant and session may be associated with the existing track and reconstructed into one coherent timeline.

The exact rejoin semantics remain subject to the synchronization decision and later implementation decisions.

---

# Artifact Layers

The intended conceptual separation is:

```text
Recording
   |
   +-- Raw Capture Artifact(s)
   |       |
   |       +-- chunks / segments
   |       +-- timing information
   |       +-- capture metadata
   |
   +-- Derived Production Artifact(s)
           |
           +-- lossless master candidates
           +-- exports
           +-- processed variants
```

Raw Capture is the source of truth for reconstruction. Derived Artifacts may be regenerated when processing rules or export requirements change.

---

# Audio Format Consideration

A lossless format such as FLAC is a strong candidate for a canonical derived audio master because it is lossless while reducing storage requirements compared with uncompressed PCM containers such as WAV or AIFF.

This ADR deliberately does **not** mandate FLAC for local capture. Browser and recorder implementations may use another capture representation as long as the resulting Capture Artifact satisfies the reconstruction requirements.

---

# Capture Artifact Integrity

NC-PoRe uses **SHA-256** for integrity verification of persisted Capture Artifacts.

Each persisted Capture Chunk receives a SHA-256 hash over its payload. The Artifact additionally receives a deterministically computed integrity hash over its relevant structure, relevant metadata and the hashes of its chunks.

The two levels answer different questions:

* The **Chunk Hash** establishes whether the stored payload bytes of an individual chunk remain unchanged.
* The **Artifact Hash** establishes whether the complete Artifact remains structurally and substantively the same Artifact.

Artifact identity and integrity hash are deliberately separate concepts:

```text
ArtifactId    = Which Artifact is this?
IntegrityHash = Are its relevant data unchanged?
```

Integrity information is part of Artifact semantics and is not provider-specific. A Storage Provider may and should verify integrity during persistence and loading, but it is not the domain authority for the meaning of the hashes.

An integrity failure is treated as **Inconsistent**. A missing or incomplete Artifact remains distinguishable as **Incomplete**, while an Artifact that does not exist is **NotFound**.

---

# Performance Requirement for the Capture Path

Integrity computation must not turn the actual recording process into a bottleneck.

Hash computation must therefore be **streaming-capable** and must not introduce a synchronous dependency on Persistence Provider latency into the capture path.

Conceptually:

```text
Audio Capture
     |
     +----> Chunk Payload
     |
     +----> SHA-256 update
              |
              +----> Persistence Queue
```

In particular, the capture path must not have to wait for slow or network-based Storage Providers merely to continue recording.

The concrete parallelization, buffering and backpressure strategy remains a later implementation decision.

---

# Consequences

## Positive Effects

* failures during capture do not necessarily destroy the complete recording
* raw source data remains available for reprocessing
* Production Artifacts can be regenerated
* capture and production formats remain decoupled
* recovery semantics become explicit
* corrupted payloads can be detected and distinguished from missing or incomplete data
* integrity remains verifiable independently of the concrete Storage Provider

---

## Negative Effects

* additional metadata must be persisted
* complete, incomplete and inconsistent artifact states must be modeled
* reconstruction logic becomes a first-class subsystem
* storage requirements may increase when Raw Capture is retained
* integrity computation and verification require additional computation

---

# Alternatives Considered

## Store Only the Final Media File

Rejected. A partially written or corrupted final file can make recovery impossible and prevents reliable reprocessing from source capture data.

---

## Make Capture Directly Produce the Canonical Production Format

Rejected as a general architectural requirement. It would couple browser/client capture to the production format and make recovery and future processing changes harder.

---

## Use Only One Hash for the Complete Artifact

Rejected. A single Artifact Hash can establish that an Artifact has changed or been corrupted, but it does not reliably identify the affected chunk. Chunk-level hashes better support recovery, diagnosis and reconstruction.

---

## Use Provider-Specific Checksums

Rejected. Integrity information must retain the same meaning across different Storage Providers. Provider-specific checksums may be used additionally, but do not replace provider-independent Artifact integrity.

---

# Relationship to Existing Architecture

This decision builds on ADR-026 (Session Data and Storage Architecture) and ADR-035 (Domain Lifecycle and State Transition Management).

It introduces a more explicit distinction between source Capture Artifacts and derived Production Artifacts without making a concrete storage technology or audio codec mandatory.

The integrity decision complements the existing Artifact and Persistence abstraction and remains compatible with the provider boundary established by ADR-065.

---

# Future Considerations

A later implementation decision must define artifact identity, chunk addressing, concrete hash representation and serialization, completion semantics, retention and reconstruction rules.

The concrete implementation must also demonstrate that integrity computation and verification do not place an unacceptable load on the real-time capture path.
