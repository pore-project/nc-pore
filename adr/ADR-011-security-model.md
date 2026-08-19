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

# Benchmark-Ergebnis: Payload-Integrität

Die für die Payload-Integritätsprüfung verwendete SHA-256-Berechnung wurde für repräsentative Payload-Größen von 64 KiB, 1 MiB, 10 MiB und 64 MiB mit Criterion gemessen.

Der Benchmark vergleicht jeweils:

- `copy_only`
- `hash_only`
- `copy_and_hash`

Die Messung auf dem GitHub Actions Runner mit Rust 1.97.1 ergab unter anderem:

| Payload | `copy_only` | `hash_only` | `copy_and_hash` |
| --- | ---: | ---: | ---: |
| 64 KiB | 1.524 µs | 46.653 µs | 48.149 µs |
| 1 MiB | 28.054 µs | 745.70 µs | 774.86 µs |
| 10 MiB | 313.41 µs | 7.4635 ms | 7.8044 ms |
| 64 MiB | 6.8307 ms | 47.762 ms | 54.856 ms |

Die Ergebnisse zeigen, dass der zusätzliche Aufwand von `copy_and_hash` gegenüber `hash_only` relativ klein bleibt. Bei 10 MiB beträgt er rund 0,34 ms, bei 64 MiB rund 7,1 ms. Der Hashing-Schritt selbst dominiert damit die zusätzliche Laufzeit; das Kopieren stellt keinen vergleichbaren zusätzlichen Engpass dar.

Für den vorgesehenen Aufnahme- und Artefaktpfad ist dieser Overhead in der gemessenen Größenordnung akzeptabel. Eine weitere Optimierung der Hash-Berechnung wird auf Grundlage dieser Messung derzeit nicht verfolgt.

Der Benchmark bleibt als reproduzierbare Performance-Messung im Repository erhalten. Er dient der Erkennung zukünftiger Performance-Regressionen. Es wird bewusst kein harter Laufzeitgrenzwert in der normalen CI definiert, da die absolute Laufzeit von der jeweiligen CI-Ausführungsumgebung abhängt.

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

---

# Benchmark Result: Payload Integrity

The SHA-256 calculation used for payload integrity was benchmarked with Criterion for representative payload sizes of 64 KiB, 1 MiB, 10 MiB, and 64 MiB.

The benchmark compares:

- `copy_only`
- `hash_only`
- `copy_and_hash`

The measurement on the GitHub Actions runner using Rust 1.97.1 produced, among others:

| Payload | `copy_only` | `hash_only` | `copy_and_hash` |
| --- | ---: | ---: | ---: |
| 64 KiB | 1.524 µs | 46.653 µs | 48.149 µs |
| 1 MiB | 28.054 µs | 745.70 µs | 774.86 µs |
| 10 MiB | 313.41 µs | 7.4635 ms | 7.8044 ms |
| 64 MiB | 6.8307 ms | 47.762 ms | 54.856 ms |

The results show that the additional cost of `copy_and_hash` compared with `hash_only` remains relatively small. At 10 MiB it is about 0.34 ms, and at 64 MiB about 7.1 ms. Hashing itself therefore dominates the additional runtime; copying does not represent a comparable additional bottleneck.

For the intended recording and artifact path, this overhead is acceptable at the measured scale. No further optimization of the hashing operation is currently pursued based on these measurements.

The benchmark remains in the repository as a reproducible performance measurement and serves to detect future performance regressions. No hard runtime threshold is deliberately defined in normal CI because absolute runtime depends on the CI execution environment.
