# Deutsch ([English version below](#english-version))

# ADR-059 Recording Payload Filesystem Persistence

- Status: Accepted
- Date: 2026-08-12
- Related issue: #7 Persist actual recording payload

## Kontext

ADR-055 definiert die physische RecordingArtifact-Grenze und die Dateisystemstruktur Artifact → Track → Chunk. ADR-058 definiert eine vom Storage-Provider unabhängige logische Payload-Referenz und verschiebt die konkrete Payload-Persistence ausdrücklich auf #7.

Der Recorder überträgt inzwischen tatsächliche Payload-Bytes von `CaptureResult` in `RecordingArtifact`. Der Dateisystem-Provider benötigt daher eine konkrete, provider-lokale Repräsentation dieser Bytes, ohne damit eine Entscheidung für einen bestimmten Audio-Codec oder ein bestimmtes Format einzuführen.

## Entscheidung

Der `FilesystemPersistenceProvider` persistiert das Payload jedes `RecordingChunk` als eine einzelne opake Payload-Datei unterhalb des Chunk-Verzeichnisses:

```text
<root>/
    <artifact-id>/
        artifact.json
        tracks/
            <track-id>/
                chunks/
                    chunk-000001.payload
                    chunk-000002.payload
```

Das Suffix `.payload` behauptet bewusst keinen Audio-Codec und kein Containerformat. Die Bytes werden exakt so persistiert, wie sie von der Capture-Schicht geliefert werden. Eine zukünftige Entscheidung über das Audioformat kann diese physische Repräsentation ersetzen, ohne die logische Payload-Referenzgrenze aus ADR-058 zu verändern.

`artifact.json` speichert die logische Payload-Referenz und die Payload-Größe für jeden Chunk. Die Payload-Bytes selbst verbleiben in der Chunk-Payload-Datei und werden nicht in JSON eingebettet.

Ein vollständiges Artifact wird zunächst in einem temporären Artifact-Verzeichnis geschrieben. Jede Payload-Datei wird innerhalb dieser temporären Struktur geschrieben und veröffentlicht. Anschließend wird das vollständige temporäre Verzeichnis in das finale Artifact-Verzeichnis umbenannt. Das finale Artifact-Verzeichnis wird daher erst sichtbar, nachdem Metadaten und Payload-Dateien geschrieben wurden.

Beim Laden eines Artifacts muss jede deklarierte Payload-Datei vorhanden sein und die deklarierte Größe besitzen. Ein fehlendes oder in seiner Größe abweichendes Payload macht das Artifact daher für den aktuellen Load-/List-Pfad des Providers nicht verfügbar.

## Hier nicht entschieden

Diese ADR definiert nicht:

- einen konkreten Audio-Codec oder ein Containerformat;
- einen Checksum- oder Hashing-Algorithmus;
- Recovery-Semantik bei Beschädigungen über die Ablehnung unvollständiger Payloads hinaus;
- Remote- oder Datenbank-Persistence;
- Payload-Verschlüsselung oder -Kompression.

Diese Themen bleiben späteren Entscheidungen vorbehalten, insbesondere #8 für Integritäts- und Recovery-Validierung.

## Konsequenzen

- Der aktuelle Dateisystem-Provider persistiert tatsächliche Recording-Bytes.
- Payload-Metadaten und Payload-Bytes bleiben physisch getrennt.
- Die bestehende `PersistenceProvider`-Schnittstelle bleibt unverändert.
- Das Artifact-Modell bleibt unabhängig von absoluten Dateisystempfaden.
- Fehlende oder gekürzte Payload-Dateien können nicht stillschweigend zu einem geladenen Artifact führen.
- Die aktuelle `.payload`-Repräsentation bleibt bewusst formatneutral.

# English Version ([Deutsche Version oben](#deutsch))

# ADR-059 Recording Payload Filesystem Persistence

- Status: Accepted
- Date: 2026-08-12
- Related issue: #7 Persist actual recording payload

## Context

ADR-055 defines the physical RecordingArtifact boundary and the Artifact → Track → Chunk filesystem structure. ADR-058 defines a storage-provider-independent logical payload reference and explicitly defers concrete payload persistence to #7.

The recorder now carries actual payload bytes from `CaptureResult` into `RecordingArtifact`. The filesystem provider therefore needs a concrete, provider-local representation for those bytes without introducing a specific audio codec or format decision.

## Decision

The `FilesystemPersistenceProvider` persists each RecordingChunk payload as one opaque payload file below the chunk directory:

```text
<root>/
    <artifact-id>/
        artifact.json
        tracks/
            <track-id>/
                chunks/
                    chunk-000001.payload
                    chunk-000002.payload
```

The `.payload` suffix deliberately does not claim an audio codec or container format. The bytes are persisted exactly as supplied by the capture layer. A future audio-format decision may replace this physical representation without changing the logical payload reference boundary from ADR-058.

`artifact.json` stores the logical payload reference and payload size for each chunk. The payload bytes themselves remain in the chunk payload file and are not embedded into JSON.

A complete artifact is written into a temporary artifact directory first. Each payload file is written and published inside that temporary structure. The complete temporary directory is then renamed to the final artifact directory. Therefore the final artifact directory is not exposed until its metadata and payload files have been written.

Loading an artifact requires every declared payload file to exist and to have the declared size. A missing or size-mismatched payload therefore makes the artifact unavailable to the current provider load/list path.

## Not decided here

This ADR does not define:

- a concrete audio codec or container format;
- a checksum or hashing algorithm;
- corruption recovery semantics beyond rejecting incomplete payloads;
- remote or database persistence;
- payload encryption or compression.

Those concerns remain subject to later decisions, in particular #8 for integrity and recovery validation.

## Consequences

- The current filesystem provider persists actual recording bytes.
- Payload metadata and payload bytes remain physically separate.
- The existing `PersistenceProvider` interface remains unchanged.
- The artifact model remains independent of absolute filesystem paths.
- Missing or truncated payload files cannot silently produce a loaded artifact.
- The current `.payload` representation is intentionally format-neutral.
