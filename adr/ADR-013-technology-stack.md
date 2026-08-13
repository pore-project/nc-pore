# Deutsch ([English version below](#english-version))

# ADR-013: Technology Stack

## Status

Accepted

## Date

2026-07-22

---

# Kontext

NC-PoRe besteht aus mehreren technisch unterschiedlichen
Bereichen.

Die Plattform benötigt:

- einen lokalen Audio-Recorder
- eine Serverintegration
- Benutzer- und Rechteverwaltung
- Metadatenverwaltung
- offene Schnittstellen

Die Komponenten sollen unabhängig voneinander
weiterentwickelt werden können.

---

# Entscheidung

NC-PoRe wird als modulare Architektur mit getrennten
Komponenten umgesetzt.

Die Hauptkomponenten sind:

```
NC-PoRe

├── Recorder Client
│
├── Nextcloud Application
│
├── Backend Services
│
└── Export Layer
```

---

# Recorder Client

## Entscheidung

Der Recorder wird als eigenständige Anwendung entwickelt.

Begründung:

- direkter Hardwarezugriff
- stabile Audioverarbeitung
- unabhängig vom Browser
- bessere Kontrolle über Ressourcen
- professionelle Einsatzmöglichkeiten

---

# Recorder Technology

Für den Recorder wird eine native oder
native-nahe Technologie bevorzugt.

Bewertungskriterien:

- Audioqualität
- Stabilität
- Plattformunterstützung
- FOSS-Eignung
- langfristige Wartbarkeit

Mögliche Technologien:

- Rust
- C++
- Qt-basierte Anwendungen
- andere geeignete FOSS-Technologien

Die endgültige Auswahl erfolgt nach Prototyping.

---

# Server Integration

## Entscheidung

Die Serverintegration erfolgt als
Nextcloud-Anwendung.

Aufgaben:

- Projekte
- Sessions
- Benutzer
- Rollen
- Metadaten
- Dateiverwaltung

Die Nextcloud-App ist nicht für die primäre
Audioaufnahme verantwortlich.

---

# Communication

Recorder und Server kommunizieren über definierte
Schnittstellen.

Beispiele:

- Session-Erzeugung
- Authentifizierung
- Upload
- Statusinformationen
- Metadaten

---

# Database and Storage

Die Speicherung orientiert sich an den bestehenden
Nextcloud-Mechanismen.

NC-PoRe nutzt:

- offene Datenstrukturen
- dokumentierte Formate
- nachvollziehbare Metadaten

---

# Open Source Principles

Der Technologiestack soll unterstützen:

- freie Entwicklungswerkzeuge
- offene Standards
- Community-Beiträge
- langfristige Wartbarkeit

---

# Konsequenzen

## Positive Auswirkungen

- klare Trennung der Verantwortlichkeiten
- professionelle Audioarchitektur möglich
- bessere Erweiterbarkeit
- geringere Abhängigkeiten

---

## Negative Auswirkungen

- mehrere Komponenten müssen gepflegt werden
- höherer initialer Entwicklungsaufwand
- Schnittstellen müssen sauber definiert werden

---

# Betrachtete Alternativen

## Alles als Nextcloud-App

Verworfen.

Grund:

Nextcloud ist nicht für Echtzeit-Audiohardware
optimiert.

---

## Alles als Desktop-Anwendung

Verworfen.

Grund:

Benutzerverwaltung und Zusammenarbeit wären
unnötig kompliziert.

---

## Browser als alleiniger Recorder

Verworfen als Hauptlösung.

Grund:

Nicht ausreichend kontrollierbar für professionelle
Aufnahmen.

---

# Hinweise

Die Architektur folgt dem Prinzip:

> Das richtige Werkzeug für die richtige Aufgabe.

Der Recorder macht Audio.
Nextcloud macht Zusammenarbeit.

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-013: Technology Stack

## Status

Accepted

## Date

2026-07-22

---

# Context

NC-PoRe consists of several technically different areas.

The platform requires:

- a local audio recorder
- server integration
- user and permission management
- metadata management
- open interfaces

The components should be able to evolve independently.

---

# Decision

NC-PoRe is implemented as a modular architecture with separate components.

The main components are:

```
NC-PoRe

├── Recorder Client
│
├── Nextcloud Application
│
├── Backend Services
│
└── Export Layer
```

---

# Recorder Client

## Decision

The recorder is developed as a standalone application.

Reasons:

- direct hardware access
- stable audio processing
- independent of the browser
- better resource control
- professional use cases

---

# Recorder Technology

A native or native-like technology is preferred for the recorder.

Evaluation criteria:

- audio quality
- stability
- platform support
- FOSS suitability
- long-term maintainability

Possible technologies:

- Rust
- C++
- Qt-based applications
- other suitable FOSS technologies

The final selection will be made after prototyping.

---

# Server Integration

## Decision

Server integration is implemented as a Nextcloud application.

Responsibilities:

- projects
- sessions
- users
- roles
- metadata
- file management

The Nextcloud app is not responsible for primary audio recording.

---

# Communication

Recorder and server communicate through defined interfaces.

Examples:

- session creation
- authentication
- upload
- status information
- metadata

---

# Database and Storage

Storage follows the existing Nextcloud mechanisms.

NC-PoRe uses:

- open data structures
- documented formats
- traceable metadata

---

# Open Source Principles

The technology stack should support:

- free development tools
- open standards
- community contributions
- long-term maintainability

---

# Consequences

## Positive Effects

- clear separation of responsibilities
- professional audio architecture possible
- better extensibility
- fewer dependencies

---

## Negative Effects

- multiple components must be maintained
- higher initial development effort
- interfaces must be defined carefully

---

# Alternatives Considered

## Everything as a Nextcloud App

Rejected.

Reason:

Nextcloud is not optimized for real-time audio hardware.

---

## Everything as a Desktop Application

Rejected.

Reason:

User management and collaboration would be unnecessarily complicated.

---

## Browser as the Sole Recorder

Rejected as the primary solution.

Reason:

Not sufficiently controllable for professional recordings.

---

# Notes

The architecture follows the principle:

> The right tool for the right task.

The recorder handles audio.
Nextcloud handles collaboration.
