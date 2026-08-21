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

Die Session-Mitgliedschaft und die Teilnahme an einem konkreten Recording werden getrennt behandelt.

Der Host bestimmt, welche Session-Mitglieder für einen konkreten Recording-Start zur Aufnahme berechtigt sind. Aus dieser Auswahl entsteht die **Recording-Teilnehmermenge** für den jeweiligen Recording-Start. Diese Menge wird zum Startzeitpunkt festgelegt und bleibt für diesen Recording-Start unverändert.

Der Aufnahmeablauf besteht aus mehreren logisch getrennten Ereignissen:

1. Der Host löst den Start der Aufnahme aus.
2. Die Recording-berechtigten Clients starten ihre lokale Audioaufnahme.
3. Jeder Client bestätigt eindeutig zu diesem Recording-Start und zu seiner Recording-Teilnahme, sobald seine lokale Aufnahme tatsächlich aktiv ist.
4. Der Host erhält den aktuellen Bereitschaftsstatus der Recording-Teilnehmer.
5. Sobald alle erforderlichen Recording-Teilnehmer `READY` gemeldet haben, wird das gemeinsame NC-PoRe Opening Sync Signet ausgelöst.
6. Das Signet wird von den bereits laufenden lokalen Aufnahmen erfasst und markiert den logischen Beginn des Recordings.
7. Der Host kann die Aufnahme später explizit stoppen. Dabei wird zunächst das NC-PoRe Closing Sync Signet ausgelöst.
8. Das Closing Sync Signet markiert das logische Ende des Recordings. Erst danach werden die lokalen Recorder technisch beendet.
9. Jeder Client bestätigt nach dem tatsächlichen technischen Ende seiner lokalen Aufnahme mit `OK`.

Ein Session-Beitritt startet ausdrücklich **keine** Audioaufnahme.

Der Beitritt zu einer Session ist keine Zustimmung zu einer bereits laufenden Aufzeichnung.

---

# Aufnahmeberechtigung und Recording-Teilnehmer

Nicht jedes Session-Mitglied muss an jedem Recording teilnehmen.

Der Host bestimmt für einen konkreten Recording-Start die Menge der Recording-berechtigten Teilnehmer. Diese Menge wird für den jeweiligen Startvorgang eingefroren.

Damit gilt:

> **Session-Mitgliedschaft ist nicht gleich Recording-Teilnahme.**

Nur die für das konkrete Recording berechtigten Teilnehmer werden in den `READY`- und `OK`-Status des Recording-Vorgangs einbezogen.

Ein Session-Mitglied, das nicht für das konkrete Recording berechtigt ist, sieht keinerlei Recording-Status. Es sieht weder seinen eigenen Status noch den Status anderer Teilnehmer.

---

# Aufnahmeablauf

## 1. Host startet die Aufnahme

Der Host betätigt den Button **„Aufnahme“**.

Dadurch wird der gemeinsame Aufnahmevorgang eingeleitet.

## 2. Clients starten lokal

Das Startsignal wird über die Session an die für das Recording berechtigten Clients verteilt.

Jeder dieser Clients startet daraufhin seine lokale Audioaufnahme.

## 3. Clients bestätigen

Sobald die lokale Aufnahme tatsächlich läuft, meldet der Client automatisch:

`READY`

an die Session.

Die Meldung muss eindeutig einem konkreten Recording-Start und einem konkreten Recording-Teilnehmer zugeordnet werden können.

Ein `READY`-Status aus einem früheren Recording-Start darf daher niemals für einen späteren Recording-Start verwendet werden.

`READY` bedeutet dabei nicht lediglich, dass der Client den Startbefehl erhalten hat, sondern:

> **Der lokale Recorder zeichnet für diesen konkreten Recording-Start tatsächlich auf.**

## 4. Statusanzeige für berechtigte Teilnehmer

Der Host erhält während der Vorbereitung der Aufnahme einen sichtbaren Überblick über den Bereitschaftsstatus der für das Recording berechtigten Teilnehmer.

Bei kleinen Sessions können die einzelnen Teilnehmer direkt dargestellt werden.

Bei größeren Sessions steht zunächst der aggregierte Status im Vordergrund, beispielsweise:

> **87 / 100 Teilnehmer bereit**

Die einzelnen Teilnehmer können bei Bedarf identifiziert werden, beispielsweise über eine detaillierbare Teilnehmerübersicht.

Damit ist für den Host jederzeit nachvollziehbar, warum das gemeinsame Signet gegebenenfalls noch nicht ausgelöst wurde.

Auch jeder Recording-berechtigte Teilnehmer sieht den eigenen aktuellen Recording-Status.

Dabei gilt:

- 🔴 **nicht bereit** – die lokale Aufnahme ist für den konkreten Recording-Start nicht aktiv
- 🟢 **bereit** – die lokale Aufnahme ist aktiv und der Client hat `READY` gemeldet

Die Statusanzeige beschreibt ausdrücklich den Recording-Zustand und nicht lediglich den Verbindungszustand des Clients.

Nicht Recording-berechtigte Session-Mitglieder sehen keinerlei Recording-Status.

## 5. Alle erforderlichen Teilnehmer sind bereit

Das NC-PoRe Opening Sync Signet wird erst ausgelöst, wenn alle für den jeweiligen Aufnahmestart erforderlichen Recording-Teilnehmer `READY` gemeldet haben.

Ein Recording-Teilnehmer, der nicht `READY` meldet, verhindert zunächst den gemeinsamen Start.

Der Host kann in diesem Zustand warten oder den Aufnahmevorgang abbrechen.

Die Recording-Teilnehmermenge wird während dieses Startvorgangs nicht automatisch erweitert, wenn weitere Session-Mitglieder beitreten.

## 6. Opening Sync Signet

Nachdem alle erforderlichen Recording-Teilnehmer `READY` gemeldet haben, wird das gemeinsame NC-PoRe Opening Sync Signet ausgelöst.

Das Signet wird mit einem kurzen Vorlauf nach dem Eintreffen der erforderlichen Ready-Bestätigungen ausgelöst.

Eine feste maximale Wartezeit von einer Sekunde wird dabei **nicht** als technische Anforderung definiert.

Die bisher diskutierte Größenordnung von bis zu etwa einer Sekunde dient lediglich als Orientierung für die Benutzererfahrung.

Entscheidend ist:

> **Das Opening Sync Signet wird erst ausgelöst, wenn die erforderlichen lokalen Aufnahmen tatsächlich laufen.**

Das Opening Sync Signet markiert den **logischen Beginn des Recordings**.

## 7. Gemeinsamer Referenzpunkt

Das Opening Sync Signet wird von den laufenden Aufnahmen der Recording-Teilnehmer erfasst.

Dadurch enthält jede zu diesem Recording gehörende Audiospur dasselbe charakteristische Audioereignis an der jeweils aufgenommenen Position.

Dieses Ereignis dient als gemeinsamer zeitlicher Referenzpunkt für die spätere Synchronisation.

---

# Stoppen der Aufnahme

Der Host kann die Aufnahme über einen separaten **„Aufnahme stoppen“**-Befehl beenden.

Der Stop-Befehl bedeutet zunächst nicht, dass die lokalen Recorder unmittelbar technisch beendet werden.

Stattdessen wird zunächst das gemeinsame Closing Sync Signet ausgelöst.

Der Ablauf ist:

1. Der Host löst den Stop-Vorgang aus.
2. Die betroffenen lokalen Recorder bleiben noch aktiv.
3. Das Closing Sync Signet wird ausgelöst.
4. Das Closing Sync Signet wird von den noch laufenden Aufnahmen erfasst.
5. Das Closing Sync Signet markiert das **logische Ende des Recordings**.
6. Danach wird der digitale Stop-Befehl an die betroffenen Clients verteilt.
7. Jeder Client beendet seine lokale Aufnahme, sobald er den Stop-Befehl verarbeitet hat.
8. Der Client meldet nach dem tatsächlichen technischen Ende seiner lokalen Aufnahme `OK`.
9. Der Stop-Befehl wird für noch nicht bestätigte Clients wiederholt, bis alle betroffenen Clients die erfolgreiche Beendigung ihrer lokalen Aufnahme bestätigt haben.

Das Wiederholungsintervall für den Stop-Befehl ist eine Implementierungsentscheidung und wird durch dieses ADR nicht auf einen bestimmten Wert festgelegt.

Das technische Ende einer lokalen Aufnahme ist **nicht** der zeitliche Referenzpunkt des Recordings.

Ein Client darf daher nach dem Closing Sync Signet noch technisch aufzeichnen. Dieses Audio gehört nicht mehr zum logischen Recording und kann bei der späteren Verarbeitung als technischer Nachlauf behandelt beziehungsweise entfernt werden.

Damit gilt:

> **Opening Sync Signet = logischer Beginn des Recordings**

> **Closing Sync Signet = logisches Ende des Recordings**

Das technische Ende der einzelnen lokalen Recorder kann zeitlich danach liegen und muss nicht bei allen Clients exakt gleichzeitig erfolgen.

---

# Verhalten bei Ausfall während eines Recordings

Wenn ein Client nach dem Opening Sync Signet die Aufnahme oder die Session-Verbindung verliert, ist dies zunächst ein Ausfall dieses Recording-Teilnehmers.

Das laufende Recording der übrigen Teilnehmer wird dadurch nicht automatisch ungültig.

Eine Wiederherstellung der Session-Verbindung stellt nicht automatisch die Kontinuität der Audioaufnahme wieder her.

Ein späterer Re-Join und eine mögliche Wiederaufnahme der lokalen Aufnahme sind getrennte Vorgänge und werden durch dieses ADR nicht vollständig spezifiziert.

Insbesondere kann ein Audioausfall zwischen zwei Zeitpunkten nicht dadurch als lückenlose Aufnahme behandelt werden, dass der Client lediglich wieder der Session beitritt.

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

Das Opening und das Closing Sync Signet verwenden dieselbe grundlegende Signaturfamilie. Das Closing Signet kann sich beispielsweise durch eine umgekehrte Ton- oder Signalfolge eindeutig vom Opening Signet unterscheiden.

Die konkrete Signalform, Dauer, spektrale Ausgestaltung und Lautheit werden durch dieses ADR noch nicht endgültig festgelegt.

Das Signet bildet zugleich die Grundlage für eine gemeinsame akustische Identität von NC-PoRe.

Ein davon abgeleitetes, freundliches Ready-Signal kann dieselbe akustische Signaturfamilie verwenden. Dieses Ready-Signal dient dem Benutzerfeedback und muss nicht Bestandteil der aufgenommenen Audiospur sein.

---

# Zwei Synchronisationsanker

Das Recording besitzt zwei gemeinsame Audio-Referenzpunkte:

- das **Opening Sync Signet** am logischen Beginn des Recordings
- das **Closing Sync Signet** am logischen Ende des Recordings

Damit können die einzelnen Audiospuren später nicht nur anhand eines gemeinsamen Startpunkts ausgerichtet werden.

Die Position beider Signets kann perspektivisch auch verwendet werden, um Unterschiede zwischen den Spuren über die Dauer des Recordings zu analysieren.

Insbesondere kann der Vergleich der Abstände zwischen Opening und Closing Sync Signet später die Untersuchung zeitlicher Abweichungen beziehungsweise Drift zwischen einzelnen Aufnahmen ermöglichen.

Eine solche Analyse oder Korrektur ist nicht Bestandteil dieser ersten Ausbaustufe.

---

# Verwendung in der ersten Ausbaustufe

In der ersten Ausbaustufe ist **keine spezielle DAW-Unterstützung** erforderlich.

Nach Abschluss der Aufnahme liegen dem Host beziehungsweise der Produktion die einzelnen Audiospuren vor.

Der Host kann diese in einer beliebigen geeigneten Audiobearbeitungssoftware öffnen und die Spuren anhand des sichtbaren Opening Sync Signets manuell ausrichten.

Das Closing Sync Signet kann zusätzlich als zweiter Referenzpunkt verwendet werden.

Damit bleibt die Synchronisation unabhängig von:

- Audacity
- Ardour
- Samplitude
- anderen DAWs
- zukünftigen Audiobearbeitungssystemen

Die Signets sind Bestandteil des aufgenommenen Audiomaterials und nicht auf ein bestimmtes DAW-Marker- oder Projektformat angewiesen.

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
- eine vollständige Recovery- oder Re-Join-Strategie für ausgefallene Recording-Teilnehmer

Insbesondere wird in dieser Ausbaustufe nicht versucht, die technischen Latenzeigenschaften der unterschiedlichen Aufnahmewege zu vermessen.

---

# Mögliche spätere Erweiterungen

Die gewählte Architektur soll spätere Erweiterungen ermöglichen, ohne diese bereits als Bestandteil einer bestimmten nächsten Version festzulegen.

Mögliche spätere Erweiterungen sind insbesondere:

- automatische Erkennung des NC-PoRe-Signets
- automatische Ausrichtung der Audiospuren anhand der Signets
- Analyse der Positionen von Opening und Closing Sync Signet zur Ermittlung zeitlicher Abweichungen
- Verwendung weiterer Signets innerhalb einer Aufnahme
- Untersuchung beziehungsweise Messung von Capture- und Übertragungslatenzen
- Verfahren zur kontinuierlichen Synchronisationsanalyse oder -korrektur
- Recovery- und Re-Join-Verfahren für unterbrochene Aufnahmen
- automatisierte Übergabe synchronisierter Aufnahmen an DAWs oder andere Audiobearbeitungssysteme

Zeitpunkt, Umfang und konkrete technische Ausgestaltung dieser Erweiterungen werden durch diese Architekturentscheidung **nicht festgelegt**.

Insbesondere ist die Möglichkeit einer späteren aktiven Synchronisation ausdrücklich berücksichtigt, ohne deren Umsetzung oder einen bestimmten Zeitpunkt dafür festzulegen.

---

# Konsequenzen

## Vorteile

- Teilnehmer werden erst nach einem expliziten Aufnahmebefehl aufgezeichnet.
- Der Aufnahmebeginn ist unabhängig vom Session-Beitritt.
- Session-Mitgliedschaft und Recording-Teilnahme bleiben sauber getrennt.
- Die Recording-Teilnehmermenge ist für einen konkreten Start eindeutig und stabil.
- Jede `READY`-Meldung ist eindeutig einem Recording-Start und einem Recording-Teilnehmer zugeordnet.
- Der Host kann erkennen, welche Teilnehmer bereits tatsächlich aufnehmen.
- Recording-berechtigte Teilnehmer können ihren eigenen tatsächlichen Recording-Status erkennen.
- Nicht Recording-berechtigte Session-Mitglieder erhalten keine Recording-Statusinformationen.
- Der Host kann auch bei größeren Sessions nachvollziehen, ob und auf welche Teilnehmer noch gewartet wird.
- Alle Aufnahmen erhalten einen gemeinsamen Opening-Referenzpunkt.
- Das Recording erhält zusätzlich einen gemeinsamen Closing-Referenzpunkt.
- Das logische Ende des Recordings ist unabhängig vom technisch unterschiedlichen Stop-Zeitpunkt der lokalen Recorder.
- Die erste Ausbaustufe benötigt keine DAW-spezifische Integration.
- Manuelle Synchronisation ist mit vorhandener Audiobearbeitungssoftware möglich.
- Die beiden Sync-Anker können später als Grundlage für automatische Erkennung, Driftanalyse und weitergehende Synchronisationsverfahren dienen.
- Aufnahme-Lifecycle, Session-Steuerung und Audio-Synchronisationsereignisse bleiben logisch getrennt.

## Nachteile

- Die erste Ausbaustufe erfordert weiterhin eine manuelle Ausrichtung der Spuren.
- Die Signets sind Bestandteil der Audiospur und müssen bei der späteren Audioproduktion berücksichtigt beziehungsweise entfernt werden.
- Ein gemeinsam aufgenommenes Audioereignis liefert zunächst einen Referenzpunkt, erklärt aber nicht automatisch die Ursache möglicher Latenz- oder Clock-Differenzen.
- Die konkrete technische Methode, mit der die Signets zu den jeweiligen Aufnahmewegen gelangen, muss im Rahmen der Implementierung festgelegt werden.
- Bei einem nicht bereiten Teilnehmer kann der gemeinsame Aufnahmebeginn zunächst blockiert sein.
- Ein Ausfall eines Recording-Teilnehmers nach dem Opening Signet kann zu einer Lücke oder einem anderen Ausfall in dessen Audiospur führen.

---

# Architekturprinzip

Session-Steuerung und Audio-Synchronisationsereignisse werden als **unterschiedliche Konzepte** behandelt.

Die Session-Mitgliedschaft bestimmt, wer an der Session teilnehmen kann.

Die Recording-Berechtigung bestimmt, wer an einem konkreten Recording teilnehmen und dessen Recording-Status sehen darf.

Der konkrete Recording-Start besitzt eine eigene Identität und eine zum Startzeitpunkt festgelegte Recording-Teilnehmermenge.

Der Aufnahme-Start ist ein **Session-/Control-Ereignis**.

Die `READY`-Meldung beschreibt den tatsächlichen Zustand der lokalen Aufnahme und ist eindeutig einem konkreten Recording-Start und Recording-Teilnehmer zugeordnet.

Das Opening Sync Signet ist die **zeitliche Referenz des logischen Beginns des Recordings**.

Das Closing Sync Signet ist die **zeitliche Referenz des logischen Endes des Recordings**.

Die technische Beendigung der lokalen Recorder ist davon getrennt und wird durch `OK` bestätigt.

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

Session membership and participation in a specific recording are treated separately.

For a specific recording start, the host determines which session members are authorized to participate in the recording. This selection forms the **recording participant set** for that recording start. The set is fixed when the recording start is initiated and remains unchanged for that recording start.

The recording flow consists of several logically separate events:

1. The host initiates the recording start.
2. Recording-authorized clients start their local audio recordings.
3. Each client confirms, explicitly associated with that recording start and its recording participation, once its local recording is actually active.
4. The host receives the current readiness status of the recording participants.
5. Once all required recording participants have reported `READY`, the common NC-PoRe Opening Sync Signet is triggered.
6. The signet is captured by the already running local recordings and marks the logical beginning of the recording.
7. The host can later explicitly stop the recording. The NC-PoRe Closing Sync Signet is triggered first.
8. The Closing Sync Signet marks the logical end of the recording. Only afterwards are the local recorders technically stopped.
9. Each client confirms `OK` after its local recording has actually stopped.

Joining a session explicitly does **not** start audio recording.

Joining a session is not considered consent to an already running recording.

---

# Recording Authorization and Recording Participants

Not every session member has to participate in every recording.

For a specific recording start, the host determines the set of recording-authorized participants. This set is fixed for that recording start.

Therefore:

> **Session membership is not the same as recording participation.**

Only participants authorized for the specific recording are included when evaluating `READY` and `OK` for that recording process.

A session member who is not authorized for the specific recording sees no recording status at all. They see neither their own status nor the status of other participants.

---

# Recording Flow

## 1. Host starts recording

The host activates the **“Record”** button.

This initiates the shared recording process.

## 2. Clients start locally

The start signal is distributed through the session to the clients authorized for the recording.

Each of these clients then starts its local audio recording.

## 3. Clients confirm

Once the local recording is actually running, the client automatically reports:

`READY`

to the session.

The message must be uniquely associated with a specific recording start and a specific recording participant.

A `READY` status from an earlier recording start must therefore never be usable for a later recording start.

`READY` does not merely mean that the client received the start command. It means:

> **The local recorder is actually recording for this specific recording start.**

## 4. Status visibility for authorized participants

During recording preparation, the host receives a visible overview of the readiness status of the participants authorized for the recording.

For small sessions, individual participants may be shown directly.

For larger sessions, the aggregated status is shown prominently first, for example:

> **87 / 100 participants ready**

Individual participants can be identified when needed, for example through a detailed participant view.

This makes it clear to the host why the shared signet may not yet have been triggered.

Each recording-authorized participant also sees their own current recording status.

The status is:

- 🔴 **not ready** – the local recording is not active for the specific recording start
- 🟢 **ready** – the local recording is active and the client has reported `READY`

The status explicitly represents the recording state and not merely the client's connection state.

Session members who are not authorized for the recording see no recording status.

## 5. All required participants are ready

The NC-PoRe Opening Sync Signet is triggered only after all recording participants required for the respective recording start have reported `READY`.

A recording participant that does not report `READY` initially prevents the shared start.

The host may wait or abort the recording start in this state.

The recording participant set is not automatically expanded during this start process if additional session members join.

## 6. Opening Sync Signet

After all required recording participants have reported `READY`, the common NC-PoRe Opening Sync Signet is triggered.

The signet is triggered with a short lead-in after the required ready confirmations have been received.

A fixed maximum wait time of one second is **not** defined as a technical requirement.

The previously discussed magnitude of up to approximately one second is only a user-experience guideline.

The important requirement is:

> **The Opening Sync Signet is triggered only after the required local recordings are actually running.**

The Opening Sync Signet marks the **logical beginning of the recording**.

## 7. Common reference point

The Opening Sync Signet is captured by the running recordings of the recording participants.

Each audio track belonging to this recording therefore contains the same characteristic audio event at its respective recorded position.

This event serves as the common temporal reference point for later synchronization.

---

# Stopping the Recording

The host can stop the recording using a separate **“Stop recording”** command.

The stop command does not initially mean that the local recorders must be technically stopped immediately.

Instead, the common Closing Sync Signet is triggered first.

The sequence is:

1. The host initiates the stop process.
2. The affected local recorders remain active.
3. The Closing Sync Signet is triggered.
4. The Closing Sync Signet is captured by the still-running recordings.
5. The Closing Sync Signet marks the **logical end of the recording**.
6. The digital stop command is then distributed to the affected clients.
7. Each client stops its local recording once it has processed the stop command.
8. The client reports `OK` after its local recording has actually stopped.
9. The stop command is repeated for clients that have not yet confirmed until all affected clients have confirmed successful termination of their local recordings.

The retry interval for the stop command is an implementation decision and is not fixed to a particular value by this ADR.

The technical end of a local recording is **not** the temporal reference point of the recording.

A client may therefore continue recording technically for a short time after the Closing Sync Signet. This audio is no longer part of the logical recording and may be treated or removed as technical tail data during later processing.

Therefore:

> **Opening Sync Signet = logical beginning of the recording**

> **Closing Sync Signet = logical end of the recording**

The technical end of the individual local recorders may occur afterwards and does not have to happen at exactly the same time on all clients.

---

# Behavior on Failure During Recording

If a client loses its recording or session connection after the Opening Sync Signet, this is initially considered a failure of that recording participant.

The ongoing recording of the other participants is not automatically invalidated by this event.

Restoring the session connection does not automatically restore continuity of the audio recording.

A later re-join and a possible resumption of local recording are separate processes and are not fully specified by this ADR.

In particular, an audio gap between two points in time cannot be treated as a continuous recording merely because the client has rejoined the session.

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

The Opening and Closing Sync Signets use the same basic signet family. The Closing Signet may, for example, use a reversed tone or signal sequence to distinguish it clearly from the Opening Signet.

The exact signal shape, duration, spectral characteristics, and level are not finally defined by this ADR.

The signet also forms the basis for a common NC-PoRe acoustic identity.

A friendly Ready signal derived from the same acoustic signet family may be used for user feedback. This Ready signal does not have to be part of the recorded audio tracks.

---

# Two Synchronization Anchors

The recording has two common audio reference points:

- the **Opening Sync Signet** at the logical beginning of the recording
- the **Closing Sync Signet** at the logical end of the recording

This allows the individual audio tracks to be aligned later not only using a common start point.

The positions of both signets can potentially also be used to analyze differences between tracks over the duration of the recording.

In particular, comparing the distance between the Opening and Closing Sync Signets may later allow the analysis of timing differences or drift between individual recordings.

Such analysis or correction is not part of the first implementation.

---

# Use in the First Implementation

The first implementation requires **no specific DAW support**.

After recording, the host or production process has the individual audio tracks available.

The host can open them in any suitable audio editing software and manually align the tracks using the visible Opening Sync Signet.

The Closing Sync Signet can additionally be used as a second reference point.

This keeps synchronization independent of:

- Audacity
- Ardour
- Samplitude
- other DAWs
- future audio editing systems

The signets are part of the recorded audio material and do not depend on a specific DAW marker or project format.

---

# Explicit Non-Goals

This decision explicitly does not include:

- automatic DAW session generation
- automatic track alignment
- continuous synchronization correction
- measurement of capture latency
- measurement of network latency
- measurement or correction of clock drift
- a specific future DAW integration
- a complete recovery or re-join strategy for failed recording participants

In particular, the first implementation does not attempt to measure the technical latency characteristics of the different recording paths.

---

# Possible Future Extensions

The chosen architecture is intended to allow future extensions without defining them as part of a specific next version.

Possible future extensions include, in particular:

- automatic detection of the NC-PoRe signet
- automatic track alignment based on the signets
- analysis of Opening and Closing Sync Signet positions to determine timing differences
- additional signets within a recording
- investigation or measurement of capture and transmission latency
- continuous synchronization analysis or correction mechanisms
- recovery and re-join mechanisms for interrupted recordings
- automated transfer of synchronized recordings to DAWs or other audio editing systems

The timing, scope, and concrete technical design of such extensions are **not defined by this architectural decision**.

In particular, the possibility of later active synchronization is explicitly considered without defining its implementation or a specific time for introducing it.

---

# Consequences

## Advantages

- Participants are recorded only after an explicit recording command.
- Recording does not begin merely because someone joins the session.
- Session membership and recording participation remain clearly separated.
- The recording participant set is unambiguous and stable for a specific recording start.
- Every `READY` message is uniquely associated with a recording start and recording participant.
- The host can see which participants are actually recording.
- Recording-authorized participants can see their own actual recording state.
- Session members who are not authorized for the recording receive no recording status information.
- The host can also determine, for larger sessions, whether and which participants are still awaited.
- All recordings receive a common Opening reference point.
- The recording also receives a common Closing reference point.
- The logical end of the recording is independent of technically different local recorder stop times.
- The first implementation requires no DAW-specific integration.
- Manual synchronization is possible with existing audio editing software.
- The two sync anchors can later serve as a basis for automatic detection, drift analysis, and more advanced synchronization mechanisms.
- Recording lifecycle, session control, and audio synchronization events remain logically separated.

## Disadvantages

- The first implementation still requires manual track alignment.
- The signets are part of the audio tracks and must be considered or removed during later production.
- A common audio event initially provides a reference point but does not automatically explain the cause of possible latency or clock differences.
- The concrete technical method by which the signets reach the respective recording paths must be defined during implementation.
- A participant that is not ready can initially block the shared recording start.
- A recording participant failure after the Opening Signet can result in a gap or other interruption in that participant's audio track.

---

# Architectural Principle

Session control and audio synchronization events are treated as **different concepts**.

Session membership determines who can participate in the session.

Recording authorization determines who may participate in a specific recording and who may see its recording status.

A specific recording start has its own identity and a recording participant set fixed when the start is initiated.

The recording start is a **session/control event**.

The `READY` message describes the actual state of the local recording and is uniquely associated with a specific recording start and recording participant.

The Opening Sync Signet is the **temporal reference for the logical beginning of the recording**.

The Closing Sync Signet is the **temporal reference for the logical end of the recording**.

Technical termination of the local recorders is separate from the logical end and is confirmed by `OK`.

This allows the first implementation to remain deliberately simple while later versions can build on the same fundamental structure.

This architectural decision explicitly defines **no future extension date and no specific future technical solution**.

This decision complements ADR-063. ADR-063 treats later active session synchronization as a separate architectural topic. This ADR instead defines the shared recording and reference points for the first implementation described here and does not replace the decision made there.
