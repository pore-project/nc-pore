# ADR-057: Domain Recording to Recording Artifact Association Boundary

* Status: Accepted
* Date: 2026-08-12
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

---

# Kontext

NC-PoRe unterscheidet zwischen dem fachlichen `Recording` im Core und
der technischen `RecordingSession` sowie dem technischen
`RecordingArtifact` im Recorder.

Die bestehenden Entscheidungen definieren bereits die Verarbeitungskette:

```text
ProductionSession / Recording
            |
            v
     Recorder boundary
            |
            v
    RecordingSession
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

Bislang konnte ein persistiertes Artifact jedoch nur seiner lokalen
`RecordingSession` zugeordnet werden. Die fachliche Herkunft des
Artifacts war damit nicht vollständig nachvollziehbar.

Gleichzeitig darf der Recorder nicht von den fachlichen Core-Typen
abhängen und `CaptureResult` darf gemäß ADR-056 keine fachliche Domain-
Logik enthalten.

---

# Entscheidung

Die Zuordnung zwischen fachlichem Recording und technischem
`RecordingArtifact` wird an der Application-/Processing-Grenze hergestellt.

Die fachlichen Identifikatoren werden dabei als **opaque values** an den
Recorder übergeben. Der Recorder benötigt dafür keine Abhängigkeit auf
den Core-Crate und interpretiert die fachlichen IDs nicht.

Die technische Zuordnung lautet:

```text
ProductionId
     |
RecordingId
     |
     v
RecordingArtifactAssociation
     |
     +---- RecordingSessionId
     |
     +---- ArtifactId
```

`RecordingSessionId` und `ArtifactId` bleiben technische Recorder-IDs.
`ProductionId` und `RecordingId` bleiben fachliche IDs des Core. Die
Verbindung wird durch die Application-/Processing-Grenze hergestellt.

Die Association wird als Metadatenbestandteil des `RecordingArtifact`
erhalten und von der Persistence wiederhergestellt.

---

# Verantwortlichkeiten

Der Core entscheidet:

* welches `Recording` zu welcher `ProductionSession` gehört,
* ob das Recording abgeschlossen ist,
* wann die technische Aufnahme gestartet oder abgeschlossen werden darf.

Der Recorder entscheidet:

* wie das Capture-Ergebnis in ein `RecordingArtifact` überführt wird,
* wie die technische Artifact-Struktur aufgebaut ist,
* wie das Artifact persistiert und wiederhergestellt wird.

Die Application-/Processing-Grenze übergibt die fachliche Herkunft als
opaque identifiers.

---

# Keine Core-Abhängigkeit des Recorders

Der Recorder importiert weder `ProductionId` noch `RecordingId` aus dem
Core.

Stattdessen werden deren Werte als Strings an die Boundary übergeben.

Damit bleibt die Richtung der Abhängigkeiten erhalten:

```text
Core
  |
  | fachliche IDs
  v
Application / Processing Boundary
  |
  | opaque values
  v
Recorder
```

Die technische Capture-Schicht bleibt vollständig frei von dieser
fachlichen Zuordnung.

---

# Persistenz

Die Association ist Bestandteil der Artifact-Metadaten.

Damit bleibt nach einem Neustart nachvollziehbar:

```text
ProductionId
     |
RecordingId
     |
RecordingSessionId
     |
ArtifactId
```

Die bestehende Filesystem-Struktur aus ADR-055 bleibt unverändert.
Insbesondere wird keine zusätzliche fachliche Verzeichnisstruktur
eingeführt.

Die fachlichen IDs werden in `artifact.json` als Metadaten gespeichert.

---

# Konsequenzen

## Positive Konsequenzen

* Ein persistiertes Artifact kann auf sein fachliches Recording
  zurückgeführt werden.
* Die Beziehung bleibt über Neustarts und Recovery erhalten.
* Core und Recorder bleiben als Crates unabhängig.
* `CaptureResult` bleibt frei von fachlicher Domain-Logik.
* Die bestehende Artifact- und Persistence-Struktur bleibt erhalten.

## Negative Konsequenzen

* Das Artifact-Metadatenmodell enthält zusätzlich zwei opaque fachliche
  Referenzen.
* Die Application-/Processing-Grenze muss die fachlichen IDs liefern.
* Alte Artifacts ohne Association bleiben zulässig und besitzen keine
  fachliche Rückverfolgbarkeit.

---

# Beziehung zu anderen ADRs

* ADR-039 Recording Architecture and Capture Boundary
* ADR-040 Recorder Workflow and Capture Lifecycle Coordination
* ADR-042 Recording Artifact Model and Lifecycle Boundary
* ADR-051 Recording Artifact Processing Boundary
* ADR-054 Recording Artifact and Local Recording Data Association
* ADR-055 Filesystem Persistence Layout
* ADR-056 Capture Result and Recording Artifact Data Boundary

---

# Ergebnis

Ein abgeschlossenes Recording aus einer ProductionSession kann über die
bestehende Recorder-Grenze in ein persistiertes RecordingArtifact
überführt werden, ohne die fachliche Core-Logik in den Recorder zu
ziehen. Die fachliche Herkunft bleibt als explizite Association am
technischen Artifact nachvollziehbar.

---

# English Version ([German version above](#deutsch))

---

# Context

NC-PoRe separates the domain `Recording` in core from the technical
`RecordingSession` and `RecordingArtifact` in the recorder.

The existing decisions already define the processing chain, but a
persisted artifact could previously only be associated with its local
`RecordingSession`. Its domain origin was therefore not fully
traceable.

At the same time, the recorder must not depend on core domain types and
`CaptureResult` must remain free of domain logic according to ADR-056.

---

# Decision

The association between a domain Recording and a technical
`RecordingArtifact` is established at the application/processing
boundary.

Domain identifiers cross this boundary as opaque values. The recorder
does not depend on the core crate and does not interpret the domain IDs.

The association is retained as artifact metadata and restored by the
persistence layer.

The existing filesystem layout from ADR-055 remains unchanged; no
additional domain-specific directory hierarchy is introduced.

---

# Consequences

* A persisted artifact can be traced back to its domain Recording.
* The relationship survives restart and recovery.
* Core and recorder remain independent crates.
* CaptureResult remains free of domain logic.
* Existing artifacts without an association remain valid.

---

# Result

A completed Recording from a ProductionSession can cross the existing
recorder boundary into a persisted RecordingArtifact while retaining an
explicit, recoverable association to its domain origin.
