# NC-PoRE Recording Information Surface V1

## Purpose

Define the product-facing recording information and control surface for the first Nextcloud Talk integration.

This document complements **ADR-072 Recording Status and Visual Semantics**. ADR-072 defines the visual meaning of statuses; this document defines what recording information is shown to whom and where it belongs in the Talk experience.

The UI is a **PoRE layer integrated into Nextcloud Talk**. It is not a separate browser recording application and must not depend on brittle Talk DOM selectors where an explicit integration point can be used.

---

## 1. Visibility model

Recording information is visible according to recording participation and role.

| User | Recording information |
|---|---|
| **Host** | Full recording control and participant/readiness overview |
| **Recording participant** | Own recording state, own READY state and local recording indication |
| **Session member not participating in this recording** | No recording status information |

Session membership alone does not grant visibility into a recording. The recording participant set is the set defined for the current recording start.

Technical diagnostics such as source-track identity, browser codec/container and connector state are secondary diagnostics and do not dominate the normal production UI.

---

## 2. Host surface

The host needs one coherent recording area within the Talk experience.

### Before recording

The host can see:

- whether a recording is idle or being prepared;
- which session members are included in the recording participant set;
- their current readiness while preparation is in progress;
- aggregate readiness such as `2 / 3 bereit` when useful;
- the primary **Aufnahme starten** action when the session is able to start recording.

The recording participant set is frozen for the current start operation. A newly joined session member does not silently become part of an already prepared recording.

### During recording

The host sees:

- unmistakable active recording state;
- recording start time and/or elapsed time;
- recording participants;
- primary **Aufnahme beenden** action.

The active recording indicator must not be confused with the Talk connection indicator.

### After stop

The host sees:

- stopping / processing / transfer state;
- actual transfer progress only when a reliable progress value exists;
- final server confirmation;
- artifact availability when confirmed.

`100 %` transfer is not itself the completed state. Completion is represented only after server confirmation.

---

## 3. Recording participant surface

A recording participant sees a compact representation of their own recording state.

During preparation this includes:

- whether local recording is not ready;
- `READY` once local capture is actually active.

During recording it includes:

- an unmistakable indication that local recording is active.

After stop it includes the relevant stopping/processing state and, where appropriate, confirmation that the local contribution has been accepted.

A participant does not need to see technical identifiers or connector details during normal operation.

---

## 4. Non-participant surface

A session member who is not part of the current recording participant set sees **no recording status**.

In particular, the UI must not expose:

- whether another member is recording;
- another member's READY state;
- recording start/stop progress;
- artifact or transfer status.

This keeps recording participation separate from ordinary session presence.

---

## 5. Status semantics

The UI uses the semantics established by ADR-072 and does not redefine them locally:

| Visual state | Meaning |
|---|---|
| **Grau** | Session wird erstellt bzw. vorbereitet |
| **Schwarz** | Listener; nicht an der Aufnahme beteiligt |
| **Rot** | nicht aufnahmebereit bzw. technischer Fehler |
| **Gelb** | technisch bereit (`READY`) |
| **Grün** | Aufnahme läuft |
| **Blau** | Aufnahme beendet; Verarbeitung/Übertragung läuft |
| **Weiß + grüner Haken** | serverseitig bestätigt und abgeschlossen |

Black is a participation indicator for a listener, not a recording state.

Blinking green may indicate the technical transition between local capture activation and confirmed Opening Sync Signet. It is not a separate domain state.

Blue progress must never be invented. If no reliable percentage is available, the UI remains blue or uses an activity indicator.

The status must be communicated through more than color alone. Text, symbol, shape or accessible semantic information must supplement the visual indicator.

---

## 6. Recording lifecycle presentation

The UI should mirror the domain lifecycle rather than inventing a parallel state machine.

```text
Session preparation
       ↓
Recording start requested by host
       ↓
Recording participant set frozen
       ↓
Local capture starts
       ↓
READY confirmations
       ↓
Opening Sync Signet
       ↓
Recording active
       ↓
Host requests stop
       ↓
Closing Sync Signet
       ↓
Local recorder stop / OK confirmations
       ↓
Processing / transfer
       ↓
Server confirmation
       ↓
Completed
```

The UI may simplify this presentation for compact Talk surfaces, but it must not collapse states in a way that changes their meaning.

---

## 7. Separation from Talk connection state

Talk connection/presence and PoRE recording state are independent dimensions.

Examples:

- a connected Talk participant can be a listener;
- a recording participant can temporarily have a technical capture problem without the whole session ending;
- a recording can be in transfer/finalization while the Talk conversation remains connected;
- successful upload is not the same as server-confirmed completion.

The UI therefore must not reuse the Talk connection indicator as the recording indicator.

---

## 8. Integration boundary

The first production UI should be exposed through an explicit PoRE/Talk integration surface rather than by manipulating arbitrary Talk DOM elements.

Preferred direction:

- Talk owns the conversation shell, participant list and normal Talk controls;
- PoRE owns the recording-specific information and actions;
- the integration supplies the stable host/participant context and PoRE state;
- PoRE state is derived from the existing Application/Core lifecycle;
- technical connector details remain below the product-facing surface.

The exact Talk extension mechanism is an implementation concern and is intentionally not fixed here. The requirement is that the integration boundary remains explicit and resilient to unrelated Talk UI changes.

---

## 9. Accessibility and recording transparency

The recording surface must:

- make active recording unmistakable;
- not rely on color alone;
- provide meaningful accessible names/state descriptions;
- expose the host's recording actions clearly;
- make preparation, active recording, stopping/transfer and confirmed completion distinguishable;
- avoid exposing recording information to users who are not participants in the current recording.

Recording controls and indicators should remain understandable on compact displays.

---

## 10. Scope boundary

V1 does **not** attempt to specify:

- Talk DOM selectors;
- a new standalone browser UI;
- a complete visual component library;
- exact pixel dimensions or icon geometry;
- technical diagnostics as part of the primary UI;
- automatic DAW alignment or drift correction.

Those concerns remain implementation or later product decisions.

---

## Related decisions

- ADR-005 Consent and Recording Transparency
- ADR-008 Client Architecture
- ADR-031 Identity, Authentication and User Roles
- ADR-039 Recording Architecture and Capture Boundary
- ADR-040 Recorder Workflow and Capture Lifecycle Coordination
- ADR-060 Recording Artifact Processing Lifecycle and Idempotency
- ADR-068 Recording Start and Audio Synchronization Signet
- ADR-071 Recording Capture, Preservation and Transport Formats
- ADR-072 Recording Status and Visual Semantics
