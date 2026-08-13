# Deutsch ([English version below](#english-version))

# ADR-011: Security Model

## Status

Accepted

## Date

2026-07-22

---

# Kontext

NC-PoRe verarbeitet persönliche Audiodaten.

Aufnahmen können enthalten:

- persönliche Gespräche
- vertrauliche Informationen
- nicht veröffentlichte Inhalte
- Produktionsmaterial

Daher muss Sicherheit ein grundlegender Bestandteil der Architektur sein.

NC-PoRe verfolgt das Prinzip:

> Zugriff erfolgt nur, wenn er ausdrücklich erlaubt ist.

---

# Entscheidung

NC-PoRe verwendet ein mehrschichtiges Sicherheitsmodell.

Die Sicherheitsarchitektur besteht aus:

- Authentifizierung
- Rollen und Berechtigungen
- Session-basierter Zugriffskontrolle
- sicherer Datenübertragung
- nachvollziehbaren Aktionen

---

# Authentifizierung

Benutzer müssen eindeutig identifiziert werden.

Die Authentifizierung erfolgt über die vorhandene Plattformintegration.

Beispiel:

- Nextcloud Benutzerkonto
- externe Gastidentität über Einladung

---

# Autorisierung

Der Zugriff auf Daten erfolgt über Berechtigungen.

Grundprinzip:

```text
Wer bin ich?

+

Was darf ich?

=

Zugriff
```

---

# Prinzip der geringsten Berechtigung

NC-PoRe folgt dem Prinzip:

> Jeder Benutzer erhält nur die Rechte, die für seine Aufgabe notwendig sind.

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

# Session-Sicherheit

Jede Session besitzt einen eigenen Sicherheitskontext.

Eine Session definiert:

- Teilnehmer
- Rollen
- Berechtigungen
- verfügbare Daten

Ein Gast darf nur auf die Session zugreifen, zu der er eingeladen wurde.

---

# Schutz der Audiodaten

Audiodateien werden nicht öffentlich abgelegt.

Zugriff erfolgt ausschließlich über:

- authentifizierte Benutzer
- gültige Berechtigungen
- definierte Freigaben

---

# Upload-Sicherheit

Uploads müssen:

- authentifiziert erfolgen
- Integrität prüfen
- unvollständige Dateien erkennen
- Wiederaufnahme ermöglichen

Beispiel:

```text
Upload

↓

Prüfsumme

↓

Validierung

↓

Archivierung
```

---

# Nachvollziehbarkeit

Sicherheitsrelevante Aktionen sollen nachvollziehbar sein.

Beispiele:

- Session erstellt
- Gast eingeladen
- Aufnahme hochgeladen
- Datei exportiert
- Berechtigung geändert

---

# Datenschutzprinzipien

NC-PoRe folgt folgenden Datenschutzprinzipien:

- Datenminimierung
- Transparenz
- Zweckbindung
- Kontrolle durch den Benutzer

---

# Konsequenzen

## Positive Auswirkungen

- klare Sicherheitsstruktur
- geeignet für professionelle Produktionen
- sichere Zusammenarbeit mit Gästen
- nachvollziehbare Zugriffe

## Negative Auswirkungen

- zusätzliche technische Komplexität
- Rechteverwaltung erforderlich
- mehr Entwicklungsaufwand

---

# Betrachtete Alternativen

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

# Hinweise

Sicherheit ist kein Zusatzfeature.

Die Sicherheit der Daten ist Bestandteil der Grundarchitektur von NC-PoRe.

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-011: Security Model

## Status

Accepted

## Date

2026-07-22

---

# Context

NC-PoRe processes personal audio data.

Recordings may contain:

- personal conversations
- confidential information
- unpublished content
- production material

Security must therefore be a fundamental part of the architecture.

NC-PoRe follows the principle:

> Access is granted only when it is explicitly permitted.

---

# Decision

NC-PoRe uses a layered security model.

The security architecture consists of:

- authentication
- roles and permissions
- session-based access control
- secure data transfer
- auditable actions

---

# Authentication

Users must be uniquely identified.

Authentication is provided through the existing platform integration.

Examples:

- Nextcloud user account
- external guest identity through invitation

---

# Authorization

Access to data is controlled through permissions.

Basic principle:

```text
Who am I?

+

What am I allowed to do?

=

Access
```

---

# Principle of Least Privilege

NC-PoRe follows the principle:

> Each user receives only the permissions necessary for their task.

Examples:

Administrator:

- system administration

Moderator:

- session management

Editor:

- production access

Guest:

- own recording within a session

---

# Session Security

Each session has its own security context.

A session defines:

- participants
- roles
- permissions
- available data

A guest may access only the session to which they were invited.

---

# Audio Data Protection

Audio files are not stored publicly.

Access is provided exclusively through:

- authenticated users
- valid permissions
- defined shares

---

# Upload Security

Uploads must:

- be authenticated
- verify integrity
- detect incomplete files
- support resumption

Example:

```text
Upload

↓

Checksum

↓

Validation

↓

Archiving
```

---

# Auditability

Security-relevant actions should be auditable.

Examples:

- session created
- guest invited
- recording uploaded
- file exported
- permission changed

---

# Privacy Principles

NC-PoRe follows these privacy principles:

- data minimization
- transparency
- purpose limitation
- user control

---

# Consequences

## Positive Effects

- clear security structure
- suitable for professional productions
- secure collaboration with guests
- auditable access

## Negative Effects

- additional technical complexity
- permission management required
- increased development effort

---

# Alternatives Considered

## Full Access Within a Project

Rejected.

Reason:

Not every participant needs access to all data.

---

## Public File Shares

Rejected.

Reason:

Not compatible with NC-PoRe's privacy principle.

---

# Notes

Security is not an additional feature.

Data security is part of the fundamental architecture of NC-PoRe.
