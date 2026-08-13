# Deutsch ([English version below](#english-version))

# ADR-014: Development Environment and Toolchain

## Status

Accepted

## Date

2026-07-22

---

# Kontext

NC-PoRe besteht aus mehreren technischen Komponenten,
die unterschiedliche Anforderungen besitzen.

Besonders der Recorder-Client stellt hohe Anforderungen
an:

- Audioqualität
- Stabilität
- Echtzeitverarbeitung
- Hardwarezugriff
- Plattformunterstützung

Die Entwicklungsumgebung muss daher langfristig
wartbar und für ein Open-Source-Projekt geeignet sein.

---

# Entscheidung

NC-PoRe verwendet eine komponentenbezogene
Technologieauswahl.

Nicht alle Bestandteile müssen mit derselben
Programmiersprache entwickelt werden.

Die Architektur trennt:

```
NC-PoRe

├── Recorder Client
│
├── Nextcloud Application
│
├── Backend Integration
│
└── Export Components
```

---

# Recorder Development

## Entscheidung

Der Recorder wird mit einer nativen oder
native-nahen Technologie entwickelt.

Bewertungskriterien:

- Performance
- Stabilität
- Speicher- und Ressourcenkontrolle
- Audio-Unterstützung
- Plattformfähigkeit
- FOSS-Kompatibilität

---

# Candidate Technologies

## Rust

Vorteile:

- moderne Systemsprache
- hohe Speichersicherheit
- gute Performance
- starke Open-Source-Community

Nachteile:

- Audio-Ökosystem weniger etabliert als C++

---

## C++ / Qt

Vorteile:

- langjährige Erfahrung im Audiobereich
- umfangreiche Bibliotheken
- professionelle Anwendungen nutzen diesen Stack

Nachteile:

- höhere Komplexität
- mehr Verantwortung für Speicherverwaltung

---

## Python

Verwendung:

- Prototyping
- technische Experimente
- Tests

Nicht vorgesehen als endgültige Recorder-Basis.

---

# Decision Principle

Die endgültige Auswahl des Recorder-Stacks erfolgt
nach einem technischen Prototyp.

Der Prototyp muss zeigen:

- Mikrofonzugriff
- WAV-Aufnahme
- stabile Langzeitaufnahme
- Chunk-Verarbeitung

---

# Nextcloud Development

Die Nextcloud-Komponente folgt dem bestehenden
Nextcloud-Ökosystem.

Verantwortlichkeiten:

- Benutzerverwaltung
- Sessions
- Rollen
- Metadaten
- Dateiverwaltung

Die Nextcloud-App ist nicht für die
Audioaufnahme verantwortlich.

---

# Development Tools

NC-PoRe verwendet bevorzugt:

- Git zur Versionsverwaltung
- offene Entwicklungswerkzeuge
- automatisierte Tests
- dokumentierte Build-Prozesse

---

# Build Philosophy

Build-Prozesse sollen:

- reproduzierbar
- dokumentiert
- automatisierbar

sein.

---

# Konsequenzen

## Positive Auswirkungen

- passende Werkzeuge für jede Aufgabe
- professionelle Audioarchitektur möglich
- langfristige Wartbarkeit
- bessere Erweiterbarkeit

---

## Negative Auswirkungen

- mehrere Technologien müssen gepflegt werden
- höherer initialer Aufwand
- Entwickler müssen mehrere Bereiche verstehen

---

# Betrachtete Alternativen

## Eine Sprache für alle Komponenten

Verworfen.

Grund:

Die Anforderungen von Recorder und Webverwaltung
unterscheiden sich deutlich.

---

## Webtechnologie für Recorder

Verworfen als Hauptansatz.

Grund:

Nicht ausreichend für professionelle
Audiohardware-Kontrolle.

---

# Hinweise

NC-PoRe folgt dem Prinzip:

> Die Architektur bestimmt die Werkzeuge,
> nicht umgekehrt.

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-014: Development Environment and Toolchain

## Status

Accepted

## Date

2026-07-22

---

# Context

NC-PoRe consists of several technical components with different requirements.

The recorder client in particular has demanding requirements for:

- audio quality
- stability
- real-time processing
- hardware access
- platform support

The development environment therefore needs to be maintainable in the long term and suitable for an open-source project.

---

# Decision

NC-PoRe uses component-specific technology selection.

Not all components need to be developed in the same programming language.

The architecture separates:

```
NC-PoRe

├── Recorder Client
│
├── Nextcloud Application
│
├── Backend Integration
│
└── Export Components
```

---

# Recorder Development

## Decision

The recorder is developed using a native or native-like technology.

Evaluation criteria:

- performance
- stability
- memory and resource control
- audio support
- platform capability
- FOSS compatibility

---

# Candidate Technologies

## Rust

Advantages:

- modern systems language
- strong memory safety
- good performance
- strong open-source community

Disadvantages:

- audio ecosystem less established than C++

---

## C++ / Qt

Advantages:

- long-standing experience in audio development
- extensive libraries
- professional applications use this stack

Disadvantages:

- higher complexity
- greater responsibility for memory management

---

## Python

Use:

- prototyping
- technical experiments
- tests

Not intended as the final recorder foundation.

---

# Decision Principle

The final selection of the recorder stack is made after a technical prototype.

The prototype must demonstrate:

- microphone access
- WAV recording
- stable long-duration recording
- chunk processing

---

# Nextcloud Development

The Nextcloud component follows the existing Nextcloud ecosystem.

Responsibilities:

- user management
- sessions
- roles
- metadata
- file management

The Nextcloud app is not responsible for audio recording.

---

# Development Tools

NC-PoRe preferably uses:

- Git for version control
- open development tools
- automated tests
- documented build processes

---

# Build Philosophy

Build processes should be:

- reproducible
- documented
- automatable

---

# Consequences

## Positive Effects

- appropriate tools for each task
- professional audio architecture possible
- long-term maintainability
- better extensibility

---

## Negative Effects

- multiple technologies must be maintained
- higher initial effort
- developers must understand multiple areas

---

# Alternatives Considered

## One Language for All Components

Rejected.

Reason:

The requirements of the recorder and web administration differ significantly.

---

## Web Technology for the Recorder

Rejected as the primary approach.

Reason:

Not sufficient for professional audio hardware control.

---

# Notes

NC-PoRe follows the principle:

> Architecture determines the tools, not the other way around.
