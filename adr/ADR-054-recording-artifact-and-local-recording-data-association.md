# ADR-054: Recording Artifact and Local Recording Data Association

* Status: Accepted
* Date: 2026-08-09
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

---

# Kontext

NC-PoRe besitzt mit den bisherigen Architekturentscheidungen eine
klare Trennung zwischen fachlichem Recording, technischer Aufnahme,
Recording Artifact und Persistenz.

Die relevanten Entscheidungen sind:

* ADR-002 definiert getrennte Monospuren pro Teilnehmer und das
  bevorzugte Masterformat.
* ADR-003 definiert die lokale Speicherung laufender Aufnahmen in
  aufeinanderfolgenden Chunks.
* ADR-039 definiert das Recording Artifact als technische
  Repräsentation des erzeugten Aufnahmeergebnisses.
* ADR-041 definiert die Trennung zwischen Recording Artifact und
  konkreter Speicherung.
* ADR-042 definiert das Recording Artifact als eigenständiges
  technisches Modell mit eigenem Lifecycle.
* ADR-043 definiert die Local Recording Persistence Boundary.
* ADR-044 definiert das Persistence Provider Interface.
* ADR-052 definiert den `FilesystemPersistenceProvider`.
* ADR-053 definiert die Wiederherstellung der lokalen Artifact
  Discovery aus persistierten Recording Artifacts.

Damit sind die wesentlichen Architekturgrenzen definiert.

Noch nicht eindeutig festgelegt ist jedoch, wie die tatsächlich
erzeugten Recording-Daten innerhalb eines `RecordingArtifact`
technisch strukturiert und einander zugeordnet werden.

Insbesondere muss festgelegt werden:

* welche Tracks zu einem Artifact gehören,
* welche Chunks zu einem Track gehören,
* wie die technische Zugehörigkeit der Chunks zum Artifact entsteht.

Diese Entscheidung ist erforderlich, bevor die konkrete physische
Persistenzstruktur der Recording-Daten festgelegt wird.

---

# Entscheidung

NC-PoRe behandelt das `RecordingArtifact` als technische Einheit,
die die zugehörigen Recording-Daten beschreibt und eindeutig
identifiziert.

Das `RecordingArtifact` ist nicht selbst eine einzelne Audiodatei, sondern die technische Einheit, die die zugehörigen Recording-Daten strukturiert.

Die tatsächlich erzeugten Recording-Daten werden als technische
Bestandteile des Artifacts strukturiert.

Die grundlegende Beziehung lautet:

```text
RecordingArtifact
        |
        +---- RecordingTrack
        |       |
        |       +---- RecordingChunk
        |       +---- RecordingChunk
        |       +---- RecordingChunk
        |
        +---- RecordingTrack
                |
                +---- RecordingChunk
                +---- RecordingChunk
                +---- RecordingChunk
```

Ein Artifact besitzt damit:

* eine eigene technische Identität,
* eine Zuordnung zu einer Recording Session,
* eine oder mehrere technische Tracks,
* innerhalb der Tracks eine Folge technischer Chunks.

---

# Recording Artifact Identity

Die Identität eines Recording Artifacts ist unabhängig von einzelnen
Dateien oder Speicherorten.

Insbesondere darf die Artifact-Identität nicht ausschließlich aus:

* Dateinamen,
* Verzeichnispfaden,
* Chunk-Nummern,
* Track-Dateinamen

abgeleitet werden.

Ein Artifact besitzt eine eigene technische Identität.

Diese Identität bleibt auch dann bestehen, wenn sich die konkrete
physische Persistenzdarstellung ändert.

---

# Relationship to Tracks

Gemäß ADR-002 werden Aufnahmen grundsätzlich als getrennte
Monospuren pro Teilnehmer behandelt.

Ein Recording Artifact kann daher mehrere technische Tracks enthalten.

Beispiel:

```text
RecordingArtifact
    |
    +---- Host Track
    |
    +---- Guest Track
    |
    +---- CoHost Track
```

Eine `RecordingTrack` beschreibt dabei eine technische Audiospur.

Sie ist **kein fachliches Teilnehmer- oder Rollenobjekt**.

Die Track-Struktur ersetzt daher keine fachliche Teilnehmer- oder
Rollenlogik des Core.

Die Zuordnung eines Tracks zu einer fachlichen Person oder Rolle ist
eine davon getrennte Entscheidung.

---

# Relationship to Chunks

Gemäß ADR-003 werden laufende Aufnahmen lokal in aufeinanderfolgenden
Chunks gespeichert.

Ein `RecordingChunk` gehört eindeutig zu genau einem technischen
Track innerhalb eines Recording Artifacts.

Beispiel:

```text
RecordingArtifact
    |
    +---- Host Track
           |
           +---- Chunk 0001
           +---- Chunk 0002
           +---- Chunk 0003
           +---- Chunk 0004
```

Die Position eines Chunks innerhalb eines Tracks wird durch eine
technische Sequenznummer beschrieben.

Die Sequenznummer ist Bestandteil der technischen Chunk-Struktur.

Ein Chunk wird nicht als eigenständiges Recording Artifact behandelt.

---

# Chunk Ordering

Chunks eines Tracks bilden eine geordnete Folge.

Die Reihenfolge wird durch die Chunk-Sequenz bestimmt.

Beispiel:

```text
Chunk 1
   ↓
Chunk 2
   ↓
Chunk 3
   ↓
Chunk 4
```

Die physische Reihenfolge von Dateien oder Verzeichnissen ist dafür
nicht maßgeblich.

Damit bleibt die technische Chunk-Reihenfolge unabhängig von der
später gewählten Persistenzimplementierung.

---

# Artifact Contents

Das Recording Artifact kann technische Informationen über sein
Aufnahmeergebnis repräsentieren.

Dazu gehören insbesondere:

* Artifact Identifier,
* Recording Session Identifier,
* technische Track-Struktur,
* Chunk-Zuordnung,
* technische Aufnahmeeigenschaften,
* Zeitinformationen,
* Integritätsinformationen,
* technischer Artifact-Lifecycle.

Diese ADR legt nicht fest, dass sämtliche genannten Informationen
bereits Bestandteil der aktuellen Implementierung sein müssen.

Die konkrete Rust-Datenstruktur wird durch die jeweilige
Implementierungsentscheidung festgelegt.

---

# Persistence Independence

Das Artifact-Modell beschreibt die technische Beziehung der
Recording-Daten.

Es entscheidet nicht über deren physische Speicherung.

Insbesondere legt diese ADR nicht fest:

* konkrete Verzeichnisse,
* konkrete Dateinamen,
* Dateiendungen,
* Serialisierungsformate,
* temporäre Dateien,
* Markerdateien,
* Indexdateien.

Diese Aspekte werden durch die konkrete Persistence-Entscheidung
festgelegt.

---

# Persistence Relationship

Die Architektur bleibt:

```text
Capture
   |
   v
Recording Data
   |
   v
RecordingArtifact
   |
   v
PersistenceProvider
   |
   v
Persistence Implementation
```

Der `PersistenceProvider` ist dafür verantwortlich, die durch das
Artifact beschriebenen persistierbaren Daten technisch zu speichern
und wiederherzustellen.

Der Recorder Workflow kennt dabei keine konkreten
Speicherstrukturen.

---

# Lifecycle Independence

Das Recording Artifact besitzt weiterhin den in ADR-042 definierten
technischen Lifecycle.

Die Existenz einzelner Chunks stellt keinen eigenen Artifact-Lifecycle
dar.

Während einer laufenden Aufnahme können bereits mehrere Chunks
existieren.

Beispiel:

```text
RecordingArtifact
    |
    +---- Track
           |
           +---- Chunk 0001   completed
           +---- Chunk 0002   completed
           +---- Chunk 0003   completed
           +---- Chunk 0004   writing
```

Ein unvollständig geschriebener Chunk erzeugt kein neues Artifact.

Die Behandlung unvollständiger oder beschädigter Chunks gehört zur
Recovery- und Persistence-Implementierung.

---

# Recovery Relationship

Persistierte Recording-Daten gehören technisch zu einem eindeutig
identifizierbaren Recording Artifact.

Die Recovery kann dadurch die Beziehung zwischen persistiertem
Artifact und lokaler Artifact Discovery wiederherstellen.

Die Recovery selbst verändert jedoch nicht die Artifact-Struktur.

Die Recovery-Verantwortung bleibt durch ADR-053 definiert.

---

# Consequences

## Positive Consequences

* Recording-Daten besitzen eine eindeutige technische Struktur.
* Tracks und Chunks sind vom fachlichen Domain-Modell getrennt.
* Artifact-Identität bleibt unabhängig von konkreter Speicherung.
* Chunk-Reihenfolge bleibt unabhängig von Dateisystemstrukturen.
* Die spätere Persistenzentscheidung kann auf einer klar definierten
  Datenbeziehung aufbauen.

## Negative Consequences

* Das Artifact-Modell enthält zusätzliche technische Strukturen.
* Track- und Chunk-Zuordnung müssen implementiert und getestet werden.
* Die Persistenzschicht muss die definierte Beziehung später technisch
  abbilden.

Diese Konsequenzen werden bewusst akzeptiert.

---

# Nicht Teil dieser Entscheidung

Diese ADR entscheidet ausdrücklich nicht über:

* konkretes Filesystem-Layout,
* konkrete Dateinamen,
* Dateiformate,
* Audio-Codecs,
* Chunk-Größe,
* Synchronisationsprotokolle,
* Konfliktauflösung,
* Exportformate,
* Garbage Collection,
* Datenbankpersistenz,
* Cloud Storage.

Diese Themen werden durch eigene technische Entscheidungen behandelt.

---

# Beziehung zu anderen ADRs

Diese ADR baut insbesondere auf folgenden Entscheidungen auf:

* ADR-002 Recording Architecture and Track Separation
* ADR-003 Local Chunk-Based Audio Storage
* ADR-039 Recording Architecture and Capture Boundary
* ADR-041 Local Recording Artifact and Storage Boundary
* ADR-042 Recording Artifact Model and Lifecycle Boundary
* ADR-043 Local Recording Persistence Boundary
* ADR-044 Persistence Provider Interface
* ADR-052 Local Filesystem Persistence Provider
* ADR-053 Artifact Recovery and Consistency Boundary

Die Beziehung zwischen den Entscheidungen lautet:

```text
ADR-003
Chunk-basierte Aufnahme

        ↓

ADR-042
Recording Artifact

        ↓

ADR-054
Artifact → Track → Chunk

        ↓

ADR-055
Filesystem Persistence Layout
```

---

# Ergebnis

Das Recording Artifact ist die technische Einheit eines lokalen
Aufnahmeergebnisses.

Seine Recording-Daten sind hierarchisch strukturiert als:

```text
RecordingArtifact
    |
    +---- RecordingTrack
              |
              +---- RecordingChunk
              +---- RecordingChunk
              +---- ...
```

Die konkrete physische Darstellung dieser Struktur wird nicht durch
diese ADR entschieden.

---

# English Version ([German version above](#deutsch))

---

# Context

NC-PoRe already defines clear architectural boundaries between
domain recording, technical capture, Recording Artifacts, and
persistence.

The relevant decisions include:

* ADR-002 defines separate mono tracks per participant and the
  preferred master format.
* ADR-003 defines local storage of ongoing recordings as consecutive
  chunks.
* ADR-039 defines the Recording Artifact as the technical
  representation of the resulting recording data.
* ADR-041 defines the separation between Recording Artifact and
  concrete storage.
* ADR-042 defines the Recording Artifact as an independent technical
  model with its own lifecycle.
* ADR-043 defines the Local Recording Persistence Boundary.
* ADR-044 defines the Persistence Provider Interface.
* ADR-052 defines the `FilesystemPersistenceProvider`.
* ADR-053 defines recovery of local artifact discovery from persisted
  Recording Artifacts.

The major architectural boundaries are therefore established.

What has not yet been explicitly defined is how the actual recording
data is technically structured and associated within a
`RecordingArtifact`.

In particular, the following relationships must be defined:

* which tracks belong to an artifact,
* which chunks belong to a track,
* how chunks are technically associated with the artifact.

This decision is required before the concrete physical persistence
structure can be defined.

---

# Decision

NC-PoRe treats the `RecordingArtifact` as the technical unit that
describes and uniquely identifies its associated recording data.

The `RecordingArtifact` is not itself a single audio file, but the technical unit that structures the associated recording data.

The actual recording data is represented as technical components of
the artifact.

The fundamental relationship is:

```text
RecordingArtifact
        |
        +---- RecordingTrack
        |       |
        |       +---- RecordingChunk
        |       +---- RecordingChunk
        |       +---- RecordingChunk
        |
        +---- RecordingTrack
                |
                +---- RecordingChunk
                +---- RecordingChunk
                +---- RecordingChunk
```

An artifact therefore has:

* its own technical identity,
* an association with a recording session,
* one or more technical tracks,
* an ordered sequence of technical chunks within each track.

---

# Recording Artifact Identity

The identity of a Recording Artifact is independent of individual
files or storage locations.

In particular, artifact identity must not be derived exclusively
from:

* filenames,
* directory paths,
* chunk numbers,
* track filenames.

An artifact has its own technical identity.

That identity remains stable even if its physical persistence
representation changes.

---

# Relationship to Tracks

According to ADR-002, recordings are fundamentally represented as
separate mono tracks per participant.

A Recording Artifact can therefore contain multiple technical tracks.

Example:

```text
RecordingArtifact
    |
    +---- Host Track
    |
    +---- Guest Track
    |
    +---- CoHost Track
```

A `RecordingTrack` represents a technical audio track.

It is **not a domain participant or role object**.

Track structure therefore does not replace participant or role logic
in the Core.

Association of a track with a domain person or role is a separate
decision.

---

# Relationship to Chunks

According to ADR-003, ongoing recordings are stored locally as
consecutive chunks.

A `RecordingChunk` belongs to exactly one technical track within a
Recording Artifact.

Example:

```text
RecordingArtifact
    |
    +---- Host Track
           |
           +---- Chunk 0001
           +---- Chunk 0002
           +---- Chunk 0003
           +---- Chunk 0004
```

The position of a chunk within a track is represented by a technical
sequence number.

The sequence number is part of the technical chunk structure.

A chunk is not treated as an independent Recording Artifact.

---

# Chunk Ordering

Chunks belonging to a track form an ordered sequence.

The sequence number defines their technical order.

Example:

```text
Chunk 1
   ↓
Chunk 2
   ↓
Chunk 3
   ↓
Chunk 4
```

The physical ordering of files or directories is not authoritative.

This keeps chunk ordering independent from the eventual persistence
implementation.

---

# Artifact Contents

The Recording Artifact may represent technical information about its
recording result.

This includes in particular:

* Artifact Identifier,
* Recording Session Identifier,
* technical track structure,
* chunk association,
* technical recording properties,
* timing information,
* integrity information,
* technical artifact lifecycle.

This ADR does not require all of the listed information to already
exist in the current implementation.

The concrete Rust data structure is defined by the corresponding
implementation decision.

---

# Persistence Independence

The artifact model defines the technical relationship between
recording data components.

It does not define their physical storage.

This ADR therefore does not define:

* concrete directories,
* concrete filenames,
* file extensions,
* serialization formats,
* temporary files,
* marker files,
* index files.

These aspects are defined by the concrete persistence decision.

---

# Persistence Relationship

The architecture remains:

```text
Capture
   |
   v
Recording Data
   |
   v
RecordingArtifact
   |
   v
PersistenceProvider
   |
   v
Persistence Implementation
```

The `PersistenceProvider` is responsible for technically storing and
restoring the persistable data described by the artifact.

The Recorder workflow does not know concrete storage structures.

---

# Lifecycle Independence

The Recording Artifact continues to have the technical lifecycle
defined by ADR-042.

The existence of individual chunks does not constitute a separate
artifact lifecycle.

Multiple chunks may already exist while a recording is still active.

Example:

```text
RecordingArtifact
    |
    +---- Track
           |
           +---- Chunk 0001   completed
           +---- Chunk 0002   completed
           +---- Chunk 0003   completed
           +---- Chunk 0004   writing
```

An incomplete chunk does not create a new artifact.

Handling incomplete or damaged chunks belongs to persistence and
recovery implementation.

---

# Recovery Relationship

Persisted recording data belongs technically to a uniquely
identifiable Recording Artifact.

Recovery can therefore restore the relationship between persisted
artifacts and local artifact discovery.

Recovery itself does not modify the artifact structure.

Recovery responsibilities remain defined by ADR-053.

---

# Consequences

## Positive Consequences

* Recording data has a clearly defined technical structure.
* Tracks and chunks remain separate from the domain model.
* Artifact identity remains independent from concrete storage.
* Chunk ordering remains independent from filesystem structures.
* The later persistence decision can build on a clearly defined data
  relationship.

## Negative Consequences

* The artifact model contains additional technical structures.
* Track and chunk relationships must be implemented and tested.
* The persistence layer must later represent these relationships
  physically.

These consequences are explicitly accepted.

---

# Not Part of This Decision

This ADR explicitly does not decide:

* concrete filesystem layout,
* concrete filenames,
* file formats,
* audio codecs,
* chunk size,
* synchronization protocols,
* conflict resolution,
* export formats,
* garbage collection,
* database persistence,
* cloud storage.

These topics are handled by separate technical decisions.

---

# Relationship to Other ADRs

This ADR builds particularly on:

* ADR-002 Recording Architecture and Track Separation
* ADR-003 Local Chunk-Based Audio Storage
* ADR-039 Recording Architecture and Capture Boundary
* ADR-041 Local Recording Artifact and Storage Boundary
* ADR-042 Recording Artifact Model and Lifecycle Boundary
* ADR-043 Local Recording Persistence Boundary
* ADR-044 Persistence Provider Interface
* ADR-052 Local Filesystem Persistence Provider
* ADR-053 Artifact Recovery and Consistency Boundary

The relationship between these decisions is:

```text
ADR-003
Chunk-based recording

        ↓

ADR-042
Recording Artifact

        ↓

ADR-054
Artifact → Track → Chunk

        ↓

ADR-055
Filesystem Persistence Layout
```

---

# Result

The Recording Artifact is the technical unit of a local recording
result.

Its recording data is hierarchically structured as:

```text
RecordingArtifact
    |
    +---- RecordingTrack
              |
              +---- RecordingChunk
              +---- RecordingChunk
              +---- ...
```

The concrete physical representation of this structure is not
decided by this ADR.
