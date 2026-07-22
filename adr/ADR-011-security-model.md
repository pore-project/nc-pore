# ADR-011: Security Model

## Status

Accepted

## Date

2026-07-22

---

# Context

NC-PoRe verarbeitet persönliche Audiodaten.

Aufnahmen können enthalten:

- persönliche Gespräche
- vertrauliche Informationen
- nicht veröffentlichte Inhalte
- Produktionsmaterial

Daher muss Sicherheit ein grundlegender Bestandteil der
Architektur sein.

NC-PoRe verfolgt das Prinzip:

> Zugriff erfolgt nur, wenn er ausdrücklich erlaubt ist.

---

# Decision

NC-PoRe verwendet ein mehrschichtiges Sicherheitsmodell.

Die Sicherheitsarchitektur besteht aus:

- Authentifizierung
- Rollen und Berechtigungen
- Session-basierter Zugriffskontrolle
- sicherer Datenübertragung
- nachvollziehbaren Aktionen

---

# Authentication

Benutzer müssen eindeutig identifiziert werden.

Die Authentifizierung erfolgt über die vorhandene
Plattformintegration.

Beispiel:

- Nextcloud Benutzerkonto
- externe Gastidentität über Einladung

---

# Authorization

Der Zugriff auf Daten erfolgt über Berechtigungen.

Grundprinzip:

```
Wer bin ich?

+

Was darf ich?

=

Zugriff
```

---

# Principle of Least Privilege

NC-PoRe folgt dem Prinzip:

> Jeder Benutzer erhält nur die Rechte, die für seine Aufgabe
> notwendig sind.

Beispiele:

Administrator:

- Systemverwaltung

Moderator:

- Sessionverwaltung

Editor:

- Produktionszugriff

Gast:

- eigene Aufnahme innerhalb einer Session

---

# Session Security

Jede Session besitzt einen eigenen Sicherheitskontext.

Eine Session definiert:

- Teilnehmer
- Rollen
- Berechtigungen
- verfügbare Daten

Ein Gast darf nur auf die Session zugreifen,
zu der er eingeladen wurde.

---

# Audio Data Protection

Audiodateien werden nicht öffentlich abgelegt.

Zugriff erfolgt ausschließlich über:

- authentifizierte Benutzer
- gültige Berechtigungen
- definierte Freigaben

---

# Upload Security

Uploads müssen:

- authentifiziert erfolgen
- Integrität prüfen
- unvollständige Dateien erkennen
- Wiederaufnahme ermöglichen

Beispiel:

```
Upload

↓

Prüfsumme

↓

Validierung

↓

Archivierung
```

---

# Auditability

Sicherheitsrelevante Aktionen sollen nachvollziehbar sein.

Beispiele:

- Session erstellt
- Gast eingeladen
- Aufnahme hochgeladen
- Datei exportiert
- Berechtigung geändert

---

# Privacy Principles

NC-PoRe folgt folgenden Datenschutzprinzipien:

- Datenminimierung
- Transparenz
- Zweckbindung
- Kontrolle durch den Benutzer

---

# Consequences

## Positive Auswirkungen

- klare Sicherheitsstruktur
- geeignet für professionelle Produktionen
- sichere Zusammenarbeit mit Gästen
- nachvollziehbare Zugriffe

---

## Negative Auswirkungen

- zusätzliche technische Komplexität
- Rechteverwaltung erforderlich
- mehr Entwicklungsaufwand

---

# Alternatives considered

## Vollständiger Zugriff innerhalb eines Projektes

Verworfen.

Grund:

Nicht jeder Teilnehmer benötigt Zugriff auf alle Daten.

---

## Öffentliche Dateifreigaben

Verworfen.

Grund:

Nicht vereinbar mit dem Datenschutzprinzip von NC-PoRe.

---

# Notes

Sicherheit ist kein Zusatzfeature.

Die Sicherheit der Daten ist Bestandteil der
Grundarchitektur von NC-PoRe.
```