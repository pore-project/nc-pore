# ADR-006: Role-Based Access Control (RBAC)

## Status

Accepted

## Date

2026-07-22

---

# Context

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

# Decision

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

# Consequences

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

# Alternatives considered

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

# Notes

Das Rollenmodell soll erweiterbar bleiben.

Spätere Rollen können sein:

- Archivmanager
- Veröffentlichungsmanager
- Transkriptionseditor
- Produktionsleiter

Neue Rollen dürfen das Grundprinzip nicht verändern.