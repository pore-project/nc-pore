# ADR-073 Local Recording Safety Cutoff After Connectivity Loss

* Status: Accepted
* Date: 2026-09-03
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

---

# Kontext

NC-PoRE trennt Connectivity und Recording-Lifecycle. Ein vorübergehender oder dauerhafter Verlust der Verbindung zum Core darf eine laufende lokale Aufnahme nicht automatisch beenden, weil dadurch gerade bei verteilten Aufnahmen wertvolles Audiomaterial verloren gehen könnte.

Gleichzeitig darf ein Recorder, der den fachlichen `STOP_RECORDING`-Befehl dauerhaft nicht mehr erreichen kann, nicht unbegrenzt lokal weiter aufnehmen.

Im V1-Design wurde deshalb eine lokale autonome Sicherheitsgrenze definiert.

---

# Entscheidung

Ein Recorder darf eine aktive lokale Aufnahme nach Verlust des Core-Kontakts zunächst fortsetzen.

Für V1 gilt jedoch ein **Safety Cutoff von drei Stunden ab Beginn der technischen Aufnahme**, wenn der fachliche Stop nicht mehr erreicht werden kann.

Der Safety Cutoff:

1. beendet die technische Aufnahme kontrolliert,
2. schließt das bis dahin sicher vorhandene Material als Preservation-Artefakt ab,
3. erzeugt einen persistenten Completion Job,
4. versucht die Weiterverarbeitung bzw. Übertragung, sofern möglich,
5. lässt den Completion Job bei fehlender Verbindung bestehen, damit er später wieder aufgenommen werden kann.

Der Safety Cutoff ist **kein reguläres Session-Ende** und ersetzt keinen fachlichen `STOP_RECORDING`-Befehl des Core.

Der Safety Cutoff erzeugt kein zusätzliches Opening- oder Closing-Signet. Er verwendet, soweit technisch möglich, dieselbe kontrollierte lokale Abschluss- und Preservation-Logik wie ein regulärer technischer Stop.

---

# Connectivity und Recording bleiben getrennt

Ein Connection Loss bedeutet nicht:

```text
Connection lost
    ↓
Recording stopped
```

sondern:

```text
Connection lost
    ↓
Recording continues locally
    ↓
Core Stop arrives
       OR
3 h Safety Cutoff
```

Damit bleibt ein Netzwerkproblem zunächst ein Connectivity-Ereignis und wird nicht fälschlich zu einem Recording-State.

---

# Warum drei Stunden?

Die V1-Grenze ist ein bewusst einfacher, vorhersehbarer Safety Cutoff.

Sie soll lange Interviews und Podcasts ermöglichen und gleichzeitig verhindern, dass ein lokal weiterlaufender Recorder bei dauerhaft fehlender Verbindung unbegrenzt Ressourcen verbraucht.

Die Grenze wird nicht rückwärts aus Dateigröße oder Audioformat berechnet.

Die erwartete oder geplante Dauer einer Aufnahme ist davon unabhängig. Eine erwartete Dauer ist eine Planungs-/Kapazitätsinformation und keine automatische Aufnahmebegrenzung.

---

# Verhalten bei Safety Cutoff

Der Safety Cutoff ist ein **kontrollierter Abschluss**, kein hartes Abschalten.

Bereits sicher gespeichertes Material wird nicht verworfen. Die anschließende Preservation- und Completion-Pipeline bleibt erhalten:

```text
Safety Cutoff
      ↓
Preservation
      ↓
Completion Job
      ↓
Conversion / Transport
      ↓
Server Confirmation
      ↓
Cleanup
```

Fehlt die Verbindung, bleibt der Completion Job persistent und kann bei einem späteren Clientlauf fortgesetzt werden.

---

# Idempotenz und Recovery

Safety Cutoff und seine lokalen Abschlussoperationen müssen idempotent sein.

Ein Absturz während oder nach dem Safety Cutoff darf weder die bereits gesicherten Daten zerstören noch die Aufnahme ein zweites Mal erzeugen.

Der Client darf nach einem Neustart aus dem persistenten lokalen Zustand erkennen, dass ein Safety Cutoff bereits erfolgt ist, und die verbleibende Completion-Pipeline fortsetzen.

---

# Konsequenzen

## Vorteile

* Connection Loss gefährdet nicht unmittelbar eine laufende Aufnahme.
* Ein dauerhaft verwaister Recorder kann nicht unbegrenzt lokal weiterlaufen.
* Bereits aufgezeichnetes Material bleibt erhalten.
* Offline- und Neustart-Szenarien bleiben mit der normalen Completion-/Recovery-Pipeline vereinbar.
* Die Regel ist unabhängig von konkreten Browser- oder Speichertechnologien.

## Kosten

* Der Client muss Connectivity bzw. den letzten bestätigten Core-Kontakt ausreichend zuverlässig verfolgen.
* Der Recorder benötigt einen autonomen Timer bzw. eine äquivalente persistente Safety-Mechanik.
* Der Safety-Cutoff muss gegen normale Stop-/Recovery-Pfade getestet werden.

---

# Beziehung zur bestehenden Architektur

Diese Entscheidung ergänzt insbesondere:

* ADR-029 Distributed Recording Architecture
* ADR-030 Synchronization Strategy for Distributed Recordings
* ADR-039 Recording Architecture and Capture Boundary
* ADR-040 Recorder Workflow and Capture Lifecycle Coordination
* ADR-046 Local Artifact Recovery and Consistency Strategy
* ADR-060 Recording Artifact Processing Lifecycle and Idempotency
* ADR-063 Active Session Synchronisation
* ADR-068 Recording Start and Audio Synchronization Signet
* ADR-071 Recording Capture, Preservation and Transport Formats

Die Entscheidung präzisiert die bereits etablierte Trennung zwischen Connectivity und Recording und definiert die autonome lokale Sicherheitsgrenze für den Fall, dass ein fachlicher Stop den Recorder dauerhaft nicht erreicht.

---

# Status

Accepted. Der Safety Cutoff gehört zur öffentlichen NC-PoRE-Recording-Architektur. Die konkrete technische Umsetzung bleibt von dieser ADR getrennt.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# Context

NC-PoRE separates connectivity from the recording lifecycle. Temporary or permanent loss of connectivity to Core must not automatically stop an active local recording, because doing so could lose valuable audio material in distributed recordings.

At the same time, a recorder that can no longer receive the domain `STOP_RECORDING` command must not continue recording locally without a bound.

The V1 design therefore defined an autonomous local safety boundary.

---

# Decision

A recorder may continue an active local recording after losing contact with Core.

For V1, however, a **three-hour Safety Cutoff measured from the beginning of technical recording** applies when the domain stop cannot be reached.

The Safety Cutoff:

1. stops technical capture in a controlled manner,
2. finalizes all safely available material as a preservation artifact,
3. creates a persistent Completion Job,
4. attempts further processing/transfer when possible,
5. keeps the Completion Job when connectivity is unavailable so it can resume later.

The Safety Cutoff is **not a regular session end** and does not replace a domain `STOP_RECORDING` command from Core.

The Safety Cutoff does not create an additional opening or closing signet. Where technically possible, it uses the same controlled local finalization and preservation path as a regular technical stop.

---

# Connectivity and Recording Remain Separate

A connection loss does not mean:

```text
Connection lost
    ↓
Recording stopped
```

Instead:

```text
Connection lost
    ↓
Recording continues locally
    ↓
Core Stop arrives
       OR
3 h Safety Cutoff
```

A network problem therefore remains a connectivity event and is not incorrectly turned into a recording state.

---

# Why Three Hours?

The V1 limit is a deliberately simple and predictable safety cutoff.

It is intended to support long interviews and podcasts while preventing a locally running recorder from consuming resources indefinitely when connectivity remains unavailable.

The limit is not calculated backwards from file size or audio format.

Expected or planned recording duration is independent of this limit. Expected duration is planning/capacity information, not an automatic recording limit.

---

# Safety Cutoff Behavior

The Safety Cutoff is a **controlled finalization**, not a hard shutdown.

Already safely stored material is not discarded. The preservation and completion pipeline remains intact:

```text
Safety Cutoff
      ↓
Preservation
      ↓
Completion Job
      ↓
Conversion / Transport
      ↓
Server Confirmation
      ↓
Cleanup
```

If connectivity is unavailable, the Completion Job remains persistent and can resume during a later client run.

---

# Idempotency and Recovery

The Safety Cutoff and its local finalization operations must be idempotent.

A crash during or after the Safety Cutoff must neither destroy already preserved data nor create the recording a second time.

After restart, the client must be able to recognize from persistent local state that a Safety Cutoff has already occurred and continue the remaining completion pipeline.

---

# Consequences

## Benefits

* Connection loss does not immediately end an active recording.
* A permanently orphaned recorder cannot continue indefinitely.
* Already captured material is preserved.
* Offline and restart scenarios remain compatible with the normal completion/recovery pipeline.
* The rule is independent of specific browser or storage technologies.

## Costs

* The client must track connectivity or the last sufficiently reliable confirmed Core contact.
* The recorder needs an autonomous timer or equivalent persistent safety mechanism.
* Safety cutoff behavior must be tested against normal stop and recovery paths.

---

# Relationship to Existing Architecture

This decision complements in particular:

* ADR-029 Distributed Recording Architecture
* ADR-030 Synchronization Strategy for Distributed Recordings
* ADR-039 Recording Architecture and Capture Boundary
* ADR-040 Recorder Workflow and Capture Lifecycle Coordination
* ADR-046 Local Artifact Recovery and Consistency Strategy
* ADR-060 Recording Artifact Processing Lifecycle and Idempotency
* ADR-063 Active Session Synchronisation
* ADR-068 Recording Start and Audio Synchronization Signet
* ADR-071 Recording Capture, Preservation and Transport Formats

The decision clarifies the established separation between connectivity and recording and defines the autonomous local safety boundary for the case where a domain stop permanently cannot reach the recorder.

---

# Status

Accepted. The Safety Cutoff is part of the public NC-PoRE recording architecture. Concrete technical implementation remains separate from this ADR.
