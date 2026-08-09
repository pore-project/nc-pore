# ADR-056: Capture Result and Recording Artifact Data Boundary

* Status: Accepted
* Date: 2026-08-09
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

---

# Kontext

NC-PoRe trennt die technische Audioaufnahme von der weiteren
Verarbeitung des erzeugten Aufnahmeergebnisses.

Diese Trennung ist durch mehrere bestehende Architekturentscheidungen
definiert:

* ADR-039 definiert die Capture Boundary.
* ADR-040 definiert die Koordination des Recorder-Workflows.
* ADR-041 definiert die Trennung zwischen lokalen Recording-Daten und
  deren Speicherung.
* ADR-042 definiert das Recording Artifact als eigenständige technische
  Repräsentation eines Aufnahmeergebnisses.
* ADR-054 definiert die technische Struktur eines Recording Artifacts
  mit Tracks und Chunks.

Damit ist die Struktur des Recording Artifacts definiert.

Die aktuelle `CaptureResult`-Implementierung enthält jedoch lediglich
eine technische Identität:

```text
CaptureResult
    |
    +---- id
```

Ein tatsächliches Aufnahmeergebnis kann dagegen aus mehreren
technischen Tracks und mehreren Chunks bestehen:

```text
CaptureResult
    |
    +---- Recording Data
            |
            +---- Track
            |       |
            |       +---- Chunk
            |       +---- Chunk
            |
            +---- Track
                    |
                    +---- Chunk
                    +---- Chunk
```

Die aktuelle Schnittstelle stellt diese Recording-Daten noch nicht
für die nachgelagerte Artifact-Erzeugung zur Verfügung.

Damit fehlt eine definierte technische Grenze zwischen dem Ergebnis
des Capture-Vorgangs und dem daraus erzeugten Recording Artifact.

---

# Entscheidung

NC-PoRe erweitert das technische Capture-Ergebnis so, dass es die
für die Erzeugung eines `RecordingArtifact` erforderlichen
Recording-Daten repräsentieren kann.

Das `CaptureResult` bleibt dabei ein technisches Ergebnis der
Capture-Schicht.

Es wird **nicht** zu einem Recording Artifact.

Die Beziehung lautet:

```text
CaptureProvider
      |
      v
CaptureResult
      |
      | enthält bzw. referenziert
      | technische Recording-Daten
      v
RecordingArtifactFactory
      |
      v
RecordingArtifact
      |
      +---- RecordingTrack
      |       |
      |       +---- RecordingChunk
      |       +---- RecordingChunk
      |
      +---- RecordingTrack
              |
              +---- RecordingChunk
```

Die Artifact-Erzeugung bleibt Aufgabe der Artifact-Schicht.

---

# CaptureResult Responsibility

`CaptureResult` beschreibt ausschließlich das technische Ergebnis
eines abgeschlossenen Capture-Vorgangs.

Es kann insbesondere repräsentieren:

* die technische Capture-Identität,
* die erzeugten Recording-Daten,
* die technische Track-Struktur,
* die technische Chunk-Struktur,
* technische Eigenschaften des erzeugten Capture-Ergebnisses.

`CaptureResult` entscheidet nicht über:

* Artifact-Lifecycle,
* Persistence,
* Synchronisation,
* Export,
* fachliche Teilnehmer,
* fachliche Rollen,
* Produktionsregeln.

Diese Verantwortlichkeiten bleiben in den bestehenden
Architekturgrenzen.

---

# Recording Data Boundary

Die von `CaptureResult` repräsentierten Recording-Daten sind
technische Daten.

Sie sind nicht mit ihrer späteren Persistenzdarstellung
gleichzusetzen.

Insbesondere darf `CaptureResult` nicht von folgenden technischen
Details abhängen:

* Persistence Provider,
* Filesystem,
* konkretem Persistence Root,
* Artifact-Verzeichnis,
* `artifact.json`,
* konkreten Persistenzpfaden.

Die Capture-Schicht erzeugt Recording-Daten.

Die Persistence-Schicht entscheidet, wie diese Daten gespeichert
werden.

---

# Relationship to RecordingArtifact

Die Artifact-Schicht übernimmt die technischen Recording-Daten aus
dem `CaptureResult` und bildet daraus ein `RecordingArtifact`.

Konzeptionell:

```text
CaptureResult
    |
    | technische Recording-Daten
    v
RecordingArtifactFactory
    |
    | strukturiertes Artifact
    v
RecordingArtifact
```

Dabei werden die technischen Beziehungen von Tracks und Chunks
erhalten.

Beispiel:

```text
CaptureResult

Track A
    |
    +---- Chunk 1
    +---- Chunk 2

Track B
    |
    +---- Chunk 1
    +---- Chunk 2
    +---- Chunk 3

        ↓

RecordingArtifact

Track A
    |
    +---- Chunk 1
    +---- Chunk 2

Track B
    |
    +---- Chunk 1
    +---- Chunk 2
    +---- Chunk 3
```

Die Factory übernimmt dabei die Verantwortung für die Erzeugung des
Artifact-Modells.

---

# Chunk Data

Ein `RecordingChunk` beschreibt eine technische Einheit der
Recording-Daten.

Die technische Identität und Reihenfolge eines Chunks werden durch
das Artifact-Modell gemäß ADR-054 beschrieben.

Diese ADR legt nicht fest, wie die eigentlichen Audiodaten innerhalb
eines Chunks im Speicher repräsentiert werden.

Insbesondere wird hier nicht festgelegt, ob ein Chunk:

* einen Dateipfad,
* einen Speicherpuffer,
* einen Stream,
* einen Handle,
* eine andere technische Datenreferenz

verwendet.

Die konkrete Repräsentation wird durch die Implementierung und
gegebenenfalls weitere Architekturentscheidungen bestimmt.

---

# No Persistence Dependency

Das `CaptureResult` darf keine direkte Abhängigkeit auf den
`PersistenceProvider` besitzen.

Nicht zulässig ist beispielsweise:

```text
CaptureResult
    |
    +---- PersistenceProvider
```

Ebenso darf der Capture Provider nicht selbst für die Persistenz
eines Recording Artifacts verantwortlich sein.

Die gewünschte Architektur bleibt:

```text
CaptureProvider
      |
      v
CaptureResult
      |
      v
Artifact Processing
      |
      v
RecordingArtifact
      |
      v
PersistenceProvider
```

Damit bleibt die Capture Boundary unabhängig von der konkreten
Speichertechnologie.

---

# No Domain Dependency

`CaptureResult` und die enthaltenen technischen Recording-Daten
dürfen keine fachliche Teilnehmer- oder Rollenlogik voraussetzen.

Insbesondere ist nicht Bestandteil dieser Boundary:

```text
CaptureResult
    |
    +---- Participant
    +---- Role
```

Die technische Track-Struktur kann später durch eine getrennte
Architekturentscheidung mit fachlichen Informationen verbunden
werden.

Diese Verbindung gehört nicht in die Capture Boundary.

---

# Relationship to ADR-054

ADR-054 definiert:

```text
RecordingArtifact
    |
    +---- RecordingTrack
            |
            +---- RecordingChunk
```

ADR-056 definiert die vorgelagerte technische Datenquelle für diese
Struktur:

```text
CaptureResult
    |
    +---- Recording Data
            |
            +---- Track
                    |
                    +---- Chunk
```

Damit ergibt sich:

```text
Capture
   |
   v
CaptureResult
   |
   v
RecordingArtifact
   |
   v
Persistence
```

ADR-054 bleibt für die interne Struktur des Recording Artifacts
maßgeblich.

ADR-056 definiert lediglich die technische Übergabe der
Recording-Daten vom Capture-Ergebnis in das Artifact-Modell.

---

# Relationship to ADR-055

ADR-055 definiert die physische Filesystem-Repräsentation eines
persistierten Recording Artifacts.

ADR-056 liegt davor:

```text
CaptureResult
      |
      v
RecordingArtifact
      |
      v
FilesystemPersistenceProvider
      |
      v
Filesystem
```

Das `CaptureResult` kennt daher weder das Artifact-Verzeichnis noch
die konkreten Chunk-Dateien.

Die Filesystem-Struktur bleibt ausschließlich Aufgabe der
Persistence-Schicht.

---

# Lifecycle

Das `CaptureResult` besitzt keinen Artifact-Lifecycle.

Es repräsentiert das Ergebnis des Capture-Vorgangs.

Der technische Artifact-Lifecycle beginnt erst mit dem
`RecordingArtifact`.

Beispiel:

```text
Capture
   |
   v
CaptureResult
   |
   v
Artifact Created
   |
   v
Artifact Lifecycle
```

Ein CaptureResult wird daher nicht selbst als `Created`, `Stored`,
`Synchronized` oder `Archived` behandelt.

Diese Zustände gehören zum Recording Artifact gemäß ADR-042.

---

# Consequences

## Positive Consequences

* Capture und Artifact bleiben getrennt.
* Die tatsächlichen Recording-Daten können die Artifact-Struktur
  erreichen.
* Tracks und Chunks werden nicht in der Capture-Schicht mit
  Persistence-Logik vermischt.
* Der `PersistenceProvider` bleibt von Capture unabhängig.
* Die bestehende Architekturgrenze bleibt erhalten.
* Die spätere Filesystem-Persistenz kann auf einem vollständigen
  Artifact-Modell aufbauen.

## Negative Consequences

* `CaptureResult` wird technisch umfangreicher.
* Die Capture-Schicht muss Recording-Daten strukturiert bereitstellen.
* Die Artifact-Erzeugung benötigt eine definierte Transformation
  zwischen CaptureResult und RecordingArtifact.
* Die technische Repräsentation der Recording-Daten muss später
  implementiert und getestet werden.

Diese Konsequenzen werden bewusst akzeptiert.

---

# Nicht Teil dieser Entscheidung

Diese ADR entscheidet ausdrücklich nicht über:

* konkrete Audiodatenformate,
* WAV- oder FLAC-Dateien,
* Chunk-Dateiendungen,
* Chunk-Dateigrößen,
* Filesystem-Pfade,
* Persistence Provider,
* Synchronisation,
* Export,
* fachliche Teilnehmerzuordnung,
* Rollen,
* Audio-Codecs,
* Kompression,
* Verschlüsselung,
* Cloud Storage,
* Datenbankpersistenz.

Diese Themen bleiben bestehenden oder zukünftigen
Architekturentscheidungen vorbehalten.

---

# Implementierungsgrenze

Die konkrete Rust-Datenstruktur für die Recording-Daten wird durch die
Implementierung festgelegt.

Diese ADR verlangt jedoch, dass die nachgelagerte Artifact-Erzeugung
die technische Track-/Chunk-Struktur aus dem Capture-Ergebnis
ableiten kann.

Die bestehende Minimalimplementierung:

```rust
pub struct CaptureResult {
    id: String,
}
```

erfüllt diese Anforderung noch nicht.

Die notwendige Erweiterung wird in einem separaten
Implementierungsschritt vorgenommen.

---

# Testabsicherung

Die Boundary wird durch Tests abgesichert.

Insbesondere muss sichergestellt werden, dass:

1. ein `CaptureResult` technische Recording-Daten repräsentieren kann,
2. Tracks und Chunks aus dem Capture-Ergebnis in das
   `RecordingArtifact` übernommen werden,
3. die Reihenfolge der Chunks erhalten bleibt,
4. die Artifact-Erzeugung keine Persistence-Abhängigkeit benötigt,
5. `CaptureResult` keinen Artifact-Lifecycle übernimmt.

---

# Ergebnis

ADR-056 definiert die technische Grenze zwischen dem Ergebnis des
Capture-Vorgangs und dem Recording Artifact.

Die Architektur lautet damit:

```text
CaptureProvider
      |
      v
CaptureResult
      |
      v
RecordingArtifactFactory
      |
      v
RecordingArtifact
      |
      v
PersistenceProvider
```

Damit kann die Implementierung der Recording-Daten erfolgen, ohne
Capture, Artifact-Modell und Persistence miteinander zu koppeln.

---

# English Version ([German version above](#deutsch))

---

# Context

NC-PoRe separates technical audio capture from further processing of
the resulting recording data.

This separation is defined by several existing architectural
decisions:

* ADR-039 defines the Capture Boundary.
* ADR-040 defines recorder workflow coordination.
* ADR-041 defines the separation between local recording data and
  storage.
* ADR-042 defines the Recording Artifact as an independent technical
  representation of a recording result.
* ADR-054 defines the technical structure of a Recording Artifact
  consisting of tracks and chunks.

The structure of the Recording Artifact is therefore defined.

The current `CaptureResult` implementation, however, contains only a
technical identifier:

```text
CaptureResult
    |
    +---- id
```

An actual recording result may consist of multiple technical tracks
and multiple chunks:

```text
CaptureResult
    |
    +---- Recording Data
            |
            +---- Track
            |       |
            |       +---- Chunk
            |       +---- Chunk
            |
            +---- Track
                    |
                    +---- Chunk
                    +---- Chunk
```

The current interface does not yet make these recording data
available to downstream Artifact creation.

A defined technical boundary between the result of capture and the
resulting Recording Artifact is therefore required.

---

# Decision

NC-PoRe extends the technical capture result so that it can represent
the recording data required to create a `RecordingArtifact`.

`CaptureResult` remains a technical result of the Capture layer.

It does **not** become a Recording Artifact.

The relationship is:

```text
CaptureProvider
      |
      v
CaptureResult
      |
      | contains or references
      | technical recording data
      v
RecordingArtifactFactory
      |
      v
RecordingArtifact
      |
      +---- RecordingTrack
      |       |
      |       +---- RecordingChunk
      |       +---- RecordingChunk
      |
      +---- RecordingTrack
              |
              +---- RecordingChunk
```

Artifact creation remains the responsibility of the Artifact layer.

---

# CaptureResult Responsibility

`CaptureResult` describes only the technical result of a completed
capture operation.

It may represent:

* technical capture identity,
* generated recording data,
* technical track structure,
* technical chunk structure,
* technical properties of the captured result.

`CaptureResult` does not decide:

* Artifact lifecycle,
* persistence,
* synchronization,
* export,
* domain participants,
* domain roles,
* production rules.

These responsibilities remain within their existing architectural
boundaries.

---

# Recording Data Boundary

The recording data represented by `CaptureResult` are technical data.

They must not be confused with their later persistence
representation.

In particular, `CaptureResult` must not depend on:

* Persistence Provider,
* filesystem,
* concrete persistence root,
* artifact directory,
* `artifact.json`,
* concrete persistence paths.

The Capture layer produces recording data.

The Persistence layer decides how those data are stored.

---

# Relationship to RecordingArtifact

The Artifact layer takes the technical recording data from
`CaptureResult` and constructs a `RecordingArtifact`.

Conceptually:

```text
CaptureResult
    |
    | technical recording data
    v
RecordingArtifactFactory
    |
    | structured artifact
    v
RecordingArtifact
```

The technical relationships between tracks and chunks are preserved.

Example:

```text
CaptureResult

Track A
    |
    +---- Chunk 1
    +---- Chunk 2

Track B
    |
    +---- Chunk 1
    +---- Chunk 2
    +---- Chunk 3

        ↓

RecordingArtifact

Track A
    |
    +---- Chunk 1
    +---- Chunk 2

Track B
    |
    +---- Chunk 1
    +---- Chunk 2
    +---- Chunk 3
```

The Factory is responsible for constructing the Artifact model.

---

# Chunk Data

A `RecordingChunk` represents a technical unit of recording data.

The technical identity and ordering of a chunk are defined by the
Artifact model according to ADR-054.

This ADR does not define how the actual audio data of a chunk are
represented in memory.

In particular, this ADR does not decide whether a chunk uses:

* a file path,
* a memory buffer,
* a stream,
* a handle,
* another technical data reference.

The concrete representation is determined by implementation and,
where required, further architectural decisions.

---

# No Persistence Dependency

`CaptureResult` must not have a direct dependency on the
`PersistenceProvider`.

The following is therefore not permitted:

```text
CaptureResult
    |
    +---- PersistenceProvider
```

Likewise, the Capture Provider must not be responsible for persisting
a Recording Artifact.

The intended architecture remains:

```text
CaptureProvider
      |
      v
CaptureResult
      |
      v
Artifact Processing
      |
      v
RecordingArtifact
      |
      v
PersistenceProvider
```

This keeps the Capture Boundary independent from concrete storage
technology.

---

# No Domain Dependency

`CaptureResult` and the technical recording data it contains must not
require domain participant or role logic.

The following is explicitly not part of this boundary:

```text
CaptureResult
    |
    +---- Participant
    +---- Role
```

A later architectural decision may connect technical tracks with
domain information.

That connection does not belong in the Capture Boundary.

---

# Relationship to ADR-054

ADR-054 defines:

```text
RecordingArtifact
    |
    +---- RecordingTrack
            |
            +---- RecordingChunk
```

ADR-056 defines the preceding technical data source for this
structure:

```text
CaptureResult
    |
    +---- Recording Data
            |
            +---- Track
                    |
                    +---- Chunk
```

The resulting architecture is:

```text
Capture
   |
   v
CaptureResult
   |
   v
RecordingArtifact
   |
   v
Persistence
```

ADR-054 remains authoritative for the internal structure of the
Recording Artifact.

ADR-056 only defines the technical transfer of recording data from
the capture result into the Artifact model.

---

# Relationship to ADR-055

ADR-055 defines the physical filesystem representation of a persisted
Recording Artifact.

ADR-056 precedes it:

```text
CaptureResult
      |
      v
RecordingArtifact
      |
      v
FilesystemPersistenceProvider
      |
      v
Filesystem
```

`CaptureResult` therefore knows neither the artifact directory nor
the concrete chunk files.

The filesystem structure remains exclusively a Persistence-layer
responsibility.

---

# Lifecycle

`CaptureResult` has no Artifact lifecycle.

It represents the result of the capture operation.

The technical Artifact lifecycle starts with the `RecordingArtifact`.

Example:

```text
Capture
   |
   v
CaptureResult
   |
   v
Artifact Created
   |
   v
Artifact Lifecycle
```

A CaptureResult is therefore not itself treated as `Created`,
`Stored`, `Synchronized`, or `Archived`.

These states belong to the Recording Artifact according to ADR-042.

---

# Consequences

## Positive Consequences

* Capture and Artifact remain separate.
* Actual recording data can reach the Artifact structure.
* Tracks and chunks are not mixed with persistence logic in the
  Capture layer.
* The Persistence Provider remains independent of Capture.
* The existing architectural boundary remains intact.
* Filesystem persistence can later build on a complete Artifact model.

## Negative Consequences

* `CaptureResult` becomes technically more substantial.
* The Capture layer must provide recording data in a structured form.
* Artifact creation requires a defined transformation between
  CaptureResult and RecordingArtifact.
* The technical representation of recording data must later be
  implemented and tested.

These consequences are intentionally accepted.

---

# Not Part of This Decision

This ADR explicitly does not decide:

* concrete audio data formats,
* WAV or FLAC files,
* chunk file extensions,
* chunk file sizes,
* filesystem paths,
* Persistence Providers,
* synchronization,
* export,
* domain participant assignment,
* roles,
* audio codecs,
* compression,
* encryption,
* cloud storage,
* database persistence.

These topics remain governed by existing or future architectural
decisions.

---

# Implementation Boundary

The concrete Rust data structure for recording data is determined by
implementation.

This ADR does, however, require that downstream Artifact creation
can derive the technical track/chunk structure from the capture
result.

The current minimal implementation:

```rust
pub struct CaptureResult {
    id: String,
}
```

does not yet satisfy this requirement.

The necessary extension will be implemented in a separate
implementation step.

---

# Test Coverage

The boundary is protected by tests.

In particular, tests must ensure that:

1. a `CaptureResult` can represent technical recording data,
2. tracks and chunks from the capture result are transferred into the
   `RecordingArtifact`,
3. chunk ordering is preserved,
4. Artifact creation has no persistence dependency,
5. `CaptureResult` does not assume an Artifact lifecycle.

---

# Result

ADR-056 defines the technical boundary between the result of a
capture operation and the Recording Artifact.

The resulting architecture is:

```text
CaptureProvider
      |
      v
CaptureResult
      |
      v
RecordingArtifactFactory
      |
      v
RecordingArtifact
      |
      v
PersistenceProvider
```

This allows recording-data implementation to proceed without coupling
Capture, Artifact modeling, and Persistence.
