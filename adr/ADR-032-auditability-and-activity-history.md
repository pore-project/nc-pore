# ADR-032: Auditability and Activity History

* Status: Accepted
* Date: 2026-07-23
* Decision Type: Architecture

---

# Deutsch (English version below)

## Kontext

NC-PoRe wurde als **Nextcloud Podcast Production Environment** definiert.

Die bisherigen Architekturentscheidungen legen fest:

* Die **Production Session** ist die zentrale fachliche Einheit.
* Menschen arbeiten gemeinsam an Produktionen.
* Rollen und Berechtigungen bestimmen Verantwortlichkeiten.
* Aufnahmen entstehen verteilt auf unterschiedlichen Clients.
* Assets werden synchronisiert und gemeinsam verarbeitet.
* Der Core ist die zentrale Instanz für fachliche Entscheidungen.

Mit mehreren Teilnehmern, mehreren Geräten und mehreren Verarbeitungsschritten entsteht eine neue Anforderung:

> Eine gemeinsame Produktion benötigt ein gemeinsames Gedächtnis.

Menschen müssen nachvollziehen können:

* was passiert ist
* wann etwas passiert ist
* wer eine Aktion durchgeführt hat
* welche Auswirkungen eine Änderung hatte

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

Jede relevante Aktion innerhalb einer Production Session kann ein Ereignis erzeugen.

Beispiele:

```text id="z3f7pm"
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

---

# Ereignismodell

Ein Aktivitätsereignis enthält mindestens:

```text id="q9j4xb"
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

```text id="5c8yq1"
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

```text id="7w3mka"
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

Interne Aktionen erzeugen Ereignisse, die:

* gespeichert
* verarbeitet
* angezeigt
* analysiert

werden können.

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

# Erweiterbarkeit

Die Architektur ermöglicht spätere Funktionen:

* Produktionschronik
* Änderungsvergleiche
* Wiederherstellung bestimmter Zustände
* automatische Zusammenfassungen
* Analyse von Produktionsabläufen

---

# Grundprinzipien

## 1. Eine Produktion braucht ein Gedächtnis

Gemeinsame Arbeit benötigt nachvollziehbare Geschichte.

---

## 2. Transparenz schafft Vertrauen

Teilnehmer können verstehen, was passiert ist.

---

## 3. Ereignisse sind wichtiger als Momentaufnahmen

Der Zustand allein erklärt nicht immer den Weg dorthin.

---

## 4. Der Mensch bleibt Mittelpunkt

Die Historie unterstützt Zusammenarbeit.

Sie ersetzt keine Kommunikation zwischen Menschen.

---

# Konsequenzen

## Vorteile

* bessere Nachvollziehbarkeit
* einfachere Fehleranalyse
* mehr Vertrauen bei Zusammenarbeit
* Grundlage für spätere Automatisierung

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
* dass Compliance-Funktionen bereits in V1 vollständig umgesetzt werden

---

# Leitgedanke

NC-PoRe unterstützt Menschen dabei, gemeinsam Produktionen zu erstellen.

Eine gute Zusammenarbeit benötigt nicht nur Werkzeuge.

Sie benötigt auch Erinnerung.

**Gemeinsam produzieren. Gemeinsam verstehen. Gemeinsam nachvollziehen.**

---

# English (Deutsche Version oben)

## Context

NC-PoRe has been defined as a **Nextcloud Podcast Production Environment**.

Previous architecture decisions established:

* The **Production Session** as the central domain entity.
* People collaborating on productions.
* Roles and permissions defining responsibilities.
* Distributed recordings across different clients.
* Assets synchronized and processed together.
* The Core as the authority for domain decisions.

With multiple participants, devices and processing steps, a new requirement appears:

> A shared production needs a shared memory.

People need to understand:

* what happened
* when it happened
* who performed an action
* what impact a change had

---

# Decision

NC-PoRe introduces a traceable activity history for Production Sessions.

The purpose is:

* transparency
* collaboration
* troubleshooting
* traceability

Guiding principle:

> Auditability supports collaboration, not surveillance.

---

# Activity History

Relevant actions create activity events.

Examples:

```text id="p4m9vx"
SessionCreated

ParticipantInvited

RecordingStarted

AssetUploaded

AssetReplaced

MetadataChanged

ExportCreated
```

---

# Event Model

An activity event contains:

```text id="r6k1pt"
Activity Event

├── Event ID
├── Timestamp
├── Actor Identity
├── Action Type
├── Target Object
├── Session ID
└── Result
```

---

# Consequences

Benefits:

* better traceability
* easier troubleshooting
* increased trust
* foundation for future automation

Costs:

* additional storage
* additional complexity
* event definitions require care

These costs are consciously accepted.

---

# Guiding Principle

NC-PoRe helps people create productions together.

Good collaboration needs not only tools.

It also needs memory.

**Produce together. Understand together. Trace together.**
