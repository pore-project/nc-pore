# NC-PoRe Domain Rules

* Version: 1.0
* Date: 2026-07-24

---

# Deutsch ([English version below](#english-version))

---

# 1. Zweck dieses Dokuments

Dieses Dokument beschreibt die fachlichen Regeln von NC-PoRe.

Es definiert, welche Zustände und Veränderungen innerhalb des Systems
fachlich gültig sind.

Die Regeln beschreiben nicht die technische Umsetzung.

Sie sind unabhängig von:

* Programmiersprachen
* Datenbanken
* Benutzeroberflächen
* Speichertechnologien

Die technische Umsetzung muss diese Regeln unterstützen.

---

# 2. Grundprinzip

NC-PoRe bildet reale Produktionsprozesse ab.

Daher gelten folgende Grundsätze:

* Fachliche Bedeutung steht vor technischer Struktur.
* Zustände müssen nachvollziehbar sein.
* Veränderungen müssen erklärbar bleiben.
* Zusammenarbeit benötigt klare Verantwortlichkeiten.

---

# 3. Production Session Regeln

## Regel 3.1

Eine Production Session ist die zentrale fachliche Einheit einer Produktion.

Alle produktionsbezogenen Vorgänge müssen einer Production Session
zugeordnet werden können.

---

## Regel 3.2

Eine Production Session besitzt einen eigenen Lebenszyklus.

Beispiel:

```text
Created
  |
  v
Preparing
  |
  v
Recording
  |
  v
Completed
  |
  v
Archived
```

Die konkreten Zustände werden später technisch definiert.

---

## Regel 3.3

Eine abgeschlossene Production Session darf nicht unkontrolliert verändert
werden.

Änderungen müssen nachvollziehbar bleiben.

---

# 4. Identity und Participant Regeln

## Regel 4.1

Eine Identity beschreibt, wer eine Handlung ausführt.

Eine Identity ist nicht automatisch Teilnehmer einer Produktion.

---

## Regel 4.2

Ein Participant entsteht durch die Teilnahme einer Identity an einer
Production Session.

---

## Regel 4.3

Eine Identity kann gleichzeitig an mehreren Production Sessions beteiligt
sein.

Die Rolle und Verantwortung kann pro Session unterschiedlich sein.

---

# 5. Rollen Regeln

## Regel 5.1

Rollen existieren immer innerhalb einer Production Session.

---

## Regel 5.2

Eine Rolle beschreibt Verantwortung, nicht technische Berechtigung allein.

Eine technische Berechtigung muss aus fachlichen Rollen abgeleitet werden.

---

## Regel 5.3

Änderungen an Rollen müssen nachvollziehbar dokumentiert werden.

---

# 6. Recording Regeln

## Regel 6.1

Ein Recording gehört immer genau zu einer Production Session.

---

## Regel 6.2

Ein Recording beschreibt einen fachlichen Aufnahmevorgang.

Es ist nicht identisch mit einer Datei.

---

## Regel 6.3

Ein abgeschlossenes Recording gilt als unveränderliches Ergebnis.

Änderungen erzeugen neue Zustände oder neue Versionen.

---

# 7. Asset Regeln

## Regel 7.1

Ein Asset erhält seine Bedeutung durch seinen Zusammenhang mit einer
Production Session.

---

## Regel 7.2

Ein Asset darf nicht unabhängig von seiner Herkunft betrachtet werden.

---

## Regel 7.3

Die Erzeugung und Veränderung von Assets muss nachvollziehbar bleiben.

---

# 8. Activity History Regeln

## Regel 8.1

Wichtige fachliche Veränderungen müssen in der Activity History
nachvollziehbar sein.

---

## Regel 8.2

Die Activity History beschreibt Ereignisse.

Sie ist keine technische Debug-Ausgabe.

---

## Regel 8.3

Historische Ereignisse dürfen nicht nachträglich verfälscht werden.

---

# 9. Synchronisationsregeln

## Regel 9.1

Synchronisation verändert nicht die fachliche Bedeutung von Daten.

---

## Regel 9.2

Lokale und zentrale Zustände müssen unterscheidbar bleiben.

---

## Regel 9.3

Konflikte müssen erkannt und nachvollziehbar behandelt werden.

---

# 10. Datenhoheit

## Regel 10.1

Die erzeugten Medien gehören den Menschen und Organisationen,
die sie erstellen.

---

## Regel 10.2

NC-PoRe darf keine unnötige Abhängigkeit von einzelnen Plattformen
erzeugen.

---

## Regel 10.3

Speicherung und fachliche Bedeutung bleiben getrennt.

---

# 11. MVP-relevante Regeln

Das erste MVP muss folgende Regeln unterstützen:

* Production Sessions besitzen klare Zustände
* Teilnehmer und Rollen sind getrennt
* Aufnahmen sind Sessions zugeordnet
* erzeugte Assets bleiben nachvollziehbar
* wichtige Aktionen werden dokumentiert
* Synchronisationszustände sind sichtbar

---

# 12. Architekturprinzip

Die Domain Rules definieren die Grenzen, die der Core schützen muss.

Der Core ist verantwortlich dafür, dass fachlich ungültige Zustände
nicht entstehen.

Technische Komponenten dürfen diese Regeln unterstützen.

Sie definieren sie nicht.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# 1. Purpose of this Document

This document describes the domain rules of NC-PoRe.

It defines which states and changes are valid from a domain perspective.

The rules do not describe technical implementation.

They are independent of:

* programming languages
* databases
* user interfaces
* storage technologies

Technical implementation must support these rules.

---

# 2. Fundamental Principle

NC-PoRe represents real production workflows.

Therefore:

* domain meaning comes before technical structure
* states must remain traceable
* changes must remain explainable
* collaboration requires clear responsibilities

---

# 3. Production Session Rules

## Rule 3.1

A Production Session is the central domain unit of a production.

All production-related activities must be assignable to a Production Session.

---

## Rule 3.2

A Production Session has its own lifecycle.

Example:

```text
Created
  |
  v
Preparing
  |
  v
Recording
  |
  v
Completed
  |
  v
Archived
```

The exact states will be defined technically later.

---

## Rule 3.3

A completed Production Session must not be changed without control.

Changes must remain traceable.

---

# 4. Identity and Participant Rules

## Rule 4.1

An Identity describes who performs an action.

An Identity is not automatically a participant in a production.

---

## Rule 4.2

A Participant is created through an Identity participating in a
Production Session.

---

## Rule 4.3

An Identity can participate in multiple Production Sessions.

Role and responsibility may differ per session.

---

# 5. Role Rules

## Rule 5.1

Roles always exist within a Production Session.

---

## Rule 5.2

A Role describes responsibility, not only technical permission.

Technical permissions must be derived from domain roles.

---

## Rule 5.3

Changes to roles must remain traceable.

---

# 6. Recording Rules

## Rule 6.1

A Recording always belongs to exactly one Production Session.

---

## Rule 6.2

A Recording describes a domain recording event.

It is not identical to a file.

---

## Rule 6.3

A completed Recording is treated as an immutable result.

Changes create new states or versions.

---

# 7. Asset Rules

## Rule 7.1

An Asset receives meaning through its relationship with a Production Session.

---

## Rule 7.2

An Asset must not be considered independently from its origin.

---

## Rule 7.3

Creation and modification of Assets must remain traceable.

---

# 8. Activity History Rules

## Rule 8.1

Important domain changes must remain traceable in Activity History.

---

## Rule 8.2

Activity History describes events.

It is not technical debugging output.

---

## Rule 8.3

Historical events must not be modified retrospectively.

---

# 9. Synchronization Rules

## Rule 9.1

Synchronization does not change the domain meaning of data.

---

## Rule 9.2

Local and central states must remain distinguishable.

---

## Rule 9.3

Conflicts must be detected and handled transparently.

---

# 10. Data Ownership

## Rule 10.1

Created media belongs to the people and organizations creating it.

---

## Rule 10.2

NC-PoRe must not create unnecessary dependency on individual platforms.

---

## Rule 10.3

Storage and domain meaning remain separated.

---

# 11. MVP Relevant Rules

The first MVP must support:

* Production Sessions with clear states
* separation of participants and roles
* recordings assigned to sessions
* traceable assets
* documented important actions
* visible synchronization states

---

# 12. Architecture Principle

Domain Rules define the boundaries the Core must protect.

The Core is responsible for preventing invalid domain states.

Technical components may support these rules.

They do not define them.
