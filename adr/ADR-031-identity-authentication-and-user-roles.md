# ADR-031: Identity, Authentication and User Roles

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
* Clients können auf unterschiedlichen Plattformen betrieben werden.
* Aufnahmen werden verteilt erzeugt und anschließend synchronisiert.
* Der Core bleibt die zentrale Instanz für fachliche Entscheidungen.

Damit Identität und Berechtigungen nachvollziehbar bleiben, muss zwischen drei Konzepten unterschieden werden:

```text
Identity

Wer bin ich?

Authentication

Wie beweise ich meine Identität?

Authorization

Was darf ich tun?
```

---

# Entscheidung

NC-PoRe trennt grundsätzlich zwischen:

1. Identität
2. Authentifizierung
3. Autorisierung

Benutzerrechte werden nicht ausschließlich aus der Existenz eines Benutzers abgeleitet. Sie entstehen aus der Rolle einer Person innerhalb einer Production Session.

Leitprinzip:

> NC-PoRe verwaltet nicht nur Dateien und Aufnahmen. NC-PoRe unterstützt Zusammenarbeit zwischen Menschen.

---

# Identity

Eine Identität beschreibt eine Person oder ein technisches Subjekt innerhalb des Systems.

Die Identität beantwortet:

> Wer handelt?

Die Identität ist unabhängig davon, welche Rolle eine Person in einer bestimmten Production Session besitzt.

---

# Authentication

Authentication beschreibt den Nachweis einer Identität.

NC-PoRe V1 verwendet Nextcloud als primären Identity Provider.

Die konkrete Authentifizierungsimplementierung bleibt von der fachlichen Rollenlogik getrennt.

---

# Authorization

Authorization beschreibt, welche Aktionen eine Identität innerhalb eines bestimmten Kontextes durchführen darf.

Entscheidungen über Berechtigungen werden vom Core getroffen.

Nicht:

> Der Client entscheidet, was erlaubt ist.

Sondern:

> Der Core prüft Identität, Rolle und erlaubte Aktion.

---

# Rollenmodell V1

NC-PoRe verwendet zunächst ein bewusst einfaches rollenbasiertes Modell:

```text
Owner

↓

Producer

↓

Participant

↓

Guest
```

Die Rollen beschreiben Verantwortlichkeiten innerhalb einer Production Session.

---

# Owner

Der Owner besitzt die zentrale Verantwortung für eine Production Session.

Beispielhafte Berechtigungen:

* Production Session erstellen
* Teilnehmer einladen
* Rollen vergeben
* Session verwalten
* finale Kontrolle über die Produktion behalten

---

# Producer

Der Producer unterstützt die operative Durchführung einer Produktion.

Beispielhafte Berechtigungen:

* Aufnahme koordinieren
* Teilnehmer verwalten
* Assets organisieren
* Produktionsabläufe begleiten

---

# Participant

Der Participant ist aktiver Teil einer Produktion.

Beispielhafte Berechtigungen:

* an Sessions teilnehmen
* eigene Aufnahmen bereitstellen
* eigene Assets verwalten

---

# Guest

Der Guest nimmt eingeschränkt an einer Produktion teil.

Beispielhafte Berechtigungen:

* Einladung annehmen
* teilnehmen
* eigene Daten liefern

Keine Berechtigungen:

* Produktion verändern
* andere Teilnehmer verwalten
* zentrale Einstellungen ändern

---

# Nextcloud Integration

Nextcloud ist in V1 der primäre Identity Provider.

Die Architektur trennt:

```text
Nextcloud Identity

        ↓

NC-PoRe Identity Layer

        ↓

Production Roles
```

Damit bleibt die fachliche Rollenlogik unabhängig von der konkreten Identity-Implementierung.

---

# Einladung und Teilnahme

Ein typischer Ablauf:

```text
Owner erstellt Session

        ↓

Teilnehmer wird eingeladen

        ↓

Teilnehmer akzeptiert

        ↓

Rolle wird zugewiesen

        ↓

Teilnahme möglich
```

---

# Security Prinzipien

NC-PoRe verfolgt das Prinzip minimaler Berechtigungen.

Grundsatz:

> Jede Aktion benötigt eine nachvollziehbare Berechtigung.

Dabei gilt:

* keine impliziten Vollzugriffe
* Rollen statt Einzel-Sonderrechte
* Prüfung im Core
* nachvollziehbare Entscheidungen

---

# Erweiterbarkeit des Rollenmodells

Das Rollenmodell bleibt erweiterbar.

Neue Rollen dürfen das Grundprinzip nicht verändern:

> Rechte entstehen aus Verantwortung innerhalb einer Produktion.

Welche weiteren Rollen erforderlich sind, wird durch konkrete fachliche Anforderungen entschieden.

---

# Nicht-Ziele

Diese Entscheidung bedeutet nicht:

* dass V1 ein komplexes Enterprise-Rechtesystem benötigt
* dass jede mögliche Rolle bereits definiert wird
* dass Benutzer außerhalb von Sessions identische Rechte besitzen
* dass Nextcloud die komplette Geschäftslogik übernimmt

---

# Konsequenzen

## Vorteile

* klare Verantwortlichkeiten
* sichere Zusammenarbeit
* Erweiterbarkeit
* gute Trennung von Identität und Rolle
* zentrale und nachvollziehbare Berechtigungsentscheidungen

## Nachteile

* zusätzliche Komplexität gegenüber einem einfachen Benutzer/Datei-Modell
* Rollen müssen gepflegt werden
* Berechtigungsprüfungen benötigen zentrale Logik

Diese Nachteile werden bewusst akzeptiert.

---

# Leitgedanke

NC-PoRe ist eine Umgebung für Menschen, die gemeinsam Produktionen erstellen.

Technik, Dateien und Aufnahmen dienen diesem Ziel.

Die Architektur orientiert sich deshalb nicht nur an Daten, sondern an Zusammenarbeit.

---

# English Version ([Deutsche Version oben](#deutsch))

## Context

NC-PoRe has been defined as a **Nextcloud Podcast Production Environment**.

Previous architecture decisions established:

* The **Production Session** is the central domain entity.
* People collaborate on productions.
* Clients may operate on different platforms.
* Recordings are created in a distributed way and synchronized afterwards.
* The Core remains the authority for domain decisions.

To keep identity and permissions traceable, three concepts must be separated:

```text
Identity

Who am I?

Authentication

How do I prove my identity?

Authorization

What am I allowed to do?
```

---

# Decision

NC-PoRe separates:

1. Identity
2. Authentication
3. Authorization

Permissions are not derived solely from the existence of a user. They arise from the person's role within a Production Session.

Guiding principle:

> NC-PoRe does not only manage files and recordings. NC-PoRe supports collaboration between people.

---

# Identity

An identity represents a person or technical subject within the system.

It answers:

> Who is acting?

Identity is independent of the role held by a person in a particular Production Session.

---

# Authentication

Authentication proves an identity.

NC-PoRe V1 uses Nextcloud as the primary Identity Provider.

The concrete authentication implementation remains separate from domain role logic.

---

# Authorization

Authorization defines which actions an identity may perform in a given context.

Authorization decisions are made by the Core.

Not:

> The client decides what is allowed.

But:

> The Core validates identity, role and permitted action.

---

# Role Model V1

NC-PoRe initially uses a deliberately simple role-based model:

```text
Owner

↓

Producer

↓

Participant

↓

Guest
```

Roles describe responsibilities within a Production Session.

---

# Owner

The Owner has primary responsibility for a Production Session.

Example responsibilities:

* create a Production Session
* invite participants
* assign roles
* manage the session
* retain final control over the production

---

# Producer

The Producer supports operational execution of a production.

Example responsibilities:

* coordinate recording
* manage participants
* organize assets
* support production workflows

---

# Participant

A Participant is an active part of a production.

Example responsibilities:

* participate in sessions
* provide own recordings
* manage own assets

---

# Guest

A Guest participates with limited permissions.

Example permissions:

* accept an invitation
* participate
* provide own data

The Guest cannot:

* modify the production
* manage other participants
* change central settings

---

# Nextcloud Integration

Nextcloud is the primary Identity Provider for V1.

The architecture separates:

```text
Nextcloud Identity

        ↓

NC-PoRe Identity Layer

        ↓

Production Roles
```

This keeps domain role logic independent from the concrete identity implementation.

---

# Invitation and Participation

A typical flow is:

```text
Owner creates session

        ↓

Participant is invited

        ↓

Participant accepts

        ↓

Role is assigned

        ↓

Participation is allowed
```

---

# Security Principles

NC-PoRe follows least-privilege principles.

Guiding rule:

> Every action requires traceable authorization.

This includes:

* no implicit full access
* roles instead of ad-hoc special permissions
* validation in the Core
* traceable decisions

---

# Role Model Extensibility

The role model remains extensible.

New roles must not change the underlying principle:

> Permissions arise from responsibility within a production.

Which additional roles are required is decided by concrete domain requirements.

---

# Non-Goals

This decision does not mean:

* that V1 requires a complex enterprise permission system
* that every possible role is already defined
* that users have identical rights outside sessions
* that Nextcloud owns all business logic

---

# Consequences

## Advantages

* clear responsibilities
* secure collaboration
* extensibility
* clear separation of identity and role
* centralized and traceable authorization decisions

## Disadvantages

* additional complexity compared with a simple user/file model
* roles require maintenance
* authorization requires central logic

These disadvantages are consciously accepted.

---

# Guiding Principle

NC-PoRe is an environment for people who create productions together.

Technology, files and recordings serve this goal.

The architecture therefore focuses not only on data, but also on collaboration.
