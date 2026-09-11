# ADR-008: Client Architecture

* Status: Accepted
* Date: 2026-07-22
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

## Kontext

NC-PoRe benötigt eine zuverlässige lokale Audioaufnahme.

Die Aufnahmequalität darf nicht von einer Serververbindung oder einer für Kommunikation optimierten Media-Pipeline abhängen.

Gleichzeitig soll NC-PoRe unterschiedlichen Nutzungssituationen Rechnung tragen, insbesondere professioneller Aufnahme und einfacher Teilnahme.

---

# Entscheidung

NC-PoRe verwendet eine modulare Client-Architektur.

Die lokale Aufnahme erfolgt durch einen spezialisierten Capture-/Recorder-Pfad.

Der Recorder bzw. Capture-Client ist für insbesondere folgende Aufgaben verantwortlich:

* Zugriff auf Audiohardware bzw. lokale Capture-Quellen
* lokale Aufnahme
* technische Aufnahmedaten und Chunks
* Metadaten-Erzeugung
* lokale Verarbeitung und Persistenz im Rahmen der Capture-Architektur
* Übergabe an den bestehenden Recording-/Artifact-Pfad

Der Server übernimmt keine primäre Audioaufnahme.

Client- und Capture-Varianten dürfen sich hinsichtlich Bedienung und technischer Möglichkeiten unterscheiden. Sie müssen jedoch denselben fachlichen Recording- und Artifact-Grenzen folgen.

---

# Client-Varianten

Die Architektur unterstützt unterschiedliche Client-Varianten für unterschiedliche Nutzungssituationen.

Ein Client für professionelle Aufnahme kann umfangreichere lokale Capture-Fähigkeiten bereitstellen. Ein Client für externe oder gelegentliche Teilnehmer kann die Teilnahme vereinfachen.

Die konkrete technische Form einer Client-Variante ist nicht Bestandteil dieser ADR.

---

# Architekturmodell

```text
                Host / Session Environment
                         |
                         |
                  Session Context
                         |
             +-----------+-----------+
             |                       |
        Capture Client          Simple Client
             |                       |
             +-----------+-----------+
                         |
                   Local Capture
                         |
                  PoRE Recording
                         |
                   Artifact Path
```

Die fachliche Recording-Logik bleibt unabhängig von der konkreten Client-Form.

---

# Browser-basierte Teilnahme

Eine browserbasierte Teilnahme kann die Einstiegshürde reduzieren.

Die konkrete Browser-Architektur wird durch die dafür zuständigen Architekturentscheidungen definiert. Sie darf die grundlegende Trennung zwischen lokaler Aufnahme, Recording-Modell und Host-Kommunikation nicht aufheben.

---

# Konsequenzen

## Positive Auswirkungen

* professionelle lokale Aufnahme bleibt möglich
* Aufnahme und Server bleiben klar getrennt
* unterschiedliche Nutzungssituationen können unterstützt werden
* Client-Implementierungen bleiben austauschbar
* der Core bleibt unabhängig von einer konkreten Client-Technologie

## Negative Auswirkungen

* Client- und Capture-Grenzen müssen gepflegt werden
* lokale Capture-Implementierungen können technisch anspruchsvoll sein
* unterschiedliche Client-Varianten benötigen eigene Tests

Diese Nachteile werden bewusst akzeptiert.

---

# Betrachtete Alternativen

## Ausschließliche Web-App

Nicht als allgemeine Architekturvorgabe gewählt. Eine reine Web-App darf die Anforderungen an lokale Aufnahmequalität und technische Capture-Kontrolle nicht als selbstverständlich voraussetzen.

## Ausschließlicher spezialisierter Client

Nicht als allgemeine Architekturvorgabe gewählt. Unterschiedliche Nutzungssituationen können einen einfacheren Teilnahmeweg erfordern.

---

# Beziehung zu bestehender Architektur

Diese ADR definiert die allgemeine Client-Grenze. Konkrete Entscheidungen zu Plattformen, Browsern und Host-Integrationen werden in den jeweils zuständigen ADRs getroffen.

---

# Leitgedanke

> Professionelle Aufnahme dort, wo sie benötigt wird; einfache Teilnahme dort, wo sie genügt — ohne die fachlichen Recording-Grenzen zu vermischen.

---

# English Version ([Deutsche Version oben](#deutsch))

## Context

NC-PoRe requires reliable local audio recording.

Recording quality must not depend on a server connection or on a media pipeline optimized for communication.

At the same time, NC-PoRe must support different usage situations, in particular professional recording and simple participation.

---

# Decision

NC-PoRe uses a modular client architecture.

Local recording is performed through a specialized capture/recorder path.

The recorder or capture client is responsible in particular for:

* access to audio hardware or local capture sources
* local recording
* technical recording data and chunks
* metadata generation
* local processing and persistence within the capture architecture
* handoff to the existing recording/artifact path

The server does not perform the primary audio recording.

Client and capture variants may differ in user experience and technical capabilities. They must nevertheless follow the same domain Recording and Artifact boundaries.

---

# Client Variants

The architecture supports different client variants for different usage situations.

A client for professional recording may provide more extensive local capture capabilities. A client for external or occasional participants may simplify participation.

The concrete technical form of a client variant is outside the scope of this ADR.

---

# Architecture Model

```text
                Host / Session Environment
                         |
                         |
                  Session Context
                         |
             +-----------+-----------+
             |                       |
        Capture Client          Simple Client
             |                       |
             +-----------+-----------+
                         |
                   Local Capture
                         |
                  PoRE Recording
                         |
                   Artifact Path
```

Domain Recording logic remains independent from the concrete client form.

---

# Browser-Based Participation

Browser-based participation can reduce the entry barrier.

The concrete browser architecture is defined by the responsible architecture decisions. It must not remove the fundamental separation between local recording, the Recording model and host communication.

---

# Consequences

## Positive Effects

* professional local recording remains possible
* recording and server remain clearly separated
* different usage situations can be supported
* client implementations remain replaceable
* the Core remains independent from a concrete client technology

## Negative Effects

* client and capture boundaries require maintenance
* local capture implementations may be technically demanding
* different client variants require their own tests

These disadvantages are consciously accepted.

---

# Alternatives Considered

## Web App Only

Not selected as a general architectural rule. A pure web app must not be assumed to satisfy local recording quality and capture-control requirements in all environments.

## Specialized Client Only

Not selected as a general architectural rule. Different usage situations may require a simpler participation path.

---

# Relationship to Existing Architecture

This ADR defines the general client boundary. Concrete decisions about platforms, browsers and host integrations are made in the respective ADRs.

---

# Guiding Principle

> Professional recording where it is needed; simple participation where it is sufficient — without mixing the domain Recording boundaries.
