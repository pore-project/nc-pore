# Deutsch ([English version below](#english-version))

# ADR-063: Active Session Synchronisation

## Status

Accepted

## Date

2026-08-27

## Decision Type

Architecture

---

# Kontext

NC-PoRe zeichnet Teilnehmer unabhängig auf unterschiedlichen Endgeräten auf. ADR-009 etabliert ein sample-basiertes internes Zeitmodell. ADR-068 definiert den gemeinsamen Recording-Start sowie Opening- und Closing-Sync-Signets als explizite Audio-Referenzpunkte.

Ein gemeinsamer Startzeitpunkt allein reicht für lange verteilte Aufnahmen nicht aus, weil unterschiedliche Geräte-Clocks über die Aufnahme hinweg driften können. Gleichzeitig ist eine netzwerkbasierte Laufzeitsynchronisation für die erste produktive Recording-Ausbaustufe nicht erforderlich und darf die lokale Aufnahme nicht von der Verfügbarkeit des Netzes abhängig machen.

Daraus ergibt sich eine gestufte Architektur: Der Capture-Pfad muss bereits ausreichende zeitliche Informationen für spätere Rekonstruktion liefern; eine aktive Laufzeitsynchronisation wird als spätere Erweiterung darauf aufgesetzt.

---

# Entscheidung

NC-PoRe verwendet für die Synchronisation verteilter Recordings ein **zweistufiges Modell**.

## Stufe 1 — signet- und sample-basierte Synchronisationsgrundlage

Die aktuelle Recording-Architektur verwendet:

* das sample-basierte Track-Zeitmodell aus ADR-009
* einen expliziten gemeinsamen Recording-Start nach ADR-068
* ein Opening Sync Signet als gemeinsamen Audio-Referenzpunkt
* ein Closing Sync Signet als gemeinsamen End-Referenzpunkt
* teilnehmerspezifische Recording- und Capture-Metadaten

Damit kann jede lokale Spur unabhängig aufgezeichnet und anschließend anhand derselben Audio-Referenz in eine gemeinsame zeitliche Lage gebracht werden.

Die lokale Aufnahme bleibt vollständig unabhängig von Netzwerkverfügbarkeit.

## Stufe 2 — aktive Laufzeitsynchronisation

Eine spätere Ausbaustufe darf während einer laufenden Session zusätzliche Synchronisationsinformationen erfassen und verteilen, um Clock Drift über lange Sessions automatisch zu erkennen und zu kompensieren.

Diese Stufe ist **nicht Bestandteil der aktuellen Capture-Implementierung**. Sie wird hinter der bestehenden, transport- und storage-unabhängigen Synchronisationsgrenze ergänzt und darf die lokale Audioaufnahme nicht blockieren oder von einer permanenten Netzwerkverbindung abhängig machen.

Für Stufe 2 gelten folgende verbindliche Anforderungen:

* stabile Session-Zeitreferenz
* teilnehmerspezifische Zeitinformationen
* Sample- oder Frame-genaue Positionierung
* Erkennung und Quantifizierung von Clock Drift
* definierte Toleranzgrenzen
* Wiederaufnahme bzw. Rejoin nach Unterbrechungen
* ausreichende Genauigkeit für Sessions von mehreren Stunden
* Unabhängigkeit von Storage Provider und Produktions-Ausgabeformat

Das konkrete Protokoll, die Clock-Schätzung und die Drift-Korrektur werden in einer späteren, eigenen ADR entschieden, sobald die erforderlichen Mess- und Testdaten vorliegen.

---

# Architekturprinzip

> Eine verteilte Recording Session besitzt eine gemeinsame zeitliche Referenz; aktive Laufzeitsynchronisation ist eine optionale zweite Schicht und kein Bestandteil des lokalen Capture-Grundpfads.

Die Capture-Daten müssen bereits in Stufe 1 genügend Informationen enthalten, damit Tracks deterministisch rekonstruiert und anhand des gemeinsamen Sync Signets ausgerichtet werden können.

---

# Verbindliche Grenzen

Die Synchronisationsarchitektur darf:

* die lokale Aufnahme nicht vom Netzwerk abhängig machen
* keinen bestimmten Storage Provider voraussetzen
* kein finales Produktionsformat voraussetzen
* eine unterbrochene Verbindung nicht mit Audioverlust gleichsetzen
* einen späteren Rejoin nicht durch einen neuen Recording-Start erzwingen

Die aktive Synchronisation darf insbesondere **nicht** zum versteckten Echtzeit-Audio-Transport werden. NC-PoRe synchronisiert die Produktion und ihre zeitlichen Beziehungen, nicht den Audiostrom selbst.

---

# Konsequenzen

## Positive Auswirkungen

* Die erste produktive Ausbaustufe bleibt robust und offline-fähig.
* Opening- und Closing-Sync-Signets liefern einen realen gemeinsamen Referenzpunkt.
* Das sample-basierte Zeitmodell bleibt die stabile Datenbasis.
* Eine spätere Driftkorrektur kann ergänzt werden, ohne den Storage- oder Capture-Pfad neu zu entwerfen.
* Rejoin und Recovery können auf denselben zeitlichen Metadaten aufbauen.

## Negative Auswirkungen

* Die automatische Korrektur von Clock Drift ist zunächst nicht Bestandteil der Capture-Ausbaustufe.
* Für lange Sessions muss Stufe 2 später mit realen Messdaten validiert werden.
* Die spätere aktive Synchronisation bringt zusätzliche Protokoll-, Zustands- und Testkomplexität.

---

# Betrachtete Alternativen

## Nur gemeinsamer Startzeitpunkt

Verworfen. Unterschiedliche Geräte-Clocks können über lange Sessions driften.

## Permanente netzwerkbasierte Laufzeitsynchronisation als Voraussetzung der Aufnahme

Verworfen. Die lokale Audioaufnahme muss auch bei Offline-Zuständen vollständig und unabhängig funktionieren.

## Ausschließliche manuelle Ausrichtung ohne Synchronisationsmetadaten

Verworfen. ADR-068 liefert bereits einen gemeinsamen Audio-Referenzpunkt; zusätzlich müssen die Capture-Daten ihre zeitliche Identität und Sample-Positionen erhalten.

---

# Beziehung zu bestehender Architektur

Diese ADR konkretisiert ADR-009 und grenzt die aktive Laufzeitsynchronisation gegenüber der in ADR-068 bereits akzeptierten Signet-basierten ersten Ausbaustufe ab.

Die Synchronisationsdaten bleiben unabhängig von der in ADR-030 festgelegten Trennung zwischen Control Synchronization und Media Synchronization. Die aktive Synchronisation ist eine zeitliche Kontrollinformation; die Audio-Assets selbst werden weiterhin unabhängig lokal aufgezeichnet und anschließend als Media Synchronization übertragen.

---

# Nächste Entscheidung

Bevor Stufe 2 implementiert wird, muss eine eigene technische Entscheidung folgende Punkte anhand messbarer Daten festlegen:

1. Zeitreferenz und Clock-Modell
2. Messintervall und Synchronisationsnachrichten
3. Offset- und Drift-Schätzung
4. Korrekturstrategie ohne Änderung des Original-Audios
5. Toleranzgrenzen für akzeptable Abweichungen
6. Verhalten bei Netzwerkausfall und Rejoin
7. Testdesign für mindestens mehrstündige Sessions und unterschiedliche Geräte-Clocks

Bis diese Entscheidung getroffen und validiert ist, bleibt Stufe 1 der verbindliche Synchronisationsmechanismus.

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-063: Active Session Synchronisation

## Status

Accepted

## Date

2026-08-27

## Decision Type

Architecture

---

# Context

NC-PoRe records participants independently on different devices. ADR-009 establishes a sample-based internal time model. ADR-068 defines the explicit common recording start together with Opening and Closing Sync Signets as audio reference points.

A common start time alone is insufficient for long distributed recordings because independent device clocks may drift over the duration of a session. At the same time, network-based runtime synchronization is not required for the first productive recording stage and must not make local recording depend on network availability.

The architecture therefore uses a staged approach: the capture path already provides sufficient timing information for later reconstruction, while active runtime synchronization is added as a later layer if and when validated by measurements.

---

# Decision

NC-PoRe uses a **two-stage model** for synchronization of distributed recordings.

## Stage 1 — signet- and sample-based synchronization foundation

The current recording architecture uses:

* the sample-based track time model from ADR-009
* an explicit common recording start defined by ADR-068
* an Opening Sync Signet as a common audio reference point
* a Closing Sync Signet as a common end reference point
* participant-specific recording and capture metadata

This allows each local track to be recorded independently and later placed into a common temporal position using the same audio reference.

Local recording remains fully independent of network availability.

## Stage 2 — active runtime synchronization

A later stage may capture and distribute additional synchronization information during a running session to detect and compensate for clock drift automatically over long sessions.

This stage is **not part of the current capture implementation**. It will be added behind the existing transport- and storage-independent synchronization boundary and must never block local audio capture or require a permanent network connection.

Stage 2 must satisfy the following requirements:

* stable session time reference
* participant-specific timing information
* sample- or frame-level positioning
* detection and quantification of clock drift
* defined tolerance limits
* resume/rejoin after interruption
* sufficient accuracy for sessions lasting several hours
* independence from storage provider and production output format

The concrete protocol, clock estimation and drift correction strategy will be decided in a separate ADR once the required measurement and test data is available.

---

# Architectural Principle

> A distributed recording session has a common temporal reference; active runtime synchronization is an optional second layer, not part of the local capture foundation.

Capture data must already contain enough information in Stage 1 to reconstruct tracks deterministically and align them using the common Sync Signet.

---

# Binding Boundaries

The synchronization architecture must:

* never make local recording depend on the network
* never require a specific storage provider
* never require a final production format
* never equate a temporary connection loss with audio loss
* never require a new recording start merely because a participant rejoins

Active synchronization must specifically **not** become hidden real-time audio transport. NC-PoRe synchronizes the production and its temporal relationships, not the audio stream itself.

---

# Consequences

## Positive Effects

* The first productive stage remains robust and offline-capable.
* Opening and Closing Sync Signets provide real common reference points.
* The sample-based time model remains the stable data foundation.
* Later drift correction can be added without redesigning storage or capture.
* Rejoin and recovery can build on the same temporal metadata.

## Negative Effects

* Automatic clock-drift correction is initially outside the capture stage.
* Stage 2 must later be validated with real measurement data for long sessions.
* Active synchronization introduces additional protocol, state and test complexity.

---

# Alternatives Considered

## Common Start Time Only

Rejected. Independent device clocks may drift during long sessions.

## Permanent network-based runtime synchronization as a recording prerequisite

Rejected. Local audio capture must remain complete and independent during offline conditions.

## Manual alignment only, without synchronization metadata

Rejected. ADR-068 already provides a common audio reference point; capture data must additionally retain temporal identity and sample positions.

---

# Relationship to Existing Architecture

This ADR concretizes ADR-009 and explicitly scopes active runtime synchronization relative to the signet-based first stage already accepted by ADR-068.

Synchronization data remains independent of the separation between Control Synchronization and Media Synchronization established by ADR-030. Active synchronization is temporal control information; audio assets remain locally recorded and are subsequently transferred as Media Synchronization.

---

# Next Decision

Before Stage 2 is implemented, a separate technical decision must define the following based on measurable data:

1. time reference and clock model
2. measurement interval and synchronization messages
3. offset and drift estimation
4. correction strategy without modifying original audio
5. tolerance limits for acceptable deviation
6. behavior during network loss and rejoin
7. test design for at least multi-hour sessions and different device clocks

Until that decision is made and validated, Stage 1 remains the binding synchronization mechanism.
