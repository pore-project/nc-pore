# Deutsch ([English version below](#english-version))

# ADR-006: Role-Based Access Control (RBAC)

## Status

Accepted

## Date

2026-07-22

---

# Kontext

NC-PoRe wird von unterschiedlichen Personengruppen genutzt.

Ein professioneller Podcast-Workflow benötigt unterschiedliche
Verantwortlichkeiten:

- technische Administration
- Produktionsleitung
- Moderation
- Teilnehmer
- externe Gäste

Eine einfache Unterscheidung zwischen "Benutzer" und
"Administrator" reicht nicht aus.

NC-PoRe benötigt ein transparentes und erweiterbares
Berechtigungsmodell.

---

# Entscheidung

NC-PoRe verwendet ein rollenbasiertes Berechtigungsmodell
(Role-Based Access Control, RBAC).

Berechtigungen werden nicht direkt einzelnen Personen,
sondern Rollen zugeordnet.

Benutzer erhalten eine oder mehrere Rollen.

---

# Rollenmodell

## Administrator

Verantwortung:

System- und Serververwaltung.

Berechtigungen:

- globale Konfiguration
- Benutzerverwaltung
- Rollenverwaltung
- technische Wartung
- Zugriff auf Systemprotokolle

---

## Moderator

Verantwortung:

Verwaltung von Podcast-Sessions.

Berechtigungen:

- Projekte erstellen
- Sessions erstellen
- Teilnehmer einladen
- Gäste verwalten
- Aufnahmen verwalten
- Produktionsstatus ändern

---

## Benutzer

Verantwortung:

Reguläre Teilnahme an Produktionen.

Berechtigungen:

- eigene Sessions sehen
- eigene Aufnahme durchführen
- eigene Daten verwalten

---

## Editor

Verantwortung:

Nachbearbeitung und Produktion.

Berechtigungen:

- Zugriff auf freigegebene Rohspuren
- Export vorbereiten
- Produktionsdateien verwalten

---

## Gast

Verantwortung:

Externe Teilnahme an einer einzelnen Session.

Berechtigungen:

- Session betreten
- Aufnahme bestätigen
- eigene Audiospur erzeugen
- Upload durchführen

Keine Berechtigung:

- andere Teilnehmerdaten sehen
- Projekte verwalten
- historische Sessions öffnen

---

# Berechtigungsprinzipien

NC-PoRe folgt dem Prinzip:

> So wenig Rechte wie möglich, so viele Rechte wie notwendig.

(Lowest Privilege Principle)

---

# Sessionbezogene Rechte

Berechtigungen können zusätzlich auf Session-Ebene
eingeschränkt werden.

Beispiel:
Projekt A

Moderator:
volle Verwaltung

Editor:
Zugriff auf Audio

Gast:
nur aktuelle Session

---

# Konsequenzen

## Positive Auswirkungen

- klare Verantwortlichkeiten
- sichere Gastteilnahme
- Erweiterbarkeit
- bessere Nachvollziehbarkeit
- geeignet für Teams

---

## Negative Auswirkungen

- höhere Komplexität
- Rechteverwaltung muss sauber umgesetzt werden
- zusätzliche Benutzeroberfläche erforderlich

---

# Betrachtete Alternativen

## Alle Benutzer gleich behandeln

Verworfen.

Gründe:

- ungeeignet für professionelle Produktionen
- keine sichere Zusammenarbeit mit Gästen

---

## Rechte direkt pro Benutzer vergeben

Verworfen.

Gründe:

- schwer wartbar
- nicht skalierbar
- widerspricht etablierten Sicherheitsmodellen

---

# Hinweise

Das Rollenmodell soll erweiterbar bleiben.

Spätere Rollen können sein:

- Archivmanager
- Veröffentlichungsmanager
- Transkriptionseditor
- Produktionsleiter

Neue Rollen dürfen das Grundprinzip nicht verändern.

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-006: Role-Based Access Control (RBAC)

## Status

Accepted

## Date

2026-07-22

---

# Context

NC-PoRe is used by different groups of people.

A professional podcast workflow requires different responsibilities:

- technical administration
- production management
- moderation
- participants
- external guests

A simple distinction between "user" and "administrator" is not sufficient.

NC-PoRe requires a transparent and extensible authorization model.

---

# Decision

NC-PoRe uses a role-based access control model (Role-Based Access Control, RBAC).

Permissions are assigned to roles rather than directly to individual people.

Users may have one or more roles.

---

# Role Model

## Administrator

Responsibility:

System and server administration.

Permissions:

- global configuration
- user management
- role management
- technical maintenance
- access to system logs

---

## Moderator

Responsibility:

Management of podcast sessions.

Permissions:

- create projects
- create sessions
- invite participants
- manage guests
- manage recordings
- change production status

---

## User

Responsibility:

Regular participation in productions.

Permissions:

- view own sessions
- make own recordings
- manage own data

---

## Editor

Responsibility:

Post-production and production work.

Permissions:

- access released raw tracks
- prepare exports
- manage production files

---

## Guest

Responsibility:

External participation in a single session.

Permissions:

- enter a session
- confirm recording
- create own audio track
- perform upload

No permission to:

- view other participants' data
- manage projects
- open historical sessions

---

# Permission Principles

NC-PoRe follows the principle:

> As few permissions as possible, as many permissions as necessary.

(Lowest Privilege Principle)

---

# Session-Specific Permissions

Permissions may additionally be restricted at session level.

Example:
Project A

Moderator:
full administration

Editor:
access to audio

Guest:
current session only

---

# Consequences

## Positive Effects

- clear responsibilities
- secure guest participation
- extensibility
- better traceability
- suitable for teams

---

## Negative Effects

- higher complexity
- permission management must be implemented carefully
- additional user interface required

---

# Alternatives Considered

## Treat All Users Equally

Rejected.

Reasons:

- unsuitable for professional productions
- no secure collaboration with guests

---

## Assign Permissions Directly per User

Rejected.

Reasons:

- difficult to maintain
- not scalable
- contradicts established security models

---

# Notes

The role model should remain extensible.

Possible future roles include:

- archive manager
- publication manager
- transcription editor
- production manager

New roles must not change the fundamental principle.
