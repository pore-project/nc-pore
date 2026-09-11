# ADR-074 Talk Recording Control Surface V1

* Status: Proposed
* Date: 2026-09-03
* Decision Type: Product / UX

---

# Deutsch ([English version below](#english-version))

---

# Kontext

NC-PoRE soll die Recording-Funktionen in die Benutzeroberfläche von Nextcloud Talk integrieren. Die bestehende Browser-Vertical-Slice beweist den fachlichen Lifecycle über Core/Application bis zur lokalen Browser-Aufnahme und Artefaktübergabe. Sie ist jedoch ausdrücklich keine Produktions-UI und kein Talk-Produktionsprotokoll.

Die UI muss deshalb die bereits festgelegte fachliche Wahrheit sichtbar machen, ohne eine eigene Recording-State-Machine im Browser zu erfinden.

Die verbindliche visuelle Grundsemantik ist in ADR-072 festgelegt. Insbesondere gelten Grau, Schwarz, Rot, Gelb, Grün, Blau und Weiß + grüner Haken mit ihren dort definierten Bedeutungen.

---

# Entscheidung

V1 stellt die Recording-Funktion als **PoRE-Schicht innerhalb von Nextcloud Talk** dar. Es wird kein unabhängiges PoRE-Web-Panel als Produktionsoberfläche eingeführt.

Die UI besteht zunächst aus drei zusammengehörigen Bereichen:

1. **Recording-Status** – kompakte, dauerhaft verständliche Statusanzeige.
2. **Recording-Teilnehmer** – nur für Personen, für die die Information fachlich relevant ist.
3. **Aktionen** – explizite Aufnahme-Steuerung für den Host.

Die Darstellung bleibt bewusst klein und Talk-nah. Technische Debug-Informationen wie Track-ID, Codec, Container oder Connector-Zustand gehören nicht in die primäre Oberfläche.

---

# Sichtbarkeit und Rollen

## Host

Der Host sieht:

* den aktuellen Recording-Zustand,
* die am Recording beteiligten Personen,
* deren READY-Zustand während der Vorbereitung,
* bei Bedarf eine aggregierte Bereitschaft (`n / total bereit`),
* die verstrichene Aufnahmezeit während `Recording`,
* die Aktionen **Aufnahme starten** und **Aufnahme beenden**,
* nach dem Stop den Zustand der Verarbeitung und die bestätigte Artefaktübernahme.

Der Host sieht dabei ausschließlich die fachlich relevante Recording-Teilnehmermenge. Talk-Mitglieder, die nicht Teil des Recordings sind, werden nicht als Recording-Teilnehmer dargestellt.

## Recording-Teilnehmer

Ein Recording-Teilnehmer sieht:

* den eigenen Recording-Zustand,
* den eigenen READY-Zustand,
* eine eindeutige Anzeige, dass die lokale Aufnahme aktiv ist,
* die für den eigenen Ablauf relevanten Übergänge.

Die UI muss nicht die technischen Details der anderen Teilnehmer offenlegen.

## Nicht am Recording beteiligte Session-Mitglieder

Nicht beteiligte Session-Mitglieder erhalten keine Recording-Statusanzeige. Ihre Talk-Kommunikation bleibt von der Recording-Oberfläche unberührt.

---

# Zustandsdarstellung

Die UI verwendet die Semantik aus ADR-072 und darf deren Farben nicht mit lokalen Bedeutungen umdeuten:

| Status | UI-Bedeutung |
|---|---|
| **Grau** | Session wird erstellt / vorbereitet |
| **Schwarz** | Listener, nicht am Recording beteiligt |
| **Rot** | nicht bereit / technischer Fehler |
| **Gelb** | technisch bereit (`READY`) |
| **Blinkendes Grün** | lokale technische Übergangsphase nach aktivem Capture, Opening noch nicht bestätigt |
| **Grün** | Aufnahme läuft |
| **Blau** | Aufnahme beendet, Verarbeitung / Übertragung läuft |
| **Weiß + grüner Haken** | Artefakt serverseitig bestätigt / abgeschlossen |

Schwarz ist kein Recording-State. Blinkendes Grün ist ebenfalls kein zusätzlicher fachlicher State, sondern ein visueller technischer Übergangsmarker.

**Wichtig:** Blau bedeutet nicht automatisch „Upload erfolgreich“. `100 %` Transportfortschritt bedeutet nicht serverseitige Bestätigung. Erst die bestätigte Übernahme darf Weiß + grünen Haken anzeigen.

Es wird kein künstlicher Fortschritt angezeigt. Ein Prozentwert darf nur aus einer belastbaren Fortschrittsquelle stammen.

---

# Recording-Steuerung

Die primären Aktionen sind explizit und verständlich beschriftet:

* **Aufnahme starten**
* **Aufnahme beenden**

Die Startaktion wird nur angeboten, wenn der Benutzer fachlich dazu berechtigt ist. Die UI darf die Startentscheidung nicht allein aus lokaler Browserbereitschaft ableiten.

Während der Vorbereitung zeigt die UI den Fortschritt durch fachliche Zustände und READY-Informationen. Sie erzeugt keinen separaten lokalen „fast bereit“-Zustand.

Beim Start folgt die UI der fachlichen Sequenz:

```text
Vorbereitung
  ↓
READY
  ↓
Opening / lokales Capture aktiv
  ↓
Opening bestätigt
  ↓
Recording
```

Der Host kann die Aufnahme beenden. Danach gilt:

```text
Recording
  ↓
Stopping
  ↓
Stop bestätigt
  ↓
Verarbeitung / Übertragung
  ↓
Artefakt serverseitig bestätigt
  ↓
Weiß + grüner Haken
```

Die UI zeigt einen abgeschlossenen Zustand also erst nach der fachlichen bzw. serverseitigen Bestätigung, nicht bereits nach einem lokalen Stop oder Upload-Ende.

---

# Talk-Integration

Die Produktionsoberfläche wird als Bestandteil der Talk-Erfahrung umgesetzt. PoRE soll sich visuell an der vorhandenen Nextcloud/Talk-Oberfläche orientieren, ohne deren DOM-Struktur als API zu behandeln.

Insbesondere gilt:

* keine fachliche Logik in CSS-Selektoren oder DOM-Zuständen,
* keine Abhängigkeit von zufälligen oder versionsabhängigen Talk-Element-IDs,
* keine eigene parallele Recording-State-Machine im Browser,
* Core/Application liefern die fachliche Wahrheit; die UI stellt sie dar und sendet Benutzeraktionen an die vorgesehenen Application-Schnittstellen.

Der konkrete technische Extension Point von Nextcloud Talk wird in der Integrationsarbeit festgelegt. Diese ADR schreibt bewusst keinen fragilen DOM-Hook vor.

---

# Accessibility und Transparenz

Recording muss für den Benutzer jederzeit eindeutig erkennbar sein.

Farbe ist niemals die einzige Bedeutungsträgerin. Aktive, kritische und abgeschlossene Zustände benötigen zusätzlich Text, Symbol, Form oder zugängliche semantische Kennzeichnung.

Insbesondere muss ein Benutzer erkennen können:

* dass tatsächlich aufgenommen wird,
* ob er selbst am Recording beteiligt ist,
* ob sein lokales Capture bereit bzw. aktiv ist,
* ob die Aufnahme beendet wurde, aber noch verarbeitet wird,
* ob das Ergebnis bereits serverseitig bestätigt wurde.

Bewegung wie Blinkeffekte darf nicht die einzige Information sein und muss sparsam eingesetzt werden.

Die UI darf keine Recording-Transparenz durch rein technische Begriffe wie `MediaRecorder`, Track-ID oder Codec ersetzen.

---

# Fehler und lokale technische Probleme

Ein lokaler Capture-Fehler kann den betroffenen Teilnehmer als nicht bereit bzw. fehlerhaft anzeigen. Er beendet nicht automatisch die gesamte Talk-Session und erzeugt keine neue Recording-Session.

Die UI soll einen lokal behebbaren Fehler als solchen erkennbar machen und nach erfolgreicher Wiederherstellung wieder den fachlich passenden Zustand anzeigen.

Der Host entscheidet über das Ende des Recordings. Ein technischer Fehler eines einzelnen Teilnehmers darf diese fachliche Entscheidung nicht stillschweigend ersetzen.

---

# Nicht Bestandteil von V1

V1 benötigt noch keine:

* frei konfigurierbare Produktions-Dashboard-Ansicht,
* vollständige technische Diagnoseoberfläche,
* eigene WebSocket-Infrastruktur ausschließlich für die UI,
* pixelgenaue Nachbildung jedes Talk-UI-Details,
* künstliche Upload-Prozentanzeigen,
* zusätzliche fachliche Zustände nur für die Darstellung.

Die Oberfläche soll aber so strukturiert sein, dass spätere Teilnehmerdetails, Synchronisationshinweise und Signet-Feedback ergänzt werden können.

---

# Konsequenzen

## Vorteile

* Benutzer sehen die Recording-Funktion dort, wo die Kommunikation stattfindet.
* Fachliche Recording-Wahrheit bleibt beim Core/Application-Layer.
* Die UI bleibt dünn und gegenüber Talk-DOM-Änderungen robuster.
* Die bereits vereinbarte Statussemantik bleibt über Browser und spätere Clients konsistent.
* V1 ist klein genug, um als echte Produktionsintegration weitergebaut zu werden.

## Kosten

* Die eigentliche Talk-Integration benötigt einen stabilen, unterstützten Extension Point.
* UI-Komponenten müssen Rollen- und Sichtbarkeitsregeln beachten.
* Accessibility muss von Anfang an mitimplementiert werden.

---

# Beziehung zu bestehenden Entscheidungen

Diese ADR baut insbesondere auf:

* ADR-005 Consent and Recording Transparency
* ADR-008 Client Architecture
* ADR-031 Identity, Authentication and User Roles
* ADR-039 Recording Architecture and Capture Boundary
* ADR-040 Recorder Workflow and Capture Lifecycle Coordination
* ADR-060 Recording Artifact Processing Lifecycle and Idempotency
* ADR-068 Recording Start and Audio Synchronization Signet
* ADR-071 Recording Capture, Preservation and Transport Formats
* ADR-072 Recording Status and Visual Semantics
* ADR-073 Local Recording Safety Cutoff After Connectivity Loss

Die ADR definiert die **V1-Produktdarstellung und Bedienung**. Sie ersetzt weder die fachliche Recording-State-Machine noch die Synchronisations- und Persistenzentscheidungen.

---

# Status

Proposed. Die Umsetzung erfolgt anschließend in der echten Nextcloud-Talk-Integration. Die Browser-Vertical-Slice bleibt technische Validierungsinfrastruktur und wird nicht zur Produktions-UI erklärt.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# Context

NC-PoRE integrates recording functionality into the Nextcloud Talk user experience. The existing browser vertical slice proves the domain lifecycle through Core/Application to local browser capture and artifact handoff. It is explicitly not the production UI and not the production Talk protocol.

The UI must therefore present established domain truth without inventing a parallel recording state machine in the browser.

ADR-072 defines the normative visual semantics. Gray, black, red, yellow, green, blue, and white + green check retain the meanings defined there.

---

# Decision

V1 presents recording as a **PoRE layer inside Nextcloud Talk**. No independent PoRE web panel is introduced as the production surface.

The initial UI consists of three coherent areas:

1. **Recording status** – compact and continuously understandable.
2. **Recording participants** – only for users for whom the information is relevant.
3. **Actions** – explicit recording controls for the host.

The UI remains compact and Talk-native. Technical debug details such as track IDs, codecs, containers, or connector state do not dominate the primary surface.

---

# Visibility and Roles

## Host

The host sees:

* current recording state,
* recording participants,
* their READY state during preparation,
* an aggregate readiness indicator (`n / total ready`) where useful,
* elapsed recording time while `Recording`,
* explicit **Start recording** and **Stop recording** actions,
* processing state and confirmed artifact handoff after stop.

Only the fachlich relevant recording participant set is shown. Talk members who are not part of the recording are not presented as recording participants.

## Recording Participant

A recording participant sees:

* their own recording state,
* their own READY state,
* an unmistakable indication that local recording is active,
* transitions relevant to their own flow.

Technical details about other participants are not required in V1.

## Non-recording Session Member

Members who do not participate in the recording receive no recording-status presentation. Their Talk communication remains unaffected by the recording surface.

---

# State Presentation

The UI uses ADR-072 semantics and must not assign local meanings to its colors:

| Status | UI meaning |
|---|---|
| **Gray** | Session is being created / prepared |
| **Black** | Listener, not participating in recording |
| **Red** | Not ready / technical error |
| **Yellow** | Technically ready (`READY`) |
| **Blinking green** | Local technical transition after capture activation, opening not yet confirmed |
| **Green** | Recording is active |
| **Blue** | Recording ended; processing / transfer is running |
| **White + green check** | Artifact server-confirmed / completed |

Black is not a recording state. Blinking green is not an additional domain state; it is a visual technical transition marker.

**Important:** Blue does not mean “upload successful”. `100%` transport progress does not mean server confirmation. Only confirmed acceptance may produce white + green check.

No artificial progress is shown. A percentage may only come from a reliable progress source.

---

# Recording Controls

The primary actions are explicit and understandable:

* **Start recording**
* **Stop recording**

The start action is only offered to users who are authorized to start the recording. The UI must not derive the start decision from local browser readiness alone.

During preparation, the UI communicates progress through domain states and READY information. It does not invent a separate local “almost ready” state.

The start flow follows:

```text
Preparation
  ↓
READY
  ↓
Opening / local capture active
  ↓
Opening confirmed
  ↓
Recording
```

After host stop:

```text
Recording
  ↓
Stopping
  ↓
Stop acknowledged
  ↓
Processing / transfer
  ↓
Artifact server-confirmed
  ↓
White + green check
```

A completed state is therefore shown only after domain/server confirmation, not merely after local stop or upload completion.

---

# Talk Integration

The production surface is implemented as part of the Talk experience. PoRE should follow the existing Nextcloud/Talk visual language without treating Talk DOM structure as an API.

In particular:

* no domain logic in CSS selectors or DOM state,
* no dependency on accidental or version-specific Talk element IDs,
* no parallel recording state machine in the browser,
* Core/Application owns domain truth; the UI presents it and sends user actions through the intended Application interfaces.

The concrete supported Talk extension point is determined during integration work. This ADR deliberately does not prescribe a brittle DOM hook.

---

# Accessibility and Transparency

Recording must remain unmistakable to the user.

Color is never the sole carrier of meaning. Active, critical, and completed states require additional text, symbols, shape, or accessible semantic labeling.

Users must be able to understand:

* that recording is actually active,
* whether they participate in the recording,
* whether local capture is ready or active,
* whether recording has stopped but processing is still running,
* whether the result has already been server-confirmed.

Motion such as blinking must not be the sole information carrier and should be used sparingly.

The UI must not replace recording transparency with technical terms such as `MediaRecorder`, track IDs, or codecs.

---

# Errors and Local Technical Problems

A local capture error may render the affected participant not-ready or in an error condition. It does not automatically terminate the entire Talk session or create a new recording session.

The UI should make locally recoverable errors recognizable and return to the appropriate domain state after recovery.

The host decides when recording ends. A technical error for one participant must not silently replace that domain decision.

---

# Out of Scope for V1

V1 does not require:

* a freely configurable production dashboard,
* a full technical diagnostics surface,
* dedicated WebSocket infrastructure solely for the UI,
* pixel-perfect reproduction of every Talk UI detail,
* artificial upload percentages,
* additional domain states created only for presentation.

The surface should nevertheless leave room for richer participant details, synchronization hints, and signet feedback later.

---

# Consequences

## Benefits

* Users see recording where the communication happens.
* Domain recording truth remains in Core/Application.
* The UI stays thin and more resilient to Talk DOM changes.
* The established status semantics remain consistent across browser and future clients.
* V1 is small enough to become a real production integration.

## Costs

* The Talk integration needs a stable supported extension point.
* UI components must enforce role and visibility rules.
* Accessibility must be implemented from the beginning.

---

# Relationship to Existing Decisions

This ADR builds in particular on:

* ADR-005 Consent and Recording Transparency
* ADR-008 Client Architecture
* ADR-031 Identity, Authentication and User Roles
* ADR-039 Recording Architecture and Capture Boundary
* ADR-040 Recorder Workflow and Capture Lifecycle Coordination
* ADR-060 Recording Artifact Processing Lifecycle and Idempotency
* ADR-068 Recording Start and Audio Synchronization Signet
* ADR-071 Recording Capture, Preservation and Transport Formats
* ADR-072 Recording Status and Visual Semantics
* ADR-073 Local Recording Safety Cutoff After Connectivity Loss

This ADR defines the **V1 product presentation and controls**. It does not replace the domain recording state machine or synchronization and persistence decisions.

---

# Status

Proposed. Implementation follows in the real Nextcloud Talk integration. The browser vertical slice remains technical validation infrastructure and is not declared the production UI.
