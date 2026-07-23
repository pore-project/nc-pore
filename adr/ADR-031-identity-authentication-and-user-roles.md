# ADR-031: Identity, Authentication and User Roles

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
* Clients können auf unterschiedlichen Plattformen betrieben werden.
* Aufnahmen werden verteilt erzeugt und anschließend synchronisiert.
* Der Core bleibt die zentrale Instanz für fachliche Entscheidungen.

Mit zunehmender Funktionalität entsteht die Notwendigkeit, Identität, Authentifizierung und Berechtigungen sauber zu definieren.

Dabei muss zwischen drei unterschiedlichen Konzepten unterschieden werden:

```text id="p8q1md"
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

Benutzerrechte werden nicht ausschließlich aus der Existenz eines Benutzers abgeleitet.

Sie entstehen aus der Rolle einer Person innerhalb einer Production Session.

Leitprinzip:

> NC-PoRe verwaltet nicht nur Dateien und Aufnahmen. NC-PoRe unterstützt Zusammenarbeit zwischen Menschen.

---

# Identity

Eine Identität beschreibt eine Person oder ein technisches Subjekt innerhalb des Systems.

Die Identität beantwortet:

> Wer handelt?

Die Identität ist unabhängig davon, welche Rolle eine Person in einer bestimmten Production Session besitzt.

Eine Person kann beispielsweise:

* Besitzer einer eigenen Session sein
* Producer in einer anderen Session sein
* Gast in einer dritten Session sein

---

# Authentication

Authentication beschreibt den Nachweis einer Identität.

Beispiele:

* Anmeldung über Nextcloud
* sichere Token-basierte Kommunikation
* zukünftige alternative Identity Provider

NC-PoRe V1 verwendet Nextcloud als primären Identity Provider.

Die Architektur bleibt jedoch offen für zukünftige Erweiterungen.

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

NC-PoRe verwendet zunächst ein bewusst einfaches rollenbasiertes Modell.

V1 Rollen:

```text id="x7b2kw"
Owner

↓

Producer

↓

Participant

↓

Guest
```

---

# Owner

Der Owner besitzt die zentrale Verantwortung für eine Production Session.

Berechtigungen:

* Production Session erstellen
* Teilnehmer einladen
* Rollen vergeben
* Session verwalten
* finale Kontrolle über die Produktion behalten

Beispiel:

Podcast-Verantwortlicher.

---

# Producer

Der Producer unterstützt die operative Durchführung einer Produktion.

Berechtigungen:

* Aufnahme koordinieren
* Teilnehmer verwalten
* Assets organisieren
* Produktionsabläufe begleiten

Beispiel:

Co-Host oder Redakteur.

---

# Participant

Der Participant ist aktiver Teil einer Produktion.

Berechtigungen:

* an Sessions teilnehmen
* eigene Aufnahmen bereitstellen
* eigene Assets verwalten

Beispiel:

Podcast-Gast.

---

# Guest

Der Guest nimmt eingeschränkt an einer Produktion teil.

Berechtigungen:

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

Die Architektur trennt jedoch:

```text id="u4x8aa"
Nextcloud Identity

        ↓

NC-PoRe Identity Layer

        ↓

Production Roles
```

Dadurch bleibt NC-PoRe erweiterbar für:

* weitere Identity Provider
* mobile Clients
* externe Teilnehmer
* Enterprise-Umgebungen

---

# Einladung und Teilnahme

Ein typischer Ablauf:

```text id="1q8vzk"
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

Das Rollenmodell ist bewusst erweiterbar.

Spätere Rollen können beispielsweise sein:

* Editor
* Sound Engineer
* Publisher
* Viewer

Neue Rollen dürfen das Grundprinzip nicht verändern:

> Rechte entstehen aus Verantwortung innerhalb einer Produktion.

---

# Nicht-Ziele

Diese Entscheidung bedeutet nicht:

* dass V1 ein komplexes Enterprise-Rechtesystem benötigt
* dass jede mögliche Rolle sofort definiert wird
* dass Benutzer außerhalb von Sessions identische Rechte besitzen
* dass Nextcloud die komplette Geschäftslogik übernimmt

---

# Konsequenzen

## Vorteile

* klare Verantwortlichkeiten
* sichere Zusammenarbeit
* einfache Erweiterbarkeit
* gute Trennung von Identität und Rolle
* passende Grundlage für mehrere Clients

## Nachteile

* zusätzliche Komplexität gegenüber einem einfachen Benutzer/Datei-Modell
* Rollen müssen gepflegt werden
* Berechtigungsprüfungen benötigen zentrale Logik

Diese Nachteile werden bewusst akzeptiert.

---

# Leitgedanke

NC-PoRe ist eine Umgebung für Menschen, die gemeinsam produzieren.

Technik, Dateien und Aufnahmen dienen diesem Ziel.

Die Architektur orientiert sich deshalb nicht nur an Daten, sondern an Zusammenarbeit.

---

# English (Deutsche Version oben)

## Context

NC-PoRe has been defined as a **Nextcloud Podcast Production Environment**.

Previous architecture decisions established:

* The **Production Session** as the central domain entity.
* People collaborate on productions.
* Clients operate on different platforms.
* Recordings are created in a distributed way and synchronized afterwards.
* The Core remains the authority for domain decisions.

As functionality grows, identity, authentication and authorization need clear definitions.

These concepts must be separated:

```text id="r9w3nf"
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

Permissions are derived from a person's role inside a Production Session.

Guiding principle:

> NC-PoRe does not only manage files and recordings. NC-PoRe supports collaboration between people.

---

# Role Model V1

NC-PoRe starts with a deliberately simple role model:

```text id="3kq8vx"
Owner

↓

Producer

↓

Participant

↓

Guest
```

---

# Nextcloud Integration

Nextcloud is the primary Identity Provider for V1.

The architecture keeps a separate NC-PoRe identity layer:

```text id="8m1qpd"
Nextcloud Identity

        ↓

NC-PoRe Identity Layer

        ↓

Production Roles
```

---

# Security Principles

NC-PoRe follows least privilege principles.

Every action requires a meaningful authorization.

The Core validates:

* identity
* role
* allowed action

---

# Consequences

Benefits:

* clear responsibilities
* secure collaboration
* extensibility
* separation of identity and role

Costs:

* additional complexity
* role maintenance
* centralized authorization logic

These costs are consciously accepted.

---

# Guiding Principle

NC-PoRe is an environment for people who create productions together.

Technology, files and recordings exist to support collaboration.
