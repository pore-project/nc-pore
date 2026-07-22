# ADR-014: Development Environment and Toolchain

## Status

Accepted

## Date

2026-07-22

---

# Context

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

# Decision

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

## Decision

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

# Consequences

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

# Alternatives considered

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

# Notes

NC-PoRe folgt dem Prinzip:

> Die Architektur bestimmt die Werkzeuge,
> nicht umgekehrt.
```