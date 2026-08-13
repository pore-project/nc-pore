# ADR-060 Recording Artifact Processing Lifecycle and Idempotency

- Status: Accepted
- Date: 2026-08-13
- Related issue: #6 Recording → RecordingArtifact lifecycle

# Deutsch ([English version below](#english-version))

## Kontext

Die lokale Recording-Pipeline erzeugt aus einem abgeschlossenen Capture ein `RecordingArtifact` und persistiert dieses über den `PersistenceProvider`.

Die bestehenden Entscheidungen definieren die Grenzen zwischen Recording, Artifact-Erzeugung und Persistence. Für den Processing-Schritt müssen jedoch die konkreten Lifecycle-Semantiken für erfolgreiche Verarbeitung, Persistence-Fehler und wiederholte Verarbeitung desselben Artifacts eindeutig festgelegt werden.

Insbesondere muss definiert sein:

- wann ein Artifact als `Available` bzw. `Stored` gilt;
- was bei einem Fehler der Persistence geschieht;
- ob dasselbe Artifact wiederholt verarbeitet werden kann;
- wie mit einem bereits vorhandenen Artifact gleicher Identität umzugehen ist;
- ob ein bereits vorhandenes Artifact überschrieben werden darf.

## Entscheidung

Das lokale Artifact-Processing verwendet den Lifecycle:

```text
Created → Available → Stored
```

`Created` bedeutet, dass das `RecordingArtifact` erzeugt wurde.

`Available` bedeutet, dass das vollständige Artifact erfolgreich erzeugt wurde und für die weitere Verarbeitung verfügbar ist.

`Stored` bedeutet, dass die Persistence die erfolgreiche Speicherung des Artifacts bestätigt hat.

Ein Artifact darf daher erst nach erfolgreicher Persistence als `Stored` gelten.

### Persistence-Fehler

Ein Fehler bei der Persistence führt nicht zu einem zusätzlichen persistenten Artifact-Lifecycle-Zustand.

Wenn die Persistence für ein `Available`-Artifact fehlschlägt:

- bleibt das Artifact `Available`;
- liefert der Processing-Vorgang den Persistence-Fehler an seinen Aufrufer zurück;
- darf das Artifact nicht als `Stored` gemeldet werden.

Der Artifact-Lifecycle beschreibt damit den Zustand des Artifacts. Das Ergebnis des Processing-Vorgangs beschreibt unabhängig davon, ob der aktuelle Verarbeitungsvorgang erfolgreich war.

### Idempotentes Processing

Das Processing ist bezüglich der Identität eines `RecordingArtifact` idempotent.

Eine wiederholte Verarbeitung desselben Artifacts darf keinen anderen fachlichen Zustand erzeugen und kein bereits gültig persistiertes Artifact stillschweigend verändern.

### Bereits vorhandene Artifacts

Wenn bereits ein Artifact mit derselben Identität vorhanden ist:

- ist das vorhandene Artifact äquivalent zum zu verarbeitenden Artifact, gilt die Verarbeitung als erfolgreich und kann als No-op behandelt werden;
- ist das vorhandene Artifact nicht äquivalent, schlägt die Verarbeitung fehl;
- ein bereits vorhandenes gültiges Artifact wird niemals stillschweigend überschrieben.

Die Artifact-Identität allein berechtigt daher nicht dazu, bereits persistierte Daten zu ersetzen.

## Hier nicht entschieden

Diese ADR definiert nicht:

- die konkrete technische Methode zur Feststellung der Äquivalenz zweier Artifacts;
- Recovery-Semantik;
- kryptographische Integritätsprüfung;
- automatische Reparatur persistierter Artifacts;
- Remote-Synchronisation;
- Archivierungszustände;
- Fehlerzustände oder Retry-Semantik von Remote- bzw. Synchronisationsvorgängen.

Diese Themen bleiben den jeweils zuständigen Architekturentscheidungen bzw. späteren Entscheidungen vorbehalten.

## Konsequenzen

- Der lokale Artifact-Lifecycle besitzt ein kleines und eindeutig definiertes Zustandsmodell.
- Ein Persistence-Fehler erfordert keinen zusätzlichen `Failed`-Zustand für das Artifact.
- Ein fehlgeschlagenes Processing kann sicher erneut versucht werden.
- Bereits korrekt persistierte Daten werden bei wiederholtem Processing nicht stillschweigend verändert.
- Identische bereits vorhandene Artifacts können idempotent behandelt werden.
- Widersprüchliche Artifacts mit derselben Identität werden als Fehler behandelt.
- Artifact-Zustand und Processing-Fehler bleiben voneinander getrennte Konzepte.

# English Version ([Deutsche Version oben](#deutsch))

## Context

The local recording pipeline creates a `RecordingArtifact` from a completed capture and persists it through the `PersistenceProvider`.

The existing decisions define the boundaries between recording, artifact creation, and persistence. The concrete lifecycle semantics for successful processing, persistence failures, and repeated processing of the same artifact must nevertheless be defined explicitly.

In particular, the following must be specified:

- when an artifact is considered `Available` or `Stored`;
- what happens when persistence fails;
- whether the same artifact can be processed repeatedly;
- how an already existing artifact with the same identity is handled;
- whether an existing artifact may be overwritten.

## Decision

Local artifact processing uses the following lifecycle:

```text
Created → Available → Stored
```

`Created` means that the `RecordingArtifact` has been created.

`Available` means that the complete artifact has been created successfully and is available for further processing.

`Stored` means that persistence has confirmed successful storage of the artifact.

An artifact therefore must not be considered `Stored` before persistence has succeeded.

### Persistence failures

A persistence failure does not introduce an additional persistent artifact lifecycle state.

If persistence fails for an `Available` artifact:

- the artifact remains `Available`;
- the processing operation returns the persistence error to its caller;
- the artifact must not be reported as `Stored`.

The artifact lifecycle therefore describes the state of the artifact, while the result of the processing operation independently describes whether the current processing operation succeeded.

### Idempotent processing

Processing is idempotent with respect to the identity of a `RecordingArtifact`.

Repeated processing of the same artifact must not create a different semantic state or silently modify an already valid persisted artifact.

### Existing artifacts

If an artifact with the same identity already exists:

- if the existing artifact is equivalent to the artifact being processed, processing succeeds and may be treated as a no-op;
- if the existing artifact is not equivalent, processing fails;
- an already existing valid artifact is never silently overwritten.

Artifact identity alone therefore does not authorize replacement of already persisted data.

## Not decided here

This ADR does not define:

- the concrete technical method for determining equivalence between two artifacts;
- recovery semantics;
- cryptographic integrity verification;
- automatic repair of persisted artifacts;
- remote synchronization;
- archival states;
- error states or retry semantics for remote or synchronization operations.

Those concerns remain subject to the respective architectural decisions or to later decisions.

## Consequences

- The local artifact lifecycle has a small and explicitly defined state model.
- A persistence failure does not require an additional `Failed` state for the artifact.
- Failed processing can safely be retried.
- Already valid persisted data is not silently modified by repeated processing.
- Identical existing artifacts can be handled idempotently.
- Conflicting artifacts with the same identity are treated as errors.
- Artifact state and processing errors remain separate concepts.
