# ADR-032: Auditability and Activity History

* Status: Accepted
* Date: 2026-07-23
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

## Kontext

NC-PoRe wurde als **Nextcloud Podcast Production Environment** definiert.

Die bisherigen Architekturentscheidungen legen fest:

* Die **Production Session** ist die zentrale fachliche Einheit.
* Menschen arbeiten gemeinsam an Produktionen.
* Rollen und Berechtigungen bestimmen Verantwortlichkeiten.
* Aufnahmen entstehen verteilt auf unterschiedlichen Clients.
* Assets werden synchronisiert und gemeinsam verarbeitet.
* Der Core ist die zentrale Instanz für fachliche Entscheidungen.

Mit mehreren Teilnehmern, Geräten und Verarbeitungsschritten muss nachvollziehbar bleiben, was in einer gemeinsamen Produktion geschehen ist.

---

# Entscheidung

NC-PoRe führt eine nachvollziehbare Aktivitätsgeschichte für Production Sessions ein.

Diese Aktivitätshistorie dient:

* Transparenz
* Zusammenarbeit
* Fehlersuche
* Nachvollziehbarkeit

Leitprinzip:

> Auditability dient der Zusammenarbeit, nicht der Überwachung.

---

# Activity History

Relevante Aktionen innerhalb einer Production Session können ein Ereignis erzeugen.

Beispiele:

```text
SessionCreated

ParticipantInvited

ParticipantJoined

RecordingStarted

RecordingFinished

AssetUploaded

AssetReplaced

MetadataChanged

ExportCreated
```

Die konkrete Auswahl und Semantik der Ereignisse richtet sich nach der jeweiligen fachlichen Funktion.

---

# Ereignismodell

Ein Aktivitätsereignis enthält mindestens:

```text
Activity Event

├── Event ID
├── Timestamp
├── Actor Identity
├── Action Type
├── Target Object
├── Session ID
└── Result
```

Damit kann nachvollzogen werden:

* wer gehandelt hat
* wann gehandelt wurde
* was betroffen war
* welches Ergebnis entstanden ist

---

# Beziehung zur Production Session

Aktivitäten gehören zur fachlichen Einheit der Produktion.

Nicht:

> Benutzerhistorie

sondern:

> Produktionshistorie

Eine Session besitzt dadurch ihr eigenes Gedächtnis.

Beispiel:

```text
Production Session

├── Participants
├── Assets
├── Recordings
├── Exports
└── Activity History
```

---

# Beziehung zu Identity und Roles

ADR-031 definiert Identität und Rollen.

ADR-032 nutzt diese Informationen.

Ein Ereignis beantwortet:

```text
Wer?

        ↓

hat was?

        ↓

wann?

        ↓

in welcher Session?

        ↓

verändert?
```

Ohne Identität ist eine Historie nicht sinnvoll.

---

# Beziehung zur API

ADR-028 definiert Events als Bestandteil der Architektur.

Activity History nutzt dieses Prinzip.

Interne Aktionen erzeugen Ereignisse, die gespeichert, verarbeitet oder angezeigt werden können.

---

# V1 Strategie

Für V1 wird eine einfache, zuverlässige Historie umgesetzt.

Gespeichert werden:

* Zeitpunkt
* Benutzer
* Aktion
* betroffenes Objekt
* Ergebnis

Nicht Bestandteil von V1:

* unveränderbare Blockchain-basierte Historie
* vollständige Versionsverwaltung aller Daten
* komplexe Compliance-Systeme

---

# Transparenz statt Kontrolle

NC-PoRe verwendet Activity History nicht als Überwachungssystem.

Ziel ist:

* gemeinsame Orientierung
* bessere Zusammenarbeit
* schnellere Fehleranalyse

Beispiel:

Nicht:

> Warum hat Benutzer X Datei Y verändert?

Sondern:

> Welche Schritte hat diese Produktion genommen?

---

# Grundprinzipien

## 1. Eine Produktion braucht ein Gedächtnis

Gemeinsame Arbeit benötigt nachvollziehbare Geschichte.

## 2. Transparenz schafft Vertrauen

Teilnehmer können verstehen, was passiert ist.

## 3. Ereignisse sind wichtiger als Momentaufnahmen

Der Zustand allein erklärt nicht immer den Weg dorthin.

## 4. Der Mensch bleibt Mittelpunkt

Die Historie unterstützt Zusammenarbeit. Sie ersetzt keine Kommunikation zwischen Menschen.

---

# Konsequenzen

## Vorteile

* bessere Nachvollziehbarkeit
* einfachere Fehleranalyse
* mehr Vertrauen bei Zusammenarbeit
* strukturierte Grundlage für Ereignisverarbeitung

## Nachteile

* zusätzlicher Speicherbedarf
* zusätzliche Systemkomplexität
* Ereignisse müssen sinnvoll definiert werden

Diese Nachteile werden bewusst akzeptiert.

---

# Nicht-Ziele

Diese Entscheidung bedeutet nicht:

* dass jede Benutzeraktion protokolliert werden muss
* dass NC-PoRe ein Überwachungssystem wird
* dass jede Dateiänderung automatisch versioniert wird
* dass ein bestimmtes Compliance-System vorgeschrieben wird

---

# Leitgedanke

NC-PoRe unterstützt Menschen dabei, gemeinsam Produktionen zu erstellen.

Eine gute Zusammenarbeit benötigt nicht nur Werkzeuge. Sie benötigt auch Erinnerung.

**Gemeinsam produzieren. Gemeinsam verstehen. Gemeinsam nachvollziehen.**

---

# English Version ([Deutsche Version oben](#deutsch))

## Context

NC-PoRe has been defined as a **Nextcloud Podcast Production Environment**.

Previous architecture decisions established:

* The **Production Session** as the central domain entity.
* People collaborating on productions.
* Roles and permissions defining responsibilities.
* Distributed recordings across different clients.
* Assets synchronized and processed together.
* The Core as the authority for domain decisions.

With multiple participants, devices and processing steps, the history of a shared production must remain traceable.

---

# Decision

NC-PoRe introduces a traceable activity history for Production Sessions.

The activity history serves:

* transparency
* collaboration
* troubleshooting
* traceability

Guiding principle:

> Auditability supports collaboration, not surveillance.

---

# Activity History

Relevant actions within a Production Session may create an activity event.

Examples:

```text
SessionCreated

ParticipantInvited

ParticipantJoined

RecordingStarted

RecordingFinished

AssetUploaded

AssetReplaced

MetadataChanged

ExportCreated
```

The concrete selection and semantics of events depend on the respective domain function.

---

# Event Model

An activity event contains at least:

```text
Activity Event

├── Event ID
├── Timestamp
├── Actor Identity
├── Action Type
├── Target Object
├── Session ID
└── Result
```

This makes it possible to determine:

* who acted
* when the action occurred
* what was affected
* what result was produced

---

# Relationship to the Production Session

Activities belong to the domain entity of the production.

Not:

> User history

But:

> Production history

A session therefore has its own memory.

Example:

```text
Production Session

├── Participants
├── Assets
├── Recordings
├── Exports
└── Activity History
```

---

# Relationship to Identity and Roles

ADR-031 defines identity and roles.

ADR-032 uses this information.

An event answers:

```text
Who?

        ↓

Did what?

        ↓

When?

        ↓

In which session?

        ↓

Changed what?
```

Without identity, an activity history is not meaningful.

---

# Relationship to the API

ADR-028 defines events as part of the architecture.

Activity History uses this principle.

Internal actions create events that can be stored, processed or displayed.

---

# V1 Strategy

V1 uses a simple and reliable activity history.

Stored information includes:

* timestamp
* user
* action
* affected object
* result

Not part of V1:

* immutable blockchain-based history
* complete versioning of all data
* complex compliance systems

---

# Transparency Instead of Control

NC-PoRe does not use Activity History as a surveillance system.

The goals are:

* shared orientation
* better collaboration
* faster troubleshooting

Example:

Not:

> Why did user X change file Y?

But:

> What steps did this production take?

---

# Core Principles

## 1. A Production Needs a Memory

Collaborative work requires a traceable history.

## 2. Transparency Builds Trust

Participants can understand what happened.

## 3. Events Matter More Than Snapshots

State alone does not always explain how it was reached.

## 4. People Remain Central

History supports collaboration. It does not replace communication between people.

---

# Consequences

## Advantages

* better traceability
* easier troubleshooting
* greater trust in collaboration
* structured basis for event processing

## Disadvantages

* additional storage
* additional system complexity
* event definitions require care

These disadvantages are consciously accepted.

---

# Non-Goals

This decision does not mean:

* that every user action must be logged
* that NC-PoRe becomes a surveillance system
* that every file change is automatically versioned
* that a specific compliance system is mandated

---

# Guiding Principle

NC-PoRe helps people create productions together.

Good collaboration needs not only tools. It also needs memory.

**Produce together. Understand together. Trace together.**
