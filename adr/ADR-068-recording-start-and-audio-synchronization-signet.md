# ADR-068: Recording Start and Audio Synchronization Signet

## Status

Proposed

## Date

2026-08-21

## Decision Type

Architecture

---

# Deutsch ([English version below](#english-version))

# Kontext

NC-PoRe ermöglicht verteilte Audioaufnahmen, bei denen jeder Teilnehmer seine eigene Audiospur lokal aufzeichnet und diese nach der Aufnahme für die gemeinsame Produktion bereitstellt.

Die einzelnen Aufnahmegeräte und Clients verfügen dabei nicht notwendigerweise über eine gemeinsame Audio-Clock oder einen gemeinsamen Sample-Zeitpunkt. Dadurch können zwischen den einzelnen Aufnahmen zeitliche Unterschiede entstehen.

Für die erste Ausbaustufe wird weder eine automatische DAW-Integration noch eine aktive Laufzeitsynchronisation benötigt.

Stattdessen soll der Host eine gemeinsame Aufnahme explizit starten können, ohne dass Teilnehmer bereits vor diesem bewussten Start aufgezeichnet werden.

Gleichzeitig soll innerhalb der laufenden Aufnahme ein eindeutig erkennbares gemeinsames Audioereignis vorhanden sein, anhand dessen die einzelnen Spuren später in einer beliebigen Audiobearbeitungssoftware manuell ausgerichtet werden können.

---

# Entscheidung

NC-PoRe führt einen **expliziten gemeinsamen Aufnahme-Start** für eine Production Session ein.

Der Aufnahmeablauf besteht aus mehreren logisch getrennten Ereignissen:

1. Der Host löst den Start der Aufnahme aus.
2. Die Clients starten ihre lokale Audioaufnahme.
3. Jeder Client bestätigt automatisch, sobald seine Aufnahme tatsächlich aktiv ist.
4. Der Host erhält den aktuellen Bereitschaftsstatus der für die Aufnahme vorgesehenen Teilnehmer.
5. Sobald alle erforderlichen Teilnehmer ihre Aufnahme bestätigt haben, wird das gemeinsame NC-PoRe Audio-Signet ausgelöst.
6. Das Signet wird von den bereits laufenden lokalen Aufnahmen erfasst und dient als gemeinsamer zeitlicher Referenzpunkt innerhalb der Audiospuren.

Ein Session-Beitritt startet ausdrücklich **keine** Audioaufnahme.

Der Beitritt zu einer Session ist keine Zustimmung zu einer bereits laufenden Aufzeichnung.

---

# Aufnahmeablauf

## 1. Host startet die Aufnahme

Der Host betätigt den Button **„Aufnahme“**.

Dadurch wird der gemeinsame Aufnahmevorgang eingeleitet.

## 2. Clients starten lokal

Das Startsignal wird über die Session an die beteiligten Clients verteilt.

Jeder Client startet daraufhin seine lokale Audioaufnahme.

## 3. Clients bestätigen

Sobald die lokale Aufnahme tatsächlich läuft, meldet der Client automatisch:

`READY`

an die Session.

`READY` bedeutet dabei nicht lediglich, dass der Client den Startbefehl erhalten hat, sondern dass die lokale Aufnahme tatsächlich aktiv ist.

## 4. Host sieht den Bereitschaftsstatus

Der Host erhält während der Vorbereitung der Aufnahme einen sichtbaren Überblick über den Bereitschaftsstatus der für die Aufnahme vorgesehenen Teilnehmer.

Bei kleinen Sessions können die einzelnen Teilnehmer direkt dargestellt werden.

Bei größeren Sessions steht zunächst der aggregierte Status im Vordergrund, beispielsweise:

> **87 / 100 Teilnehmer bereit**

Die einzelnen Teilnehmer können bei Bedarf identifiziert werden, beispielsweise über eine detaillierbare Teilnehmerübersicht.

Damit ist für den Host jederzeit nachvollziehbar, warum das gemeinsame Signet gegebenenfalls noch nicht ausgelöst wurde.

## 5. Alle erforderlichen Teilnehmer sind bereit

Das NC-PoRe-Sync-Signet wird erst ausgelöst, wenn alle für den jeweiligen Aufnahmestart erforderlichen Teilnehmer `READY` gemeldet haben.

Ein Teilnehmer, der nicht `READY` meldet, verhindert zunächst den gemeinsamen Start.

Der Host kann in diesem Zustand warten oder den Aufnahmevorgang abbrechen.

Ob und in welcher Form ein Teilnehmer für einen konkreten Aufnahmestart aus der Gruppe der erforderlichen Teilnehmer entfernt werden kann, ist eine davon getrennte Produktentscheidung.

## 6. NC-PoRe-Signet

Nachdem alle erforderlichen Teilnehmer `READY` gemeldet haben, wird das gemeinsame NC-PoRe-Sync-Signet ausgelöst.

Das Signet wird mit einem kurzen Vorlauf nach dem Eintreffen der erforderlichen Ready-Bestätigungen ausgelöst.

Eine feste maximale Wartezeit von einer Sekunde wird dabei **nicht** als technische Anforderung definiert.

Die bisher diskutierte Größenordnung von bis zu etwa einer Sekunde dient lediglich als Orientierung für die Benutzererfahrung.

Entscheidend ist:

> Das Signet wird erst ausgelöst, wenn die erforderlichen lokalen Aufnahmen tatsächlich laufen.

## 7. Gemeinsamer Referenzpunkt

Das Signet befindet sich anschließend in den laufenden Aufnahmen der Teilnehmer.

Dadurch enthält jede zu dieser Session gehörende Audiospur dasselbe charakteristische Audioereignis an der jeweils aufgenommenen Position.

Dieses Ereignis dient als gemeinsamer zeitlicher Referenzpunkt für die spätere Synchronisation.

---

# Audio-Signet

Das NC-PoRe-Signet ist ein bewusst gestaltetes, kurzes Audioereignis.

Es soll gleichzeitig:

- als akustische Bestätigung des gemeinsamen Aufnahmestarts dienen,
- in den aufgenommenen Audiospuren eindeutig sichtbar sein,
- maschinell zuverlässig erkennbar sein,
- unabhängig von einer bestimmten DAW funktionieren,
- und für die Teilnehmer kurz und möglichst wenig störend sein.

Als Ausgangspunkt für die erste technische Umsetzung dient eine Signatur aus **drei kurzen, breitbandigen und zeitlich gleichmäßig angeordneten Signalereignissen**.

Die konkrete Signalform, Dauer, spektrale Ausgestaltung und Lautheit werden durch dieses ADR noch nicht endgültig festgelegt.

Das Signet bildet zugleich die Grundlage für eine gemeinsame akustische Identität von NC-PoRe.

Ein davon abgeleitetes, freundliches **Ready-Signal** kann dieselbe akustische Signaturfamilie verwenden.

Das Ready-Signal dient dem Benutzerfeedback und muss nicht Bestandteil der aufgenommenen Audiospur sein.

---

# Verwendung in V1

In der ersten Ausbaustufe ist **keine spezielle DAW-Unterstützung** erforderlich.

Nach Abschluss der Aufnahme liegen dem Host beziehungsweise der Produktion die einzelnen Audiospuren vor.

Der Host kann diese in einer beliebigen geeigneten Audiobearbeitungssoftware öffnen und die Spuren anhand des sichtbaren NC-PoRe-Signets manuell ausrichten.

Damit bleibt die Synchronisation unabhängig von:

- Audacity
- Ardour
- Samplitude
- anderen DAWs
- zukünftigen Audiobearbeitungssystemen

Das Signet ist Bestandteil des aufgenommenen Audiomaterials und nicht auf ein bestimmtes DAW-Marker- oder Projektformat angewiesen.

---

# Bewusste Abgrenzung

Diese Entscheidung umfasst ausdrücklich nicht:

- automatische DAW-Session-Erzeugung
- automatische Spurausrichtung
- kontinuierliche Synchronisationskorrektur
- Messung von Capture-Latenzen
- Messung von Netzwerk-Latenzen
- Messung oder Korrektur von Clock Drift
- eine konkrete spätere DAW-Integration

Insbesondere wird in dieser Ausbaustufe nicht versucht, die technischen Latenzeigenschaften der unterschiedlichen Aufnahmewege zu vermessen.

---

# Mögliche spätere Erweiterungen

Die gewählte Architektur soll spätere Erweiterungen ermöglichen, ohne diese bereits als Bestandteil einer bestimmten nächsten Version festzulegen.

Mögliche spätere Erweiterungen sind insbesondere:

- automatische Erkennung des NC-PoRe-Signets
- automatische Ausrichtung der Audiospuren anhand des Signets
- Verwendung mehrerer Signets innerhalb einer Aufnahme
- Analyse der Positionen mehrerer Signets zur Ermittlung zeitlicher Abweichungen
- Untersuchung beziehungsweise Messung von Capture- und Übertragungslatenzen
- Verfahren zur kontinuierlichen Synchronisationsanalyse oder -korrektur
- automatisierte Übergabe synchronisierter Aufnahmen an DAWs oder andere Audiobearbeitungssysteme

Zeitpunkt, Umfang und konkrete technische Ausgestaltung dieser Erweiterungen werden durch diese Architekturentscheidung **nicht festgelegt**.

Insbesondere ist die Möglichkeit einer späteren aktiven Synchronisation ausdrücklich berücksichtigt, ohne deren Umsetzung oder einen bestimmten Zeitpunkt dafür festzulegen.

---

# Konsequenzen

## Vorteile

- Teilnehmer werden erst nach einem expliziten Aufnahmebefehl aufgezeichnet.
- Der Aufnahmebeginn ist unabhängig vom Session-Beitritt.
- Der Host kann erkennen, welche Teilnehmer bereits tatsächlich aufnehmen.
- Der Host kann auch bei größeren Sessions nachvollziehen, ob und auf welche Teilnehmer noch gewartet wird.
- Alle Aufnahmen erhalten einen gemeinsamen, tatsächlich aufgenommenen Referenzpunkt.
- Die erste Ausbaustufe benötigt keine DAW-spezifische Integration.
- Manuelle Synchronisation ist mit vorhandener Audiobearbeitungssoftware möglich.
- Das Signet kann später als Grundlage für automatische Erkennung und weitergehende Synchronisationsverfahren dienen.
- Aufnahme-Lifecycle, Session-Steuerung und Audio-Synchronisationsereignis bleiben logisch getrennt.

## Nachteile

- Die erste Ausbaustufe erfordert weiterhin eine manuelle Ausrichtung der Spuren.
- Das Signet ist Bestandteil der Audiospur und muss bei der späteren Audioproduktion berücksichtigt beziehungsweise entfernt werden.
- Ein gemeinsam aufgenommenes Audioereignis liefert zunächst einen gemeinsamen Referenzpunkt, erklärt aber nicht automatisch die Ursache möglicher Latenz- oder Clock-Differenzen.
- Die konkrete technische Methode, mit der das Signet zu den jeweiligen Aufnahmewegen gelangt, muss im Rahmen der Implementierung festgelegt werden.
- Bei einem nicht bereiten Teilnehmer kann der gemeinsame Aufnahmebeginn zunächst blockiert sein.

---

# Architekturprinzip

Session-Steuerung und Audio-Synchronisationsereignis werden als **unterschiedliche Konzepte** behandelt.

Der Aufnahme-Start ist ein **Session-/Control-Ereignis**.

Die `READY`-Meldung beschreibt den tatsächlichen Zustand der lokalen Aufnahme.

Das NC-PoRe-Signet ist die **zeitliche Referenz des gemeinsamen Startvorgangs im Audiomaterial**.

Damit kann die erste Ausbaustufe bewusst einfach bleiben, während spätere Versionen auf derselben grundlegenden Struktur weiterentwickelt werden können.

Die Architekturentscheidung legt dabei ausdrücklich **keinen späteren Erweiterungszeitpunkt und keine konkrete spätere technische Lösung** fest.

Die Entscheidung steht in Ergänzung zu ADR-063. ADR-063 behandelt die spätere aktive Session-Synchronisation als eigenes Architekturthema. Dieses ADR definiert dagegen den gemeinsamen Aufnahme- und Referenzpunkt für den hier beschriebenen ersten Ausbauschritt und ersetzt die dort getroffene Entscheidung nicht.

---

# English Version ([Deutsche Version oben](#deutsch))

# Context

NC-PoRe supports distributed audio recording in which each participant records their own local audio track and makes that recording available for the joint production after recording.

The individual recording devices and clients do not necessarily share a common audio clock or sample position. As a result, timing differences may exist between individual recordings.

The first implementation requires neither automatic DAW integration nor active runtime synchronization.

Instead, the host must be able to explicitly start a recording without participants being recorded before that deliberate action.

At the same time, a clearly identifiable common audio event should be present within the running recordings so that the individual tracks can later be manually aligned in arbitrary audio editing software.

---

# Decision

NC-PoRe introduces an **explicit shared recording start** for a production session.

The recording flow consists of several logically separate events:

1. The host initiates the recording start.
2. Clients start their local audio recordings.
3. Each client automatically confirms once its recording is actually active.
4. The host receives the current readiness status of the participants intended for the recording.
5. Once all required participants have confirmed their recordings, the common NC-PoRe audio signet is triggered.
6. The signet is captured by the already running local recordings and serves as a common temporal reference point within the audio tracks.

Joining a session explicitly does **not** start audio recording.

Joining a session is not considered consent to an already running recording.

---

# Recording Flow

## 1. Host starts recording

The host activates the **“Record”** button.

This initiates the shared recording process.

## 2. Clients start locally

The start signal is distributed through the session to the participating clients.

Each client then starts its local audio recording.

## 3. Clients confirm

Once the local recording is actually running, the client automatically reports:

`READY`

to the session.

`READY` does not merely mean that the client received the start command. It means that the local recording is actually active.

## 4. Host sees readiness status

During recording preparation, the host receives a visible overview of the readiness status of the participants intended for the recording.

For small sessions, individual participants may be shown directly.

For larger sessions, the aggregated status is shown prominently first, for example:

> **87 / 100 participants ready**

Individual participants can be identified when needed, for example through a detailed participant view.

This makes it clear to the host why the shared signet may not yet have been triggered.

## 5. All required participants are ready

The NC-PoRe sync signet is triggered only after all participants required for the respective recording start have reported `READY`.

A participant that does not report `READY` initially prevents the shared start.

The host may wait or abort the recording start in this state.

Whether and how a participant can be removed from the set of required participants for a particular recording start is a separate product decision.

## 6. NC-PoRe signet

After all required participants have reported `READY`, the common NC-PoRe sync signet is triggered.

The signet is triggered with a short lead-in after the required ready confirmations have been received.

A fixed maximum wait time of one second is **not** defined as a technical requirement.

The previously discussed magnitude of up to approximately one second is only a user-experience guideline.

The important requirement is:

> The signet is triggered only after the required local recordings are actually running.

## 7. Common reference point

The signet is then present in the participants' running recordings.

Each audio track belonging to the session therefore contains the same characteristic audio event at its respective recorded position.

This event serves as the common temporal reference point for later synchronization.

---

# Audio Signet

The NC-PoRe signet is a deliberately designed short audio event.

It should simultaneously:

- provide audible confirmation of the shared recording start,
- be clearly visible in the recorded audio tracks,
- be reliably detectable by software,
- remain independent of any specific DAW,
- and be short and minimally disruptive for participants.

For the initial technical implementation, a signature consisting of **three short, broadband, evenly spaced signal events** is used as the starting point.

The exact signal shape, duration, spectral characteristics, and level are not finally defined by this ADR.

The signet also forms the basis for a shared acoustic identity for NC-PoRe.

A friendly **ready signal** derived from the same acoustic signature family may be used.

The ready signal is user feedback and does not have to be part of the recorded audio.

---

# Use in V1

The first implementation requires **no special DAW support**.

After recording, the host or production receives the individual audio tracks.

The host can open them in any suitable audio editing application and manually align the tracks using the visible NC-PoRe signet.

This keeps synchronization independent of:

- Audacity
- Ardour
- Samplitude
- other DAWs
- future audio editing systems

The signet is part of the recorded audio material and does not depend on a specific DAW marker or project format.

---

# Explicit Scope Boundaries

This decision explicitly does not include:

- automatic DAW session generation
- automatic track alignment
- continuous synchronization correction
- capture latency measurement
- network latency measurement
- clock drift measurement or correction
- a specific future DAW integration

In particular, this implementation does not attempt to measure the technical latency characteristics of the different recording paths.

---

# Possible Future Extensions

The chosen architecture is intended to allow future extensions without defining them as part of a specific next version.

Possible future extensions include:

- automatic detection of the NC-PoRe signet
- automatic audio track alignment based on the signet
- use of multiple signets within a recording
- analysis of multiple signet positions to determine timing deviations
- investigation or measurement of capture and transmission latency
- continuous synchronization analysis or correction
- automated transfer of synchronized recordings to DAWs or other audio editing systems

The timing, scope, and concrete technical implementation of these extensions are **not defined by this architecture decision**.

In particular, future active synchronization is explicitly considered without committing to its implementation or to a particular timeframe.

---

# Consequences

## Benefits

- Participants are recorded only after an explicit recording command.
- Recording does not begin merely by joining a session.
- The host can see which participants are actually recording.
- The host can also determine, for larger sessions, whether and which participants are still pending.
- All recordings receive a common, actually recorded reference point.
- The first implementation requires no DAW-specific integration.
- Manual synchronization is possible with existing audio editing software.
- The signet can later serve as the basis for automatic detection and more advanced synchronization mechanisms.
- Recording lifecycle, session control, and the audio synchronization event remain logically separate.

## Drawbacks

- The first implementation still requires manual track alignment.
- The signet is part of the audio track and must be considered or removed during later audio production.
- A common recorded audio event initially provides a common reference point, but does not automatically identify the cause of latency or clock differences.
- The concrete technical mechanism used to deliver the signet to the respective recording paths must be determined during implementation.
- A participant that is not ready may initially block the shared recording start.

---

# Architectural Principle

Session control and the audio synchronization event are treated as **different concepts**.

The recording start is a **session/control event**.

The `READY` message describes the actual state of the local recording.

The NC-PoRe signet is the **temporal reference of the shared start process in the recorded audio**.

This allows the first implementation to remain deliberately simple while later versions can build on the same fundamental structure.

This architecture decision explicitly defines **neither a future extension timeframe nor a specific future technical solution**.

This decision complements ADR-063. ADR-063 treats active session synchronization as a separate architectural concern for later synchronization mechanisms. This ADR defines the shared recording and reference point for the first implementation described here and does not replace the decision made there.
