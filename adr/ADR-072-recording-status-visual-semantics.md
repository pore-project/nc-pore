# ADR-072 Recording Status and Visual Semantics

* Status: Accepted
* Date: 2026-09-03
* Decision Type: Product / UX

---

# Deutsch ([English version below](#english-version))

---

# Kontext

NC-PoRE verwendet einen verteilten Recording-Lifecycle, bei dem fachlicher Recording-Zustand, lokale technische Verarbeitung und Listener-/Teilnehmerstatus unterschiedliche Bedeutungen haben. Die Benutzeroberfläche muss diese Zustände schnell und eindeutig erkennbar machen, ohne fachliche Zustände zu vermischen.

Für die Statusdarstellung wurden im V1-Design bewusst Farben, Formen und Bewegungsmodifikatoren definiert. Die Darstellung soll insbesondere in einer Talk-Integration auf engem Raum funktionieren und auch dann verständlich bleiben, wenn ein Benutzer nicht aus dem Farbton allein auf die Bedeutung schließen kann.

Diese Entscheidung rekonstruiert und formalisiert die im August 2026 gemeinsam festgelegte Statussemantik.

---

# Entscheidung

NC-PoRE verwendet folgende verbindliche visuelle Grundsemantik für Recording-nahe Statusanzeigen:

| Darstellung | Bedeutung |
|---|---|
| **Grau** | Session wird erstellt bzw. vorbereitet |
| **Schwarz** | Listener; nicht an der Aufnahme beteiligt |
| **Rot** | nicht aufnahmebereit bzw. technischer Fehler |
| **Gelb** | Aufnahme vorbereitet und technisch bereit (`READY`) |
| **Grün** | Aufnahme läuft |
| **Blau** | Aufnahme abgeschlossen; Verarbeitung bzw. Übertragung läuft |
| **Weiß + grüner Haken** | Artefakt serverseitig bestätigt und erfolgreich abgeschlossen |

Schwarz ist ausdrücklich **kein Zustand der Recording-State-Machine**. Es bezeichnet die Recording-Beteiligung als Listener.

Grau bezeichnet die Vorbereitungsphase der Session und ist von Gelb (`READY`) zu unterscheiden.

Rot bezeichnet einen nicht bereiten bzw. fehlerhaften technischen Zustand. Ein Capture-Fehler beendet nicht automatisch die gesamte Session.

---

# Visuelle Modifikatoren

Farben werden bei Bedarf durch Form oder Bewegung ergänzt.

## Blinkendes Grün

Blinkendes Grün bedeutet einen lokalen technischen Übergang innerhalb des laufenden Startvorgangs:

```text
READY
  ↓
Capture aktiv / Opening noch nicht bestätigt
  ↓
Opening bestätigt
  ↓
Grün dauerhaft
```

Blinkendes Grün ist **kein zusätzlicher fachlicher Recording-State**. Es zeigt eine technische Übergangsbedingung an.

## Blau und Übertragungsfortschritt

Während Verarbeitung oder Übertragung kann Blau durch einen Fortschrittsindikator ergänzt werden.

Wenn ein belastbarer tatsächlicher Prozentwert verfügbar ist, darf ein **blauer Fortschrittsring** den gemeldeten Fortschritt darstellen. Konzeptuell kann dabei ein weißer Kreis durch einen blauen Ring bzw. eine blaue Füllung entsprechend dem tatsächlichen Fortschritt aufgebaut werden.

Es gilt ausdrücklich:

> **NC-PoRE zeigt keinen erfundenen Fortschritt.**

Wenn kein belastbarer Prozentwert verfügbar ist, bleibt der Zustand blau oder verwendet einen geeigneten Aktivitätsindikator. Ein Prozess ohne sinnvoll bestimmbaren Prozentfortschritt darf beispielsweise blau blinken.

`100 % Upload` ist nicht gleichbedeutend mit serverseitiger Bestätigung. Erst die bestätigte Übernahme führt zum Zustand **Weiß + grüner Haken**.

---

# Form und Farbe

Farbe darf nicht die einzige Information sein, insbesondere nicht für kritische Zustände.

Für Warn- und Fehlerzustände können daher geometrische Formen und Symbole ergänzt werden. Kritische Warnungen sollen sich auch auf kleinen Anzeigen eindeutig von normalen Recording-Zuständen unterscheiden.

Die konkrete Pixelgröße, Strichstärke oder exakte Icon-Geometrie ist **keine** Entscheidung dieser ADR und bleibt der konkreten UI-Implementierung bzw. dem Designsystem überlassen.

---

# Status und fachliche Wahrheit

Die Darstellung folgt dem Grundsatz:

> **Die UI zeigt fachliche oder technische Zustände an; sie erfindet keine fachliche Wahrheit.**

Insbesondere gilt:

* Grün bedeutet nicht lediglich „Client läuft“, sondern aktive Aufnahme.
* Blau bedeutet nicht automatisch „Upload erfolgreich“, sondern Verarbeitung/Übertragung nach Abschluss der Aufnahme.
* Weiß + grüner Haken bedeutet bestätigten Abschluss.
* Ein lokaler Upload-Fortschritt darf nur angezeigt werden, wenn der zugrunde liegende Transport tatsächlich einen belastbaren Fortschrittswert liefert.
* Listener werden unabhängig vom Recording-Lifecycle als schwarz dargestellt.

---

# Accessibility

Die Statusdarstellung muss so umgesetzt werden, dass die Bedeutung nicht ausschließlich vom Farbton abhängt.

Mindestens kritische und aktive Zustände müssen zusätzlich über Text, Symbol, Form oder zugängliche semantische Information unterscheidbar sein.

Die visuelle Semantik ist deshalb als Kombination aus **Farbe + Form/Symbol + optionaler Bewegung + Text/Accessible Name** zu verstehen.

---

# Konsequenzen

## Vorteile

* Recording-, Listener- und Abschlusszustände sind konsistent und schnell erfassbar.
* Die Statusfarben bleiben über unterschiedliche Clients und Hostintegrationen hinweg semantisch stabil.
* Upload-Fortschritt wird nur dann visualisiert, wenn er technisch belastbar ist.
* Fachlicher Abschluss und bloßer Upload-Fortschritt werden klar unterschieden.
* Die Semantik ist nicht an eine konkrete UI-Technologie gebunden.

## Kosten

* UI-Komponenten müssen die Statussemantik zentral verwenden und dürfen keine eigenen Bedeutungen für dieselben Farben erfinden.
* Für gute Accessibility müssen Farbe und zusätzliche semantische Kennzeichnung gemeinsam umgesetzt werden.
* Fortschrittsanzeigen benötigen eine belastbare Quelle für den tatsächlichen Fortschritt.

---

# Beziehung zur bestehenden Architektur

Diese Entscheidung baut insbesondere auf den Recording-, Client-, Artifact- und Synchronisationsentscheidungen des Projekts auf, insbesondere auf:

* ADR-005 Consent and Recording Transparency
* ADR-008 Client Architecture
* ADR-031 Identity, Authentication and User Roles
* ADR-039 Recording Architecture and Capture Boundary
* ADR-040 Recorder Workflow and Capture Lifecycle Coordination
* ADR-060 Recording Artifact Processing Lifecycle and Idempotency
* ADR-068 Recording Start and Audio Synchronization Signet
* ADR-071 Recording Capture, Preservation and Transport Formats

Diese ADR definiert **die Darstellung** der Zustände. Sie ersetzt keine fachliche Lifecycle- oder Synchronisationsentscheidung.

---

# Status

Accepted. Die Statussemantik ist Teil der öffentlichen NC-PoRE-Produktdefinition und soll bei künftigen Clients und Hostintegrationen konsistent verwendet werden.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# Context

NC-PoRE uses a distributed recording lifecycle in which domain recording state, local technical processing, and listener/participant status have different meanings. The user interface must make these states quickly and unambiguously recognizable without conflating domain states.

The V1 design deliberately defined colors, shapes, and motion modifiers for status presentation. The presentation must work particularly well in a compact Talk integration and remain understandable when a user cannot rely on color alone.

This decision reconstructs and formalizes the status semantics jointly established during the August 2026 design work.

---

# Decision

NC-PoRE uses the following normative visual semantics for recording-related status indicators:

| Presentation | Meaning |
|---|---|
| **Gray** | Session is being created or prepared |
| **Black** | Listener; not participating in the recording |
| **Red** | Not record-ready or technical error |
| **Yellow** | Recording prepared and technically ready (`READY`) |
| **Green** | Recording is active |
| **Blue** | Recording has ended; processing or transfer is in progress |
| **White + green check** | Artifact has been server-confirmed and successfully completed |

Black is explicitly **not a state of the recording state machine**. It represents recording participation as a listener.

Gray represents session preparation and must be distinguished from yellow (`READY`).

Red represents a technical not-ready or error condition. A capture error does not automatically terminate the entire session.

---

# Visual Modifiers

Colors may be supplemented by shape or motion.

## Blinking Green

Blinking green represents a local technical transition during recording start:

```text
READY
  ↓
Capture active / opening not yet confirmed
  ↓
Opening confirmed
  ↓
Solid green
```

Blinking green is **not an additional domain recording state**. It indicates a technical transition condition.

## Blue and Transfer Progress

During processing or transfer, blue may be supplemented by a progress indicator.

When a reliable actual percentage is available, a **blue progress ring** may represent the reported progress. Conceptually, a white circle may be filled by a blue ring or blue progress arc according to the actual transfer progress.

The rule is explicit:

> **NC-PoRE never displays invented progress.**

If no reliable percentage is available, the state remains blue or uses an appropriate activity indicator. A process without a meaningful determinable percentage may, for example, blink blue.

`100% upload` does not mean server confirmation. Only confirmed server-side acceptance leads to **white + green check**.

---

# Shape and Color

Color must not be the only source of meaning, especially for critical states.

Warning and error states may therefore be supplemented by geometric shapes and symbols. Critical warnings should remain clearly distinguishable from normal recording states on small displays.

Exact pixel dimensions, stroke widths, or icon geometry are **not** decided by this ADR and remain implementation/design-system concerns.

---

# Status and Domain Truth

The presentation follows this principle:

> **The UI presents domain or technical state; it does not invent domain truth.**

In particular:

* Green means active recording, not merely a running client.
* Blue does not automatically mean “upload successful”; it means post-recording processing/transfer.
* White + green check means confirmed completion.
* Local transfer progress may only be displayed when the underlying transport provides a reliable progress value.
* Listeners are represented as black independently of the recording lifecycle.

---

# Accessibility

Status presentation must be implemented so that meaning does not depend solely on hue.

At minimum, critical and active states must also be distinguishable through text, symbol, shape, or accessible semantic information.

The visual semantics therefore combine **color + shape/symbol + optional motion + text/accessible name**.

---

# Consequences

## Benefits

* Recording, listener, and completion states are consistent and quickly recognizable.
* Status colors retain stable meaning across clients and host integrations.
* Transfer progress is shown only when technically reliable.
* Domain completion and mere transfer progress are clearly distinguished.
* The semantics are independent of a particular UI technology.

## Costs

* UI components must use the central semantics and must not invent local meanings for the same colors.
* Good accessibility requires color and additional semantic labeling to be implemented together.
* Progress indicators require a reliable source for actual progress.

---

# Relationship to Existing Architecture

This decision builds on the project's recording, client, artifact, and synchronization decisions, in particular:

* ADR-005 Consent and Recording Transparency
* ADR-008 Client Architecture
* ADR-031 Identity, Authentication and User Roles
* ADR-039 Recording Architecture and Capture Boundary
* ADR-040 Recorder Workflow and Capture Lifecycle Coordination
* ADR-060 Recording Artifact Processing Lifecycle and Idempotency
* ADR-068 Recording Start and Audio Synchronization Signet
* ADR-071 Recording Capture, Preservation and Transport Formats

This ADR defines **presentation semantics**. It does not replace domain lifecycle or synchronization decisions.

---

# Status

Accepted. The status semantics are part of the public NC-PoRE product definition and should be used consistently by future clients and host integrations.
