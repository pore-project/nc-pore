# ADR-085: Production Completion, Exceptional Closure and Late Artifact Delivery

## Status

Accepted

## Date

2026-09-18

## Decision Type

Architecture

## Related

- ADR-065 — Storage Provider and Retention Policy Boundaries
- ADR-083 — Verified Nextcloud Artifact Transport
- ADR-084 — Recording Artifact Aggregation and Completion Semantics

---

# Deutsch

## Kontext

PoRE muss zwischen dem fachlichen Ende einer Production und der technischen Fertigstellung einzelner Recording-Artefakte unterscheiden.

Nach dem Stop einer Aufnahme können Teilnehmer bereits vollständig lokal gesichert haben, während einzelne Uploads noch ausstehen. Ein Host muss die Production trotzdem kontrolliert schließen können, ohne auf unbestimmte Zeit auf einen einzelnen fehlenden Teilnehmer warten zu müssen.

Gleichzeitig darf eine fachliche Production-Schließung einen bereits entstandenen Recording-Artefakt nicht entwerten. Ein Teilnehmer kann die Verbindung für lange Zeit verlieren und erst viel später wieder verfügbar werden.

Die folgenden Fälle müssen deshalb gleichzeitig möglich sein:

1. alle erwarteten Artefakte treffen rechtzeitig ein;
2. ein Artefakt fehlt und die Production wird nach Ablauf der Wartefrist automatisch geschlossen;
3. der Host schließt die Production vorzeitig manuell;
4. ein fehlendes Artefakt wird erst deutlich später geliefert;
5. die spätere Lieferung vervollständigt das Recording, ohne die bereits geschlossene Production wieder zu öffnen.

## Entscheidung

### 1. Production.Completed ist fachliche Schließung, keine Daten-Ablauffrist

`core.Production.Completed` bedeutet:

> Die Production nimmt keine weitere fachliche Produktionsaktivität mehr auf.

Es bedeutet **nicht**:

> Alle technischen Artefakte müssen zu diesem Zeitpunkt bereits vorhanden sein.

Die Production-Schließung ist insbesondere **keine Retention Policy und keine Ablaufzeit für Artefakte**. Retention und tatsächliche Datenlöschung bleiben getrennte Storage-/Betriebsentscheidungen gemäß ADR-065.

### 2. Eine Production kann auf drei Arten geschlossen werden

Eine Production wird fachlich abgeschlossen, wenn einer der folgenden Fälle eintritt:

1. **Alle relevanten Recordings sind vollständig abgeschlossen.**
2. **Die konfigurierte Wartefrist für noch ausstehende Artefakte ist abgelaufen.**
3. **Der Host fordert einen manuellen Force-Close an.**

Für V1 gilt als fachlicher Default eine Wartefrist von **24 Stunden**. Diese Frist ist eine Wartefrist für die Production-Schließung, keine Artefakt-Ablaufzeit. Eine spätere Änderung des konkreten Fristwerts ist eine Betriebs-/Produktkonfiguration und verändert diese Trennung nicht.

Die Wartefrist wird anhand ausstehender Artefakte bewertet. Für ein ausstehendes Artefakt beginnt die maßgebliche Frist mit dem fachlichen Stop des zugehörigen Recordings.

### 3. Die Entscheidung über Production.Completed liegt beim Core

Der Core ist die fachliche Autorität für `core.Production.Completed`.

Ein Scheduler, Browser, Connector oder Hintergrundprozess darf lediglich die Prüfung bzw. den Abschluss anstoßen. Er darf nicht selbst entscheiden, dass die Production fachlich beendet ist.

Insbesondere darf die technische Feststellung

```text
"24 Stunden sind vergangen"
```

nicht außerhalb des Core zu einer eigenständigen Production-State-Transition führen.

### 4. Automatischer Abschluss bei vollständigen Recordings

Sind alle relevanten Recordings vollständig abgeschlossen, kann der Core die Production ohne weitere Wartefrist fachlich schließen.

Damit ist der Normalfall:

```text
Recording Stop
    ->
all expected participant artifacts confirmed
    ->
core.Recording.Completed
    ->
core.Production.Completed
```

### 5. Timeout beendet das Warten, nicht die Nachlieferbarkeit

Erreicht ein ausstehendes Artefakt seine 24-Stunden-Wartefrist, darf die Production fachlich abgeschlossen werden, obwohl das zugehörige Recording noch nicht `Completed` ist.

Der ausstehende Artefakt-Slot bleibt dabei gültig.

Es gilt ausdrücklich:

> **Beendigung einer Production beendet niemals die Nachlieferbarkeit eines bereits entstandenen Recording-Artefakts.**

Timeout bedeutet daher:

```text
"Wir warten nicht länger auf die Vervollständigung der Production."
```

und nicht:

```text
"Dieses Artefakt ist jetzt ungültig."
```

### 6. Host Force-Close

Der Host darf die Production vor Ablauf der Wartefrist manuell schließen, wenn er weiß, dass eine weitere Wartezeit fachlich nicht sinnvoll ist.

Force-Close hat dieselbe wichtige Grenze wie der Timeout:

- es beendet die Production;
- es verwirft kein bereits entstandenes Artefakt;
- es löscht keine lokale Preservation;
- es widerruft nicht die stabile Artefaktidentität;
- es verhindert keine spätere Übertragung.

### 7. Late Artifact Completion ist ausdrücklich zulässig

Ein bereits entstandenes, lokal sicher erhaltenes und noch nicht remote bestätigtes Artefakt bleibt auch nach `core.Production.Completed` zur Nachlieferung berechtigt.

Dabei gelten drei voneinander unabhängige Zeitachsen:

```text
LIVE
Talk / Recording
    |
    +--> Recording Stop

PRODUCTION
Production Active
    |
    +--> Completed (normal / timeout / force-close)

ARTIFACT
local preserved
    |
    +--> pending
    |
    +--> remote completion — ggf. lange später
```

Eine verspätete Artefaktlieferung darf das zugehörige Recording von `Stopped` nach `Completed` überführen, sofern das Artefakt korrekt und eindeutig dem Recording zugeordnet ist.

Sie darf dagegen niemals

```text
core.Production.Completed -> Active
```

bewirken.

**Late Artifact Completion öffnet die Production nicht erneut.**

### 8. Temporäre Transport-Autorisierung ist kein Artefaktablauf

Ein für den Upload verwendeter Transport-Handle, Share, Token, URL oder eine andere temporäre Berechtigung darf ablaufen.

Das bedeutet lediglich, dass für die spätere Nachlieferung erneut ein Transport vorbereitet werden muss.

Konzeptionell:

```text
stable artifact identity
        |
        +--> old transport authorization expired
        |
        +--> prepare again
        |
        +--> fresh authorization
        |
        +--> verified upload
        |
        +--> artifact completed
```

Die stabile Artefaktidentität bleibt dabei unverändert.

### 9. Konkretes Extrembeispiel

Ein Teilnehmer kann seine Aufnahme lokal vollständig abgeschlossen haben, kurz danach aber für sehr lange Zeit unerreichbar sein.

Selbst Monate später muss eine erneute Anmeldung desselben Teilnehmers die bereits vorhandene, unveränderte lokale Completion-Arbeit wiederaufnehmen können. Die damals geschlossene Production ist dabei weiterhin abgeschlossen.

Das Architekturziel ist damit nicht "24 Stunden Aufbewahrung", sondern:

> **24 Stunden maximale fachliche Wartezeit für die Production – ohne Ablauf der Nachlieferbarkeit eines bereits entstandenen Artefakts.**

### 10. Completion-Grund wird fachlich unterschieden

Die Ursache der Production-Schließung ist Teil der fachlichen Historie.

V1 definiert konzeptionell:

```text
core.ProductionCompletionReason
    = AllRecordingsCompleted
    | ArtifactCompletionTimeout
    | HostForced
```

Die konkrete Persistenz- und Auditdarstellung bleibt eine Implementierungsentscheidung, darf diese fachliche Unterscheidung aber nicht verlieren.

## Konsequenzen

- Production-Abschluss und Artifact-Abschluss sind unabhängig modellierbar.
- Ein Timeout kann die Production schließen, ohne ein fehlendes Artefakt zu verwerfen.
- Der Host kann die Production manuell schließen, ohne auf einen fehlenden Teilnehmer warten zu müssen.
- Späte Uploads bleiben möglich, auch deutlich nach Production-Abschluss.
- Die Production wird durch späte Uploads niemals wieder geöffnet.
- Transport-Autorisierungen dürfen kurzlebig sein; die Artefaktidentität ist es nicht.
- Storage-Retention und fachliche Production-Schließung bleiben sauber getrennt.

## Nicht durch diese ADR entschieden

Diese ADR legt nicht fest:

- den konkreten Scheduler-Mechanismus;
- die konkrete API für Force-Close;
- die konkrete Berechtigungsaktion für Late Artifact Completion;
- die konkrete Datenbankstruktur für Completion Reasons;
- eine universelle Storage-Retention;
- eine technische maximale Lebensdauer des lokalen Browser-Speichers.

---

# English Version

## Context

PoRE must distinguish fachlich closing a Production from technical completion of individual Recording Artifacts.

After Recording Stop, some participants may have fully preserved and uploaded their artifacts while another participant remains offline. The Host must still be able to close the Production deliberately.

At the same time, closing the Production must never invalidate an already created Recording Artifact.

## Decision

`core.Production.Completed` means fachliche production closure, not proof that all artifacts have already arrived.

A Production may close when:

1. all relevant Recordings are complete;
2. the configured artifact-completion wait period expires; or
3. the Host explicitly force-closes the Production.

The V1 default wait period is **24 hours**. It is a production waiting deadline, not an artifact expiration time.

The Core is authoritative for the Production transition. Schedulers and technical components may trigger a check, but they do not own the fachliche decision.

A timeout or Host Force-Close does not invalidate any already created artifact, does not delete preservation data, does not remove stable artifact identity, and does not prevent later delivery.

A pending artifact remains deliverable after `core.Production.Completed`. A reconnecting client may prepare transport again and obtain fresh temporary authorization when an older token/share/URL has expired.

Late completion may change the associated Recording from `Stopped` to `Completed`, but it must never reopen the Production.

The following invariant is therefore normative:

> **Production closure never ends the deliverability of an already created Recording Artifact.**

The fachlich reason for Production closure is represented conceptually as:

```text
core.ProductionCompletionReason
    = AllRecordingsCompleted
    | ArtifactCompletionTimeout
    | HostForced
```

## Consequences

Production closure, Recording completion and Artifact completion remain separate lifecycle concerns. A production may be closed while an artifact is still pending, and a late artifact may complete the Recording without reopening the Production.

## Not Decided by this ADR

This ADR does not define the scheduler, Force-Close API, concrete permission action, persistence structure for completion reasons, storage retention, or a technical maximum lifetime for browser-local storage.
