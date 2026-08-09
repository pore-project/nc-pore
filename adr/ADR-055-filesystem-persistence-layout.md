# ADR-055: Filesystem Persistence Layout

* Status: Accepted
* Date: 2026-08-09
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

---

# Kontext

ADR-052 führte den `FilesystemPersistenceProvider` als konkrete
Implementierung der bestehenden Persistence Boundary ein.

ADR-052 ließ das konkrete Filesystem-Layout ausdrücklich offen.

ADR-054 definiert inzwischen die technische Beziehung:

```text
RecordingArtifact
    |
    +---- RecordingTrack
              |
              +---- RecordingChunk
              +---- RecordingChunk
              +---- ...
```

Damit ist festgelegt, welche technischen Bestandteile ein Artifact
beschreiben kann.

Für die tatsächliche lokale Persistenz fehlt jedoch noch eine
explizite Entscheidung darüber, wie diese Struktur auf dem
Filesystem dargestellt wird.

Das bisherige Implementierungsmodell speichert ein Artifact als
einzelne Datei:

```text
<artifact-id>.json
```

Dieses Modell kann die in ADR-054 definierte Track-/Chunk-Struktur
nicht vollständig repräsentieren.

Eine konkrete Filesystem-Struktur wird daher benötigt.

---

# Entscheidung

Der `FilesystemPersistenceProvider` verwendet pro
`RecordingArtifact` ein eigenes Verzeichnis.

Die Artifact-ID bildet dabei den technischen Namen des
Artifact-Verzeichnisses.

Grundstruktur:

```text
<root>/
    <artifact-id>/
        artifact.json
        tracks/
            <track-id>/
                chunks/
                    chunk-000001.*
                    chunk-000002.*
                    chunk-000003.*
```

Beispiel:

```text
recordings/
    artifact-001/
        artifact.json
        tracks/
            track-host/
                chunks/
                    chunk-000001.wav
                    chunk-000002.wav
                    chunk-000003.wav
            track-guest/
                chunks/
                    chunk-000001.wav
                    chunk-000002.wav
```

Das Artifact-Verzeichnis bildet damit die physische Persistenzgrenze
für ein einzelnes Recording Artifact.

---

# Artifact Directory

Jedes Recording Artifact besitzt genau ein eigenes
Artifact-Verzeichnis innerhalb des Persistence-Root.

Beispiel:

```text
<root>/
    artifact-001/
    artifact-002/
    artifact-003/
```

Die Verzeichnisse sind unabhängig voneinander.

Die Artifact-ID ist nicht aus dem Verzeichnisnamen abzuleiten.

Vielmehr bestimmt die Artifact-ID den Namen des Verzeichnisses.

Damit bleibt die logische Artifact-Identität unabhängig von der
Filesystem-Technologie.

---

# Artifact Metadata

Jedes Artifact-Verzeichnis enthält eine Metadatendatei:

```text
artifact.json
```

Diese Datei beschreibt das persistierte Recording Artifact.

Sie enthält mindestens die für die Wiederherstellung erforderlichen
Informationen:

* Artifact Identifier
* Recording Session Identifier
* Artifact Lifecycle Status
* technische Track-Informationen

Weitere technische Metadaten können später ergänzt werden.

Die Metadatendatei enthält keine Audiodaten.

---

# Track Representation

Jeder technische Track wird als eigenes Verzeichnis dargestellt.

Beispiel:

```text
tracks/
    track-host/
    track-guest/
```

Die Track-ID bildet den technischen Namen des Track-Verzeichnisses.

Die Track-ID ist eine technische Identität.

Sie ist nicht automatisch identisch mit:

* Teilnehmer-ID,
* Benutzer-ID,
* Rolle,
* Anzeigename.

Eine fachliche Zuordnung bleibt durch die entsprechenden
Architekturentscheidungen geregelt.

---

# Chunk Representation

Die Chunks eines Tracks werden in einem eigenen `chunks`-Verzeichnis
gespeichert.

Beispiel:

```text
track-host/
    chunks/
        chunk-000001.wav
        chunk-000002.wav
        chunk-000003.wav
```

Die Chunk-Nummer entspricht der technischen Sequenznummer des
`RecordingChunk`.

Die Nummerierung beginnt bei `000001`.

Die führenden Nullen dienen ausschließlich einer stabilen
lexikographischen Sortierung.

Die Dateireihenfolge ist jedoch nicht die fachliche Quelle der
Chunk-Reihenfolge.

Die Sequenznummer bleibt maßgeblich.

---

# Chunk Files

Ein Chunk wird als eigene physische Datei gespeichert.

Das konkrete Audioformat wird durch die bestehenden Audio- und
Recording-Entscheidungen bestimmt und nicht durch diese ADR neu
definiert.

Die Dateiendung muss daher dem tatsächlich verwendeten
Audioformat entsprechen.

Beispiel:

```text
chunk-000001.wav
```

Die Chunk-Datei enthält ausschließlich die Recording-Daten des
jeweiligen Chunks.

Artifact- und Track-Metadaten werden nicht in die Audiodatei
eingebettet, sofern dies nicht durch eine separate Entscheidung
vorgesehen wird.

---

# Metadata versus Recording Data

Die Persistenz trennt beschreibende Metadaten und eigentliche
Recording-Daten:

```text
Artifact Directory
        |
        +---- artifact.json
        |
        +---- tracks/
               |
               +---- track-001/
               |      |
               |      +---- chunks/
               |             +---- chunk-000001.wav
               |             +---- chunk-000002.wav
               |
               +---- track-002/
                      |
                      +---- chunks/
                             +---- chunk-000001.wav
```

Damit kann das Artifact beschrieben und wiedererkannt werden, ohne
die Audiodaten selbst in eine Metadatendatei einzubetten.

---

# Persistence Root

Der `FilesystemPersistenceProvider` erhält weiterhin ein Root-
Verzeichnis.

Dieses Root-Verzeichnis enthält ausschließlich die durch den Provider
verwalteten Artifact-Verzeichnisse.

Beispiel:

```text
<root>/
    artifact-001/
    artifact-002/
    artifact-003/
```

Der Provider darf keine fachlichen Verzeichnisstrukturen außerhalb
dieser Persistence Boundary voraussetzen.

---

# Atomic Persistence

Die Persistenz eines neuen oder veränderten Artifacts darf nicht
durch einen teilweise geschriebenen finalen Zustand sichtbar werden.

Insbesondere darf ein Absturz während des Schreibens nicht dazu
führen, dass ein unvollständiges Artifact-Verzeichnis als vollständig
persistiertes Artifact behandelt wird.

Die konkrete Implementierung soll deshalb einen temporären
Schreibzustand verwenden.

Beispiel:

```text
<root>/
    .artifact-001.tmp/
```

Nach erfolgreichem Abschluss der notwendigen Schreiboperationen wird
der persistierte Zustand als vollständig sichtbar gemacht:

```text
<root>/
    artifact-001/
```

Die genaue technische Umsetzung der atomaren Sichtbarkeit bleibt eine
Implementierungsfrage, sofern die beschriebene Eigenschaft erhalten
bleibt.

---

# Incomplete Chunks

Ein Chunk gilt erst dann als vollständig persistiert, wenn sein
Schreibvorgang erfolgreich abgeschlossen wurde.

Ein teilweise geschriebener Chunk darf nicht unter seinem finalen
Chunk-Dateinamen als vollständiger Chunk sichtbar sein.

Beispiel für einen temporären Zustand:

```text
chunks/
    .chunk-000004.wav.tmp
```

Nach erfolgreichem Abschluss:

```text
chunks/
    chunk-000004.wav
```

Dadurch kann Recovery zwischen:

* vollständig persistierten Chunks
* unvollständigen Schreibvorgängen

unterscheiden.

---

# Recovery

ADR-053 definiert die Recovery-Grenze zwischen Persistenz und
`LocalArtifactRegistry`.

ADR-055 ergänzt lediglich die physische Struktur, aus der ein
`FilesystemPersistenceProvider` Recording Artifacts wiederherstellen
kann.

Recovery darf dabei nicht allein aus der Existenz eines beliebigen
Unterverzeichnisses auf ein vollständig persistiertes Artifact
schließen.

Ein Artifact gilt als persistiert, wenn seine erforderliche
Metadatenstruktur vollständig vorhanden und lesbar ist.

Unvollständige temporäre Schreibzustände werden nicht als reguläre
Recording Artifacts behandelt.

---

# Listing

`FilesystemPersistenceProvider::list()` betrachtet die direkten
Artifact-Verzeichnisse unterhalb des Persistence-Root als
Kandidaten für persistierte Artifacts.

Temporäre Verzeichnisse und Dateien werden dabei ignoriert.

Ein fehlerhaftes oder unlesbares Artifact darf nicht dazu führen,
dass das Listing andere gültige Artifacts nicht mehr findet.

Das konkrete Fehlerverhalten bei beschädigten Artifacts wird durch
eine spätere Fehlerbehandlungsentscheidung präzisiert, sofern dies
erforderlich wird.

---

# Removal

Das Entfernen eines Recording Artifacts entfernt dessen vollständige
physische Persistenz:

```text
<root>/
    artifact-001/
```

wird vollständig entfernt.

Damit werden entfernt:

* `artifact.json`
* Track-Verzeichnisse
* Chunk-Verzeichnisse
* Chunk-Dateien
* sonstige zum Artifact gehörende persistierte Dateien

Das Entfernen eines Artifacts entfernt nicht automatisch andere
Artifacts.

---

# Identity and Paths

Filesystem-Pfade sind keine fachlichen Identitäten.

Insbesondere gilt:

```text
Artifact ID
    ≠
Filesystem Path
```

Der Pfad ist lediglich die physische Darstellung der Zuordnung.

Ein zukünftiger Persistence Provider kann dieselbe Artifact-ID in
einer vollständig anderen Speicherstruktur verwenden.

---

# Filename and Path Safety

Artifact- und Track-IDs dürfen nicht dazu führen, dass der
Persistence Root verlassen wird.

Die konkrete Implementierung muss deshalb sicherstellen, dass
technische IDs nicht ungeprüft als relative Pfade interpretiert
werden.

Insbesondere sind Pfadbestandteile wie:

```text
..
/
\
```

nicht als frei verwendbare Bestandteile einer technischen ID
zuzulassen.

Die genaue Validierungsstrategie ist Teil der Implementierung.

---

# Serialization Format

`artifact.json` verwendet JSON als Persistenzformat für die
beschreibenden Artifact-Metadaten.

Die genaue JSON-Struktur wird durch die Implementierung definiert.

Die JSON-Datei ist keine öffentliche API.

Eine spätere Änderung des internen Persistenzformats bleibt möglich,
sofern die Persistence Boundary nach außen unverändert bleibt.

---

# Consequences

## Positive Consequences

* Die physische Struktur entspricht der in ADR-054 definierten
  Artifact-/Track-/Chunk-Beziehung.
* Ein Artifact besitzt eine klar erkennbare physische Grenze.
* Recording-Daten und Metadaten bleiben getrennt.
* Einzelne Tracks können unabhängig voneinander verwaltet werden.
* Einzelne Chunks bleiben als Wiederherstellungseinheiten erhalten.
* Recovery kann vollständige und unvollständige Schreibzustände
  unterscheiden.
* Die Persistence Boundary bleibt erhalten.

## Negative Consequences

* Ein Artifact benötigt mehrere Dateien und Verzeichnisse.
* Die Persistenzlogik wird komplexer als bei einer einzelnen
  Metadatendatei.
* Atomic Writes müssen berücksichtigt werden.
* Recovery muss unvollständige Schreibzustände erkennen.
* Die Implementierung benötigt zusätzliche Tests.

Diese Konsequenzen werden bewusst akzeptiert.

---

# Nicht Teil dieser Entscheidung

Diese ADR entscheidet ausdrücklich nicht über:

* Chunk-Größe,
* Audio-Samplingrate,
* Bit-Tiefe,
* Audio-Codec,
* Masterformat,
* Synchronisationsprotokoll,
* Cloud Storage,
* Datenbankpersistenz,
* Verschlüsselung,
* Kompression,
* Garbage Collection,
* langfristige Archivierung.

Diese Themen bleiben bestehenden oder zukünftigen ADRs vorbehalten.

---

# Beziehung zu anderen ADRs

```text
ADR-003
Local Chunk-Based Audio Storage
        |
        v
ADR-042
Recording Artifact Model
        |
        v
ADR-054
Artifact → Track → Chunk
        |
        v
ADR-055
Filesystem Persistence Layout
        |
        v
FilesystemPersistenceProvider
```

Insbesondere konkretisiert ADR-055 die in ADR-052 ausdrücklich
offengelassenen Filesystem-Details.

ADR-053 verwendet die durch ADR-055 definierte Persistenzstruktur,
bleibt aber für die Recovery-Verantwortung maßgeblich.

---

# Implementierungsfolgen

Nach Annahme dieser ADR muss der
`FilesystemPersistenceProvider` angepasst werden.

Insbesondere betrifft dies:

* Artifact-Verzeichnis statt einzelner Artifact-Datei,
* `artifact.json`,
* Track-Verzeichnisse,
* Chunk-Verzeichnisse,
* Chunk-Dateien,
* Lesen und Schreiben der Track-/Chunk-Struktur,
* Umgang mit temporären Schreibzuständen,
* Listing der Artifact-Verzeichnisse,
* vollständiges Entfernen eines Artifacts.

Die bestehende `PersistenceProvider`-Schnittstelle bleibt dabei
unverändert.

---

# English Version ([German version above](#deutsch))

---

# Context

ADR-052 introduced the `FilesystemPersistenceProvider` as a concrete
implementation of the existing Persistence Boundary.

ADR-052 explicitly left the concrete filesystem layout open.

ADR-054 now defines the technical relationship:

```text
RecordingArtifact
    |
    +---- RecordingTrack
              |
              +---- RecordingChunk
              +---- RecordingChunk
              +---- ...
```

This defines which technical components may belong to an artifact.

The actual local persistence structure, however, still requires an
explicit decision.

The current implementation stores an artifact as a single file:

```text
<artifact-id>.json
```

This representation cannot fully represent the track/chunk structure
defined by ADR-054.

A concrete filesystem structure is therefore required.

---

# Decision

The `FilesystemPersistenceProvider` uses one dedicated directory for
each `RecordingArtifact`.

The artifact ID determines the technical name of that artifact
directory.

Basic structure:

```text
<root>/
    <artifact-id>/
        artifact.json
        tracks/
            <track-id>/
                chunks/
                    chunk-000001.*
                    chunk-000002.*
                    chunk-000003.*
```

Example:

```text
recordings/
    artifact-001/
        artifact.json
        tracks/
            track-host/
                chunks/
                    chunk-000001.wav
                    chunk-000002.wav
                    chunk-000003.wav
            track-guest/
                chunks/
                    chunk-000001.wav
                    chunk-000002.wav
```

The artifact directory therefore forms the physical persistence
boundary for one Recording Artifact.

---

# Artifact Directory

Each Recording Artifact has exactly one dedicated artifact directory
within the persistence root.

Example:

```text
<root>/
    artifact-001/
    artifact-002/
    artifact-003/
```

The directories are independent.

The artifact ID is not derived from the directory name.

Instead, the artifact ID determines the directory name.

This keeps logical artifact identity independent from filesystem
technology.

---

# Artifact Metadata

Each artifact directory contains:

```text
artifact.json
```

This file describes the persisted Recording Artifact.

It contains at least the information required for restoration:

* Artifact Identifier
* Recording Session Identifier
* Artifact Lifecycle Status
* technical track information

Additional technical metadata may be added later.

The metadata file does not contain audio data.

---

# Track Representation

Each technical track is represented by its own directory.

Example:

```text
tracks/
    track-host/
    track-guest/
```

The track ID determines the technical name of the track directory.

The track ID is a technical identity.

It is not automatically identical to:

* participant ID,
* user ID,
* role,
* display name.

Domain-level association remains governed by the corresponding
architecture decisions.

---

# Chunk Representation

Chunks belonging to a track are stored in a dedicated `chunks`
directory.

Example:

```text
track-host/
    chunks/
        chunk-000001.wav
        chunk-000002.wav
        chunk-000003.wav
```

The chunk number corresponds to the technical sequence number of the
`RecordingChunk`.

Numbering starts at `000001`.

Leading zeroes are used only to provide stable lexical sorting.

Filesystem order is not authoritative for chunk ordering.

The sequence number remains authoritative.

---

# Chunk Files

Each chunk is stored as an individual physical file.

The concrete audio format is governed by the existing audio and
recording decisions and is not redefined by this ADR.

The file extension must therefore correspond to the actual audio
format.

Example:

```text
chunk-000001.wav
```

The chunk file contains only the recording data for that chunk.

Artifact and track metadata are not embedded in the audio file unless
a separate decision explicitly requires this.

---

# Metadata versus Recording Data

Persistence separates descriptive metadata from the actual recording
data:

```text
Artifact Directory
        |
        +---- artifact.json
        |
        +---- tracks/
               |
               +---- track-001/
               |      |
               |      +---- chunks/
               |             +---- chunk-000001.wav
               |             +---- chunk-000002.wav
               |
               +---- track-002/
                      |
                      +---- chunks/
                             +---- chunk-000001.wav
                             +---- chunk-000002.wav
```

This allows the artifact to be described and identified without
embedding the recording data into the metadata file.

---

# Persistence Root

The `FilesystemPersistenceProvider` continues to receive a root
directory.

That root directory contains only artifact directories managed by
the provider.

Example:

```text
<root>/
    artifact-001/
    artifact-002/
    artifact-003/
```

The provider must not depend on domain-specific directory structures
outside this persistence boundary.

---

# Atomic Persistence

Persistence of a new or modified artifact must not expose a partially
written final state.

In particular, a crash during writing must not cause an incomplete
artifact directory to be treated as a fully persisted artifact.

The implementation should therefore use a temporary write state.

Example:

```text
<root>/
    .artifact-001.tmp/
```

After successful completion of the required write operations, the
persisted state becomes visible as:

```text
<root>/
    artifact-001/
```

The exact implementation of atomic visibility remains an
implementation concern as long as the described property is
preserved.

---

# Incomplete Chunks

A chunk is considered fully persisted only after its write operation
has completed successfully.

A partially written chunk must not become visible under its final
chunk filename as a complete chunk.

Example temporary state:

```text
chunks/
    .chunk-000004.wav.tmp
```

After successful completion:

```text
chunks/
    chunk-000004.wav
```

This allows recovery to distinguish between:

* fully persisted chunks,
* incomplete write operations.

---

# Recovery

ADR-053 defines the recovery boundary between persistence and
`LocalArtifactRegistry`.

ADR-055 only defines the physical structure from which a
`FilesystemPersistenceProvider` can restore Recording Artifacts.

Recovery must not infer a fully persisted artifact solely from the
existence of an arbitrary subdirectory.

An artifact is considered persisted when its required metadata
structure exists completely and can be read.

Incomplete temporary write states are not treated as regular
Recording Artifacts.

---

# Listing

`FilesystemPersistenceProvider::list()` considers direct artifact
directories below the persistence root as candidates for persisted
artifacts.

Temporary directories and files are ignored.

A broken or unreadable artifact must not prevent valid artifacts from
being listed.

Specific error handling for corrupted artifacts can be defined by a
later error-handling decision if required.

---

# Removal

Removing a Recording Artifact removes its complete physical
persistence:

```text
<root>/
    artifact-001/
```

is removed completely.

This includes:

* `artifact.json`,
* track directories,
* chunk directories,
* chunk files,
* other persisted files belonging to the artifact.

Removing one artifact does not automatically remove other artifacts.

---

# Identity and Paths

Filesystem paths are not domain identities.

In particular:

```text
Artifact ID
    ≠
Filesystem Path
```

The path is merely the physical representation of the association.

A future Persistence Provider may use the same artifact ID in a
completely different storage structure.

---

# Filename and Path Safety

Artifact and track IDs must not allow the persistence root to be
escaped.

The implementation must therefore ensure that technical IDs are not
interpreted as unrestricted relative paths.

In particular, path components such as:

```text
..
/
\
```

must not be accepted as freely usable components of a technical ID.

The exact validation strategy is an implementation concern.

---

# Serialization Format

`artifact.json` uses JSON as the persistence format for descriptive
artifact metadata.

The exact JSON structure is defined by the implementation.

The JSON file is not a public API.

A future change of the internal persistence format remains possible
as long as the external Persistence Boundary remains unchanged.

---

# Consequences

## Positive Consequences

* The physical structure matches the artifact/track/chunk
  relationship defined by ADR-054.
* Each artifact has a clearly identifiable physical boundary.
* Recording data and metadata remain separate.
* Individual tracks can be managed independently.
* Individual chunks remain available as recovery units.
* Recovery can distinguish complete and incomplete write states.
* The Persistence Boundary remains intact.

## Negative Consequences

* An artifact requires multiple files and directories.
* Persistence becomes more complex than a single metadata file.
* Atomic writes must be considered.
* Recovery must recognize incomplete write states.
* Additional implementation tests are required.

These consequences are explicitly accepted.

---

# Not Part of This Decision

This ADR explicitly does not decide:

* chunk size,
* audio sampling rate,
* bit depth,
* audio codec,
* master format,
* synchronization protocol,
* cloud storage,
* database persistence,
* encryption,
* compression,
* garbage collection,
* long-term archiving.

These topics remain governed by existing or future ADRs.

---

# Relationship to Other ADRs

```text
ADR-003
Local Chunk-Based Audio Storage
        |
        v
ADR-042
Recording Artifact Model
        |
        v
ADR-054
Artifact → Track → Chunk
        |
        v
ADR-055
Filesystem Persistence Layout
        |
        v
FilesystemPersistenceProvider
```

ADR-055 specifically resolves the filesystem details that ADR-052
left open.

ADR-053 uses the persistence structure defined by ADR-055 while
remaining authoritative for recovery responsibilities.

---

# Implementation Consequences

After acceptance of this ADR, the
`FilesystemPersistenceProvider` must be updated.

This includes:

* artifact directory instead of a single artifact file,
* `artifact.json`,
* track directories,
* chunk directories,
* chunk files,
* reading and writing the track/chunk structure,
* handling temporary write states,
* listing artifact directories,
* complete artifact removal.

The existing `PersistenceProvider` interface remains unchanged.
