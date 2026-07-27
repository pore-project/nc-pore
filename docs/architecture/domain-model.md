# NC-PoRe Domain Model

* Version: 1.0
* Date: 2026-07-24

---

# Deutsch ([English version below](#english-version))

---

# 1. Zweck dieses Dokuments

Dieses Dokument beschreibt das fachliche Modell von NC-PoRe.

Es definiert die zentralen Begriffe des Systems und ihre Beziehungen
zueinander.

Das Domain Model beschreibt nicht die technische Implementierung.

Es beschreibt die fachliche Realität, die durch NC-PoRe unterstützt wird.

Technische Entscheidungen wie Datenbanken, Programmiersprachen oder
Frameworks werden davon getrennt betrachtet.

---

# 2. Grundidee

NC-PoRe unterstützt Menschen bei der gemeinsamen Erstellung,
Verwaltung und Bewahrung von Medienproduktionen.

Die zentrale fachliche Einheit ist:

> Die Production Session.

Eine Production Session verbindet:

* Menschen
* Rollen
* Aufnahmen
* Medien
* Zustände
* Historie

zu einer gemeinsamen Produktion.

---

# 3. Fachliche Kernobjekte

---

# 3.1 Production Session

## Bedeutung

Eine Production Session ist die fachliche Klammer einer gemeinsamen Produktion.

Sie beschreibt einen zeitlich und organisatorisch zusammengehörenden
Produktionsprozess.

Beispiele:

* Podcast-Episode
* Interview
* Gespräch
* Remote-Aufnahme
* gemeinsames Medienprojekt

---

## Eigenschaften

Eine Production Session besitzt:

* eindeutige Identität
* Teilnehmer
* Rollen
* Aufnahmen
* Assets
* Status
* Aktivitätshistorie

---

## Verantwortung

Die Production Session ist die zentrale Einheit für:

* Zusammenarbeit
* Berechtigungen
* Produktionsstatus
* Nachvollziehbarkeit

---

# 3.2 Identity

## Bedeutung

Eine Identity beschreibt eine eindeutig erkennbare Person oder technische
Entität innerhalb des Systems.

Identität beantwortet:

> Wer handelt?

---

## Eigenschaften

Eine Identity kann besitzen:

* Authentifizierungsinformationen
* persönliche Informationen
* externe Verknüpfungen

---

## Abgrenzung

Identity ist nicht gleich:

* Rolle
* Berechtigung
* Teilnahme an einer Produktion

Diese Konzepte werden getrennt behandelt.

---

# 3.3 Participant

## Bedeutung

Ein Participant ist eine Identity, die an einer Production Session beteiligt ist.

Ein Participant beschreibt die konkrete Teilnahme an einer Produktion.

---

## Beispiel

Eine Person kann:

* in einer Session Moderator sein
* in einer anderen Session Gast sein
* in einer dritten Session nur Zuhörer sein

Die Identität bleibt gleich.

Die Rolle kann wechseln.

---

# 3.4 Role

## Bedeutung

Eine Role beschreibt die Verantwortung eines Participants innerhalb
einer Production Session.

---

## Beispiele

Mögliche Rollen:

* Host
* Producer
* Participant
* Guest
* Observer

---

## Grundprinzip

Rollen gehören zur Session.

Sie sind nicht fest mit einer Identity verbunden.

---

# 3.5 Recording

## Bedeutung

Ein Recording beschreibt einen Aufnahmevorgang innerhalb einer
Production Session.

Es beschreibt:

> Was wurde wann von wem aufgenommen?

---

## Eigenschaften

Ein Recording kann enthalten:

* Teilnehmer
* Zeitpunkt
* technische Aufnahmeinformationen
* Status
* zugehörige Assets

---

## Abgrenzung

Recording ist nicht gleich Datei.

Ein Recording beschreibt das fachliche Ereignis.

---

# 3.6 Asset

## Bedeutung

Ein Asset ist eine erzeugte oder verwaltete Ressource.

Beispiele:

* Audiodatei
* Exportdatei
* Metadaten
* Produktionsmaterial

---

## Grundprinzip

Assets sind die Ergebnisse oder Bestandteile einer Produktion.

Die fachliche Bedeutung entsteht durch ihre Verbindung
zu anderen Domain-Objekten.

---

# 3.7 Activity History

## Bedeutung

Die Activity History beschreibt nachvollziehbare Ereignisse innerhalb
einer Production Session.

Sie beantwortet:

> Was ist passiert?

---

## Beispiele

* Session erstellt
* Teilnehmer hinzugefügt
* Aufnahme gestartet
* Aufnahme abgeschlossen
* Rolle geändert
* Asset erzeugt

---

## Grundprinzip

Die Activity History ist das Produktionsgedächtnis von NC-PoRe.

---

# 3.8 Synchronization State

## Bedeutung

Der Synchronization State beschreibt den Zustand der Kommunikation
zwischen lokalen und zentralen Komponenten.

---

## Beispiele

* lokal erstellt
* übertragen
* bestätigt
* Konflikt erkannt

---

## Grundprinzip

Synchronisation ist ein Zustand der Produktion.

Sie verändert nicht die fachliche Bedeutung der Daten.

---

# 4. Beziehungen

Die wichtigsten Beziehungen:

```text
Identity

   |
   v

Participant

   |
   |
   v

Production Session

   |
   +----------------+
   |                |
   v                v

Recording        Activity History

   |
   v

Asset


Production Session

   |
   v

Synchronization State
```

---

# 5. Fachliche Regeln

## Regel 1

Eine Production Session ist ohne Teilnehmer nicht sinnvoll.

---

## Regel 2

Eine Identity kann an mehreren Production Sessions teilnehmen.

---

## Regel 3

Eine Role existiert nur innerhalb einer Production Session.

---

## Regel 4

Ein Recording gehört immer zu genau einer Production Session.

---

## Regel 5

Ein Asset kann nur über seinen fachlichen Zusammenhang interpretiert werden.

---

## Regel 6

Aktivitäten innerhalb einer Produktion sollen nachvollziehbar bleiben.

---

# 6. MVP-Relevanz

Das erste MVP benötigt folgende Domain-Objekte:

## Muss enthalten:

* Production Session
* Identity
* Participant
* Role
* Recording
* Asset
* grundlegende Activity History

---

## Später:

* komplexe Workflows
* umfangreiche Rechteverwaltung
* Versionierung
* Produktionsautomatisierung

---

# 7. Architekturprinzip

Das Domain Model bildet die fachliche Wahrheit von NC-PoRe.

Technische Komponenten müssen diese Wahrheit unterstützen.

Nicht umgekehrt.

Die Software wird um die Bedeutung der Produktion gebaut.

Nicht um die technischen Möglichkeiten einzelner Werkzeuge.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# 1. Purpose of this Document

This document describes the domain model of NC-PoRe.

It defines the central concepts of the system and their relationships.

The domain model does not describe technical implementation.

It describes the business reality supported by NC-PoRe.

Technical decisions such as databases, programming languages or
frameworks are considered separately.

---

# 2. Fundamental Idea

NC-PoRe supports people in creating, managing and preserving
media productions together.

The central domain entity is:

> The Production Session.

A Production Session connects:

* people
* roles
* recordings
* media
* states
* history

into a shared production.

---

# 3. Core Domain Objects

---

# 3.1 Production Session

## Meaning

A Production Session is the domain container of a shared production.

It describes a production process that belongs together in time
and organization.

Examples:

* podcast episode
* interview
* conversation
* remote recording
* collaborative media project

---

## Properties

A Production Session has:

* unique identity
* participants
* roles
* recordings
* assets
* status
* activity history

---

## Responsibility

The Production Session is the central unit for:

* collaboration
* permissions
* production status
* traceability

---

# 3.2 Identity

## Meaning

An Identity describes a uniquely recognizable person or technical
entity within the system.

Identity answers:

> Who is acting?

---

## Properties

An Identity may contain:

* authentication information
* personal information
* external references

---

## Separation

Identity is not equal to:

* role
* permission
* participation in a production

These concepts are intentionally separated.

---

# 3.3 Participant

## Meaning

A Participant is an Identity involved in a Production Session.

A Participant describes concrete participation in a production.

---

## Example

A person can be:

* host in one session
* guest in another session
* observer in a third session

The identity remains the same.

The role can change.

---

# 3.4 Role

## Meaning

A Role describes the responsibility of a Participant within
a Production Session.

---

## Examples

Possible roles:

* Host
* Producer
* Participant
* Guest
* Observer

---

## Principle

Roles belong to sessions.

They are not permanently attached to an Identity.

---

# 3.5 Recording

## Meaning

A Recording describes a recording process within a Production Session.

It describes:

> What was recorded, when and by whom?

---

## Properties

A Recording may contain:

* participant
* timestamp
* technical recording information
* status
* related assets

---

## Separation

A Recording is not the same as a file.

A Recording describes the domain event.

---

# 3.6 Asset

## Meaning

An Asset is a generated or managed resource.

Examples:

* audio file
* export file
* metadata
* production material

---

## Principle

Assets are results or components of a production.

Their meaning comes from their relationship
to other domain objects.

---

# 3.7 Activity History

## Meaning

Activity History describes traceable events within
a Production Session.

It answers:

> What happened?

---

## Examples

* session created
* participant added
* recording started
* recording completed
* role changed
* asset created

---

## Principle

Activity History is the production memory of NC-PoRe.

---

# 3.8 Synchronization State

## Meaning

Synchronization State describes the communication state
between local and central components.

---

## Examples

* created locally
* transferred
* confirmed
* conflict detected

---

## Principle

Synchronization is a production state.

It does not change the domain meaning of data.

---

# 4. Relationships

The main relationships:

```text
Identity

   |
   v

Participant

   |
   |
   v

Production Session

   |
   +----------------+
   |                |
   v                v

Recording      Activity History

   |
   v

Asset


Production Session

   |
   v

Synchronization State
```

---

# 5. Domain Rules

## Rule 1

A Production Session without participants has no meaningful purpose.

---

## Rule 2

An Identity can participate in multiple Production Sessions.

---

## Rule 3

A Role exists only within a Production Session.

---

## Rule 4

A Recording always belongs to exactly one Production Session.

---

## Rule 5

An Asset can only be interpreted through its domain context.

---

## Rule 6

Activities within a production should remain traceable.

---

# 6. MVP Relevance

The first MVP requires:

## Must contain:

* Production Session
* Identity
* Participant
* Role
* Recording
* Asset
* basic Activity History

---

## Later:

* complex workflows
* advanced permission management
* versioning
* production automation

---

# 7. Architecture Principle

The Domain Model represents the business truth of NC-PoRe.

Technical components must support this truth.

Not the other way around.

The software is built around the meaning of production.

Not around the capabilities of individual tools.
