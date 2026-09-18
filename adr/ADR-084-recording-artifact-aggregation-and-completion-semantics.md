# ADR-084 Recording Artifact Aggregation and Completion Semantics

* Status: Accepted
* Date: 2026-09-18
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

---

# Kontext

Ein fachliches PoRE-Recording ist keine einzelne Audiodatei. Bei einer Mehrteilnehmer-Aufnahme entsteht für jeden erwarteten Teilnehmer ein eigenes lokales und später remote bestätigtes Audio-Artefakt.

Die bisherige technische Modellierung mit einer einzelnen `artifact_id` am Recording bildet diese Realität nicht ausreichend ab. Gleichzeitig darf die Fertigstellung eines einzelnen Teilnehmer-Artefakts nicht mit der Fertigstellung des gesamten Recordings oder der Production verwechselt werden.

Insbesondere müssen folgende Zustände unabhängig voneinander darstellbar sein:

- ein Teilnehmer hat sein Artefakt noch nicht vollständig übertragen;
- mehrere andere Teilnehmer-Artefakte sind bereits verfügbar;
- das Recording ist deshalb noch nicht vollständig;
- die Production kann bereits fachlich geschlossen sein.

# Entscheidung

# Recording ist die logische Mehrspur-Aufnahme

`core.Recording` repräsentiert die **logische Aufnahme** und nicht eine einzelne Datei.

Ein Recording besteht V1 aus einer Menge erwarteter Teilnehmer-Artefakte:

```text
core.Recording
   |
   +-- participant A -> artifact.ArtifactId
   +-- participant B -> artifact.ArtifactId
   +-- participant C -> artifact.ArtifactId
   +-- participant D -> artifact.ArtifactId
```

Die stabile Artefaktidentität bleibt `artifact.ArtifactId`. Ein erneuter Uploadversuch erzeugt keine neue fachliche Artefaktidentität.

# Erwartete Teilnehmer werden für das Recording festgelegt

Für V1 wird die Menge der für ein Recording erwarteten Teilnehmer mit dem fachlichen Recording-Start festgelegt.

- Ein Teilnehmer, der zu diesem Zeitpunkt erwartet wird, bleibt für dieses Recording erwartet, auch wenn er später den Talk-Raum verlässt.
- Ein Teilnehmer, der erst nach dem Recording-Start hinzukommt, wird nicht nachträglich zu einem erwarteten Track dieses laufenden Recordings.
- Das Verlassen des Talk-Raums entfernt keinen bereits erwarteten Artefakt-Slot.

Damit ist eindeutig, welche Artefakte für ein Recording noch ausstehen.

# Pro erwarteten Teilnehmer ein Artefakt-Slot

V1 führt für jeden erwarteten Teilnehmer eines Recordings genau einen fachlichen Artefakt-Slot.

Konzeptionell gilt:

```text
core.Recording + core.Participant
          |
          v
     artifact.ArtifactId
```

Der Slot kann technisch zunächst noch ohne vollständig bestätigtes Remote-Artefakt sein. Das Artefakt darf sich unabhängig vom Recording- und Production-Zustand weiterentwickeln.

# Recording.Completed bedeutet vollständige Artefaktabdeckung

Ein Recording ist fachlich erst dann **Completed**, wenn für **alle erwarteten Teilnehmer** ein zugehöriges Artefakt existiert und dessen serverseitige Übernahme fachlich/technisch bestätigt ist.

Die maßgebliche Kette ist:

```text
Participant Artifact
    -> local preservation
    -> verified remote transport
    -> artifact.ArtifactId confirmed
    -> all expected participant artifacts confirmed
    -> core.Recording.Completed
```

Ein erfolgreich abgeschlossener Transport eines einzelnen Teilnehmer-Artefakts bedeutet daher **nicht automatisch**, dass das gesamte Recording abgeschlossen ist.

# Teilmengen sind erlaubt, aber nicht vollständig

Während ein Recording noch auf Teilnehmer-Artefakte wartet, darf PoRE bereits mit der vorhandenen Teilmenge arbeiten.

Beispiel:

```text
Alice   ✓
Carol   ✓
Dave    ✓
Bob     pending
```

Diese Teilmenge ist ein **partieller Arbeitsstand**. Sie ist weder ein vollständig abgeschlossenes Recording noch darf sie stillschweigend als vollständiges Mehrspurprojekt ausgegeben werden.

Eine vollständige Mehrspur-Repräsentation bzw. ein vollständiger Export setzt ein `core.Recording.Completed` voraus.

# Recording-, Artifact- und Production-Fertigstellung bleiben getrennt

Die folgenden Aussagen sind bewusst verschieden:

- `artifact` vollständig bestätigt = ein Teilnehmer-Artefakt ist fertig;
- `core.Recording.Completed` = alle erwarteten Teilnehmer-Artefakte dieses Recordings sind fertig;
- `core.Production.Completed` = die fachliche Production wurde geschlossen.

Insbesondere folgt aus `core.Production.Completed` **nicht**, dass jedes Recording-Artefakt bereits vollständig remote vorliegt.

Die Semantik der Production-Schließung und die Zulässigkeit später Artefakt-Nachlieferung sind in ADR-085 festgelegt.

# Consequences

- Das Recording-Modell kann Mehrspur-Aufnahmen mit individuellen Teilnehmer-Artefakten korrekt repräsentieren.
- Ein einzelnes `artifact_id` am Recording ist für die fachliche V1-Semantik nicht ausreichend.
- Ein einzelner Teilnehmer kann noch ausstehen, ohne die bereits bestätigten Artefakte anderer Teilnehmer zu invalidieren.
- Teilweise verfügbare Tracks dürfen als Arbeitsgrundlage verwendet werden, müssen aber als unvollständig erkennbar bleiben.
- Ein vollständiger Export kann eindeutig an `core.Recording.Completed` gebunden werden.
- Late Artifact Completion kann ein zuvor unvollständiges Recording noch zu `Completed` führen, ohne die Production erneut zu öffnen.

# Future Considerations

Diese ADR legt nicht fest:

- die konkrete Datenbank- oder Struct-Repräsentation der Artefakt-Slots;
- die konkrete HTTP-/WebDAV-Transporttechnik;
- die Aufbewahrungsdauer lokaler oder remote gespeicherter Artefakte;
- die Benutzeroberfläche für partielle Arbeitsstände;
- die konkrete Exportimplementierung.

Diese Themen bleiben den jeweiligen technischen bzw. produktbezogenen Entscheidungen vorbehalten.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# Context

A PoRE Recording is a logical multi-track recording, not a single audio file. A multi-participant recording creates one participant-specific audio artifact per expected participant.

The former representation with one `artifact_id` on the Recording is therefore insufficient for the actual multi-track semantics. At the same time, completion of one participant artifact must not be confused with completion of the complete Recording or the Production.

# Decision

`core.Recording` represents the **logical recording**, not one file.

For V1, a Recording consists of one expected artifact slot per participant. The stable artifact identity remains `artifact.ArtifactId`; retries never create a second semantically distinct artifact.

The expected participant set is fixed when the fachlich recording starts. Participants expected at that point remain expected even if they later leave the Talk room. Participants joining after recording start are not retroactively added to the active Recording.

Each expected participant has exactly one fachlich defined artifact slot for that Recording.

A Recording is **Completed** only when every expected participant artifact exists and its server-side receipt has been successfully confirmed.

A Recording may therefore temporarily have a partial working set of confirmed artifacts. Such a partial set is valid for processing work, but it is not a complete Recording and must not be silently presented as a complete multi-track export.

The following states remain distinct:

- individual artifact completion;
- `core.Recording.Completed`;
- `core.Production.Completed`.

Production completion therefore does not imply that every participant artifact is already complete. Late artifact completion is defined separately by ADR-085.

# Consequences

The domain model can represent a multi-track Recording correctly, partial working sets remain possible, full exports can be tied to `core.Recording.Completed`, and a late participant artifact may complete an earlier Recording without reopening the Production.

# Future Considerations

This ADR does not define the concrete database/struct representation, transport technology, retention policy, partial-work UI, or export implementation.
