# Deutsch ([English version below](#english-version))

# ADR-038: Reconstructable Capture Artifacts

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

# Konsequenzen

## Positive Auswirkungen

* Fehler während des Captures zerstören nicht zwangsläufig die vollständige Aufnahme
* Quelldaten bleiben für erneutes Processing verfügbar
* Production Artifacts können neu erzeugt werden
* Capture- und Production-Formate bleiben entkoppelt
* Recovery-Semantik wird explizit

---

## Negative Auswirkungen

* zusätzliche Metadaten müssen persistiert werden
* vollständige und unvollständige Artifact-Zustände müssen modelliert werden
* Rekonstruktionslogik wird zu einem eigenen Subsystem
* die Storage-Anforderungen können steigen, wenn Raw Capture erhalten bleibt

---

# Betrachtete Alternativen

## Nur die fertige Mediendatei speichern

Verworfen. Eine teilweise geschriebene oder beschädigte fertige Datei kann Recovery unmöglich machen und verhindert zuverlässiges Reprocessing aus den Quelldaten.

---

## Capture direkt im kanonischen Produktionsformat erzeugen

Als allgemeine Architekturvorgabe verworfen. Dadurch würde Capture auf Browser-/Client-Seite an das Produktionsformat gekoppelt und Recovery sowie zukünftige Änderungen am Processing erschwert.

---

# Beziehung zu bestehender Architektur

Diese Entscheidung baut auf ADR-026 (Session Data and Storage Architecture) und ADR-035 (Domain Lifecycle and State Transition Management) auf.

Sie führt eine deutlichere Trennung zwischen Source Capture Artifacts und abgeleiteten Production Artifacts ein, ohne eine konkrete Storage-Technologie oder einen Audio-Codec vorzuschreiben.

---

# Zukünftige Betrachtungen

Eine spätere Implementierungsentscheidung muss Artifact Identity, Chunk Addressing, Integritätsinformationen, Completion Semantics, Retention und Rekonstruktionsregeln definieren.

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-038: Reconstructable Capture Artifacts

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

# Consequences

## Positive Effects

* failures during capture do not necessarily destroy the complete recording
* raw source data remains available for reprocessing
* Production Artifacts can be regenerated
* capture and production formats remain decoupled
* recovery semantics become explicit

---

## Negative Effects

* additional metadata must be persisted
* complete and incomplete artifact states must be modeled
* reconstruction logic becomes a first-class subsystem
* storage requirements may increase when Raw Capture is retained

---

# Alternatives Considered

## Store Only the Final Media File

Rejected. A partially written or corrupted final file can make recovery impossible and prevents reliable reprocessing from source capture data.

---

## Make Capture Directly Produce the Canonical Production Format

Rejected as a general architectural requirement. It would couple browser/client capture to the production format and make recovery and future processing changes harder.

---

# Relationship to Existing Architecture

This decision builds on ADR-026 (Session Data and Storage Architecture) and ADR-035 (Domain Lifecycle and State Transition Management).

It introduces a more explicit distinction between source Capture Artifacts and derived Production Artifacts without making a concrete storage technology or audio codec mandatory.

---

# Future Considerations

A later implementation decision must define artifact identity, chunk addressing, integrity information, completion semantics, retention and reconstruction rules.
