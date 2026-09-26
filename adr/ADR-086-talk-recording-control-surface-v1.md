# ADR-086 Talk Recording Control Surface V1

- Status: Accepted
- Date: 2026-09-26
- Decision Type: Product / UX

---

# Deutsch ([English version below](#english-version))

---

# Kontext

NC-PoRE stellt die Recording-Funktion in V1 innerhalb der Nextcloud-Talk-Erfahrung bereit. Die fachliche Recording-Wahrheit liegt dabei im Core/Application-Layer; die Talk-Integration liefert Host- und Teilnehmerkontext und stellt die PoRE-Funktion dar.

Der aktuelle V1-Stand besitzt bereits eine funktionale Talk-nahe Recording-Oberfläche mit:

- explizitem **Aufnahme starten** für den Host;
- explizitem **Aufnahme beenden** für den Host;
- Anzeige des fachlichen Recording-Zustands;
- READY- und Opening-Fortschritt für den Host;
- Anzeige der verstrichenen Aufnahmezeit;
- eigenständiger Anzeige für Recording-Teilnehmer;
- einer Host-Aktion zum endgültigen Schließen einer bereits beendeten Production.

Die Oberfläche ist damit keine reine technische Debug-Oberfläche mehr. Sie ist eine nutzbare PoRE-Produktoberfläche, die vor einer Beta-/App-Store-Freigabe noch visuell und funktional weiter gehärtet werden muss.

# Entscheidung

V1 stellt die Recording-Funktion als **PoRE-Schicht innerhalb der Host-Erfahrung** dar. Für Nextcloud Talk bedeutet das eine kompakte, Talk-nahe Oberfläche statt eines unabhängigen Recording-Fensters.

Die Oberfläche bleibt bewusst dünn:

- Core/Application liefern den autoritativen Recording-Zustand;
- die Oberfläche rendert diesen Zustand;
- Benutzeraktionen werden an die vorgesehenen fachlichen Kommandogrenzen weitergereicht;
- Talk-spezifische Media- und Session-Details bleiben im Connector;
- technische Diagnoseinformationen werden nicht zur primären Benutzeroberfläche.

Die Oberfläche darf keine eigene fachliche Recording-State-Machine erzeugen.

# Rollen und Sichtbarkeit

## Host

Der Host darf die für die konkrete Aufnahme relevanten Informationen sehen:

- aktuellen Recording-Zustand;
- erwartete Recording-Teilnehmer;
- deren READY-Zustand während der Vorbereitung;
- aggregierte Bereitschaft;
- Opening-Bestätigungen während des Übergangs;
- verstrichene Aufnahmezeit während der aktiven Aufnahme;
- **Aufnahme starten**;
- **Aufnahme beenden**;
- nach dem Stop den weiteren Abschluss-/Transferzustand;
- nach einem gestoppten Recording gegebenenfalls **Production endgültig schließen**.

Die Teilnehmermenge bezieht sich auf das konkrete Recording und nicht auf die gesamte Talk-Session.

## Recording-Teilnehmer

Ein Recording-Teilnehmer sieht mindestens:

- den eigenen Recording-Zustand;
- den eigenen READY-Zustand;
- eine eindeutige Anzeige, wenn die lokale Aufnahme aktiv ist;
- den weiteren für die eigene Aufnahme relevanten Abschlusszustand.

Technische Identitäten, Transport-Handles und interne Connector-Details gehören nicht in die normale Oberfläche.

## Nicht beteiligte Session-Mitglieder

Ein Session-Mitglied, das nicht am konkreten Recording beteiligt ist, erhält keine Recording-Statusinformationen.

Insbesondere dürfen ihm nicht angezeigt werden:

- ob andere Personen aufnehmen;
- deren READY-Zustand;
- Stop-/Transferfortschritt;
- Artefaktstatus.

# Zustandsdarstellung

Die Oberfläche unterscheidet mindestens die fachlich relevanten Zustände:

- Vorbereitung;
- READY;
- Opening;
- Recording;
- Stopping bzw. Übertragung;
- Aufnahme beendet;
- Aufnahme bestätigt;
- Production geschlossen;
- Fehler.

Die visuelle Darstellung darf diese Bedeutungen nicht durch lokale oder zufällige UI-Zustände verändern.

Farbe allein darf niemals die Bedeutung tragen. Text, Symbol oder zugängliche semantische Kennzeichnung müssen den Zustand zusätzlich verständlich machen.

Ein technischer Übergang wie das Warten auf Opening-Bestätigung ist ein Darstellungsdetail und kein neuer fachlicher Recording-State.

# Recording-Steuerung

Die primären Benutzeraktionen sind explizit:

- **Aufnahme starten**
- **Aufnahme beenden**

Für den Host gilt beim Start:

```text
Vorbereitung
    ->
lokales Capture
    ->
READY
    ->
Opening
    ->
Recording
```

Beim Stop:

```text
Recording
    ->
Stopping
    ->
lokaler technischer Abschluss
    ->
Artifact-/Transport-Abschluss
    ->
Recording Completed
```

Eine erfolgreiche technische Teilaktion darf nicht als fachliche Gesamtbestätigung dargestellt werden.

Insbesondere gilt:

- lokaler Recorder-Stop ist nicht gleich serverseitige Artefaktbestätigung;
- erfolgreicher Transport eines einzelnen Teilnehmer-Artefakts ist nicht gleich `core.Recording.Completed`;
- `core.Recording.Completed` ist nicht gleich `core.Production.Completed`.

Die Production kann nach den Regeln von ADR-085 ausdrücklich auch vor vollständiger Artefaktlieferung geschlossen werden.

# Recording-Transparenz

Ein Benutzer muss jederzeit erkennen können:

- ob eine Aufnahme tatsächlich aktiv ist;
- ob er selbst daran beteiligt ist;
- ob seine lokale Aufnahme technisch bereit ist;
- ob die Aufnahme bereits beendet wurde;
- ob die Verarbeitung bzw. Übertragung noch läuft;
- ob die serverseitige Bestätigung bereits erreicht wurde.

Die UI darf technische Begriffe wie `MediaRecorder`, Track-ID, EventSource oder interne Transportzustände nicht an die Stelle dieser fachlichen Aussage setzen.

# Talk-Integration

Die Oberfläche wird als PoRE-Funktion in die Talk-Erfahrung integriert.

Dabei gilt:

- Talk bleibt Eigentümer seiner Gesprächs- und Medienoberfläche;
- PoRE bleibt Eigentümer seiner Recording-Funktion;
- die Integration verwendet einen expliziten Mount-/Kontextpfad;
- fachliche Entscheidungen werden nicht aus Talk-DOM-Zuständen abgeleitet;
- zufällige oder versionsabhängige DOM-Selektoren sind keine fachliche API.

Die konkrete visuelle Einbettung darf sich während der Alpha-Härtung noch verbessern, ohne die fachlichen Zuständigkeiten zu verändern.

# Zugänglichkeit

Recording-Status und Recording-Aktionen müssen auch ohne Farbwahrnehmung verständlich bleiben.

Dazu gehören insbesondere:

- zugängliche Namen für die PoRE-Steuerung;
- semantische Statusinformationen;
- klare Beschriftung der Host-Aktionen;
- eindeutige Kennzeichnung eines aktiven Recordings;
- keine rein visuelle oder blinkende Darstellung als alleinige Statusinformation.

# Bewusste Nichtziele für V1

Diese ADR legt nicht fest:

- eine vollständige Produktions- oder Studio-Dashboard-Oberfläche;
- eine eigene Browser-Recording-Anwendung außerhalb der Host-Erfahrung;
- eine vollständige technische Diagnosekonsole;
- pixelgenaue Nachbildung der Talk-Oberfläche;
- künstliche Upload-Prozentwerte ohne belastbare Datenquelle;
- automatische DAW-Synchronisation oder Driftkorrektur.

# Aktueller Implementierungsstand

Der funktionale V1-Pfad ist in `develop` vorhanden. Die Oberfläche kann Recording-Zustände darstellen, Host-Aktionen auslösen, READY-/Opening-Fortschritt anzeigen, Aufnahmezeit darstellen und den Production-Status berücksichtigen.

Vor einer Beta-/App-Store-Freigabe sind insbesondere noch visuelle Produktpolitur, konsistente Statusdarstellung, reale Browser-/Talk-Validierung und die allgemeine Release-Härtung erforderlich.

Diese verbleibenden Arbeiten ändern die fachliche Produktentscheidung dieser ADR nicht.

# Beziehung zu bestehenden Entscheidungen

Diese ADR baut insbesondere auf:

- ADR-005 Consent and Recording Transparency
- ADR-031 Identity, Authentication and User Roles
- ADR-068 Recording Start and Audio Synchronization Signet
- ADR-069 Nextcloud Remote Artifact Storage v1
- ADR-071 Recording Capture, Preservation and Transport Formats
- ADR-072 Host-Integrated Local Audio Capture via Connector
- ADR-073 Local Recording Safety Cutoff After Connectivity Loss
- ADR-075 Local Capture Independence from Communication Pipeline
- ADR-076 Event-Driven Recording Coordination
- ADR-083 Verified Nextcloud Artifact Transport
- ADR-084 Recording Artifact Aggregation and Completion Semantics
- ADR-085 Production Completion, Exceptional Closure and Late Artifact Delivery

# Status

Die Entscheidung gilt als angenommen.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# Context

NC-PoRE presents recording functionality inside the Nextcloud Talk experience in V1. Fachliche recording truth remains in the Core/Application layer; the Talk integration supplies host and participant context and exposes the PoRE function.

The current V1 implementation already has a functional Talk-like recording surface with:

- explicit **Start recording** for the host;
- explicit **Stop recording** for the host;
- authoritative recording-state presentation;
- READY and Opening progress for the host;
- elapsed recording time;
- a participant-specific recording surface;
- a Host action for finally closing an already stopped Production.

The surface is therefore no longer only technical validation UI. It is a usable PoRE product surface that still requires visual and functional hardening before a Beta/App-Store release.

# Decision

V1 presents recording as a **PoRE layer inside the host experience**. For Nextcloud Talk this means a compact Talk-like surface rather than an independent recording window.

The UI remains intentionally thin:

- Core/Application provide authoritative recording state;
- the UI renders that state;
- user actions are forwarded to the defined fachliche command boundaries;
- Talk-specific media and session details remain in the connector;
- technical diagnostics do not become the primary product surface.

The UI must not create a parallel fachliche recording state machine.

# Roles and Visibility

## Host

The host may see information relevant to the concrete recording:

- current recording state;
- expected recording participants;
- their READY state during preparation;
- aggregate readiness;
- Opening confirmations during the transition;
- elapsed recording time while actively recording;
- **Start recording**;
- **Stop recording**;
- post-stop completion/transfer state;
- where applicable, **Close Production permanently** after a stopped recording.

The participant set refers to the concrete recording, not the entire Talk session.

## Recording Participants

A recording participant sees at minimum:

- their own recording state;
- their own READY state;
- an unmistakable indication when local recording is active;
- the completion state relevant to their contribution.

Technical identities, transport handles and internal connector details do not belong in the normal product surface.

## Non-Participants

A session member who is not part of the concrete recording receives no recording status information.

In particular, they must not see:

- whether other users are recording;
- other users' READY state;
- stop or transfer progress;
- artifact status.

# State Presentation

The UI distinguishes at least these fachlich relevant states:

- preparation;
- READY;
- Opening;
- Recording;
- stopping or transfer;
- recording stopped;
- recording confirmed;
- Production closed;
- error.

The visual representation must not alter these meanings through local or incidental UI states.

Color alone must never carry the meaning. Text, symbol or accessible semantic information must make the state understandable as well.

A technical transition such as waiting for Opening confirmation is a presentation detail, not a new fachlicher recording state.

# Recording Controls

The primary user actions are explicit:

- **Start recording**
- **Stop recording**

For the Host the start path is:

```text
Preparation
    ->
local capture
    ->
READY
    ->
Opening
    ->
Recording
```

The stop path is:

```text
Recording
    ->
Stopping
    ->
local technical completion
    ->
Artifact/transport completion
    ->
Recording Completed
```

A successful technical sub-step must not be presented as a fachliche overall confirmation.

In particular:

- local recorder stop is not server-side artifact confirmation;
- successful transport of one participant artifact is not `core.Recording.Completed`;
- `core.Recording.Completed` is not `core.Production.Completed`.

Under ADR-085, the Production may explicitly close before all artifact delivery has completed.

# Recording Transparency

A user must be able to understand:

- whether recording is actually active;
- whether they participate in the recording;
- whether their local recording is technically ready;
- whether recording has already stopped;
- whether processing or transfer is still running;
- whether server-side confirmation has already been reached.

The UI must not replace these fachlich meaningful statements with technical terms such as `MediaRecorder`, track IDs, EventSource or internal transport state.

# Talk Integration

The surface is integrated as a PoRE function inside the Talk experience.

The following boundaries apply:

- Talk remains responsible for its conversation and media UI;
- PoRE remains responsible for recording functionality;
- the integration uses an explicit mount/context path;
- fachliche decisions are not derived from Talk DOM state;
- random or version-dependent DOM selectors are not fachliche APIs.

The exact visual embedding may still improve during alpha hardening without changing these responsibility boundaries.

# Accessibility

Recording states and actions must remain understandable without color perception.

This includes:

- accessible names for PoRE controls;
- semantic state information;
- clearly labeled host actions;
- an unmistakable active-recording indication;
- no purely visual or blinking indicator as the sole status information.

# Deliberate V1 Non-Goals

This ADR does not define:

- a full production or studio dashboard;
- a standalone browser recording application outside the host experience;
- a complete technical diagnostic console;
- pixel-perfect reproduction of the Talk UI;
- artificial upload percentages without a reliable source;
- automatic DAW synchronization or drift correction.

# Current Implementation Status

The functional V1 path is present in `develop`. The UI can render recording states, trigger host actions, show READY/Opening progress, display recording time and account for Production status.

Before a Beta/App-Store release, visual product polish, consistent status presentation, real browser/Talk validation and general release hardening are still required.

These remaining tasks do not change the product decision recorded by this ADR.

# Relationship to Existing Decisions

This ADR builds in particular on:

- ADR-005 Consent and Recording Transparency
- ADR-031 Identity, Authentication and User Roles
- ADR-068 Recording Start and Audio Synchronization Signet
- ADR-069 Nextcloud Remote Artifact Storage v1
- ADR-071 Recording Capture, Preservation and Transport Formats
- ADR-072 Host-Integrated Local Audio Capture via Connector
- ADR-073 Local Recording Safety Cutoff After Connectivity Loss
- ADR-075 Local Capture Independence from Communication Pipeline
- ADR-076 Event-Driven Recording Coordination
- ADR-083 Verified Nextcloud Artifact Transport
- ADR-084 Recording Artifact Aggregation and Completion Semantics
- ADR-085 Production Completion, Exceptional Closure and Late Artifact Delivery

# Status

This decision is accepted.
