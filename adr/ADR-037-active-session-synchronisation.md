# Deutsch ([English version below](#english-version))

# ADR-037: Active Session Synchronisation

## Status

Proposed

## Date

2026-08-17

## Decision Type

Architecture

---

# Kontext

NC-PoRe zeichnet Teilnehmer unabhängig auf unterschiedlichen Endgeräten auf. ADR-009 etabliert bereits ein sample-basiertes internes Zeitmodell, aber ein gemeinsamer Startzeitpunkt allein kann keine Synchronität über lange Aufnahmesessions garantieren.

Unterschiedliche Geräte können unterschiedliche Clock Sources verwenden und über die Dauer einer Aufnahme Clock Drift aufweisen.

Die Betrachtung von Ennuicastr zeigt einen interessanten Ansatz aktiver Synchronisation, bei dem Clients kontinuierlich ihre Beziehung zu einer serverseitigen Zeitreferenz bestimmen und Zeitinformationen an die aufgenommenen Daten anfügen.

---

# Entscheidung

NC-PoRe behandelt **aktive Session-Synchronisation** als ausdrückliches Architekturthema.

Eine verteilte Recording Session muss eine kontinuierlich nutzbare zeitliche Beziehung zwischen den Teilnehmern aufrechterhalten, statt sich ausschließlich auf einen gemeinsamen Startzeitpunkt zu verlassen.

Das konkrete Synchronisationsprotokoll wird durch diese ADR bewusst noch nicht festgelegt. Es muss mindestens unterstützen:

* eine stabile Session-Zeitreferenz
* teilnehmerspezifische Zeitinformationen
* Positionierung auf Sample- oder Frame-Ebene
* Erkennung und Behandlung von Clock Drift
* Rekonstruktion von Tracks nach Unterbrechungen
* hinreichende zeitliche Genauigkeit für lange Aufnahmen

Das Protokoll muss unabhängig vom späteren Storage Provider und vom Produktions-Ausgabeformat bleiben.

---

# Architekturprinzip

> Eine verteilte Recording Session wird kontinuierlich synchronisiert, nicht lediglich gleichzeitig gestartet.

Synchronisationsmetadaten sind Bestandteil der Capture-Informationen, die für eine zuverlässige Rekonstruktion erforderlich sind.

---

# Konsequenzen

## Positive Auswirkungen

* lange Aufnahmen können trotz unterschiedlicher Geräte-Clocks zeitlich konsistent bleiben
* Clock Drift wird zu einem expliziten und testbaren Problem
* Synchronisation kann dieselbe zeitliche Grundlage für Recovery und Rejoin verwenden
* Synchronisation bleibt unabhängig von finalen Medienformaten

---

## Negative Auswirkungen

* zusätzliche Protokoll- und Zustandskomplexität
* Synchronisationsqualität muss gemessen und getestet werden
* Netzwerk-Timing und vorübergehende Verbindungsprobleme müssen sorgfältig behandelt werden

---

# Betrachtete Alternativen

## Nur gemeinsamer Startzeitpunkt

Verworfen. Unterschiedliche Geräte-Clocks können driften, wodurch eine Synchronisation allein über den Startzeitpunkt bei langen Aufnahmen unzureichend wird.

---

## Ausschließliche manuelle Ausrichtung in der Postproduktion

Als primärer Mechanismus verworfen. Manuelle Ausrichtung kann als optionales Produktionswerkzeug sinnvoll bleiben, aber das Capture-System muss selbst zuverlässige zeitliche Informationen bereitstellen.

---

# Beziehung zu bestehender Architektur

Diese Entscheidung **konkretisiert ADR-009**. Das dort etablierte sample-basierte Track-Zeitmodell bleibt gültig und bildet die Datenebene, auf der eine aktive Synchronisationsmechanik aufsetzt.

Sie steht außerdem in direktem Zusammenhang mit den im Core definierten Entscheidungen zu Recording Lifecycle und Recovery.

---

# Zukünftige Betrachtungen

Eine spätere ADR muss das konkrete Synchronisationsprotokoll, die Clock-Schätzung, die Strategie zur Drift-Korrektur, Toleranzgrenzen und Tests für Sessions von mehreren Stunden definieren.

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-037: Active Session Synchronisation

## Status

Proposed

## Date

2026-08-17

## Decision Type

Architecture

---

# Context

NC-PoRe records participants independently on different devices. ADR-009 already establishes a sample-based internal time model, but a shared start time alone cannot guarantee synchronization over long recording sessions.

Different devices may use different clock sources and may exhibit clock drift over the duration of a recording.

The review of Ennuicastr demonstrates an interesting active synchronization approach in which clients continuously establish their relationship to a server-side time reference and attach timing information to captured data.

---

# Decision

NC-PoRe treats **active session synchronization** as an explicit architectural concern.

A distributed recording session must maintain a continuously usable temporal relationship between participants rather than relying solely on a common start time.

The concrete synchronization protocol is deliberately not fixed by this ADR. It must support at least:

* a stable session time reference
* participant-specific timing information
* sample- or frame-level positioning
* detection and handling of clock drift
* reconstruction of tracks after interruptions
* sufficient timing accuracy for long recordings

The protocol must remain independent of the eventual storage provider and production output format.

---

# Architectural Principle

> A distributed recording session is synchronized continuously, not merely started simultaneously.

Synchronization metadata is part of the capture information required for reliable reconstruction.

---

# Consequences

## Positive Effects

* long recordings can remain temporally coherent despite different device clocks
* clock drift becomes an explicit and testable concern
* synchronization can provide the same temporal basis for recovery and rejoin
* synchronization remains independent of final media formats

---

## Negative Effects

* additional protocol and state complexity
* synchronization quality must be measured and tested
* network timing and temporary connectivity problems must be handled carefully

---

# Alternatives Considered

## Common Start Time Only

Rejected. Different device clocks can drift, making start-time-only synchronization insufficient for long recordings.

---

## Post-Production Manual Alignment Only

Rejected as the primary mechanism. Manual alignment may remain useful as an optional production tool, but the capture system must provide reliable temporal information itself.

---

# Relationship to Existing Architecture

This decision **refines ADR-009**. The sample-based track time model established there remains valid and becomes the data-level representation on which an active synchronization mechanism operates.

It also relates directly to the Recording Lifecycle and recovery decisions defined in the Core.

---

# Future Considerations

A later ADR must define the concrete synchronization protocol, clock estimation, drift correction strategy, tolerance limits and test scenarios for sessions lasting several hours.
