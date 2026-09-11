# Deutsch ([English version below](#english-version))

# ADR-015: Initial Architecture of the NC-PoRe Recorder Client

## Status

Accepted

## Date

2026-07-23

---

# Kontext

Der Recorder Client ist eine aktive Softwarekomponente von NC-PoRe.

Er bildet die Grundlage für die lokale Audioerfassung und nachgelagerte Verarbeitung.

Die Architektur soll erweiterbar, wartbar und für weitere Entwickler verständlich sein.

Der Recorder soll nicht als einzelne große Softwarekomponente entstehen, sondern aus klar getrennten Verantwortungsbereichen bestehen.

---

# Entscheidung

Der NC-PoRe Recorder wird modular aufgebaut.

Die grundlegenden Verantwortungsbereiche werden getrennt voneinander entwickelt.

Die Kernbereiche sind:

- Audio Capture
- Session Management
- Metadata Handling
- Local Storage
- Export Interface

Die konkrete technische Umsetzung einzelner Module wird durch technische Entscheidungen festgelegt.

---

# Module Concept

Die logische Struktur des Recorders wird modular organisiert.

Konzeptionell:

```text
recorder/
└── src/
    ├── audio/
    ├── session/
    ├── metadata/
    ├── storage/
    ├── export/
    └── main.rs
```

Die konkrete Dateiorganisation kann angepasst werden, wenn praktische Erfahrungen dies erforderlich machen.

---

# Module Responsibilities

## Audio Capture

Verantwortlich für:

- Zugriff auf Audioquellen
- Aufnahme von Audiodaten
- Verarbeitung von Audio-Streams
- Buffer-Verwaltung

---

## Session Management

Verantwortlich für:

- Verwaltung von Aufnahmesitzungen
- Start und Stop von Aufnahmen
- Zustandsverwaltung
- Sitzungsinformationen

---

## Metadata Handling

Verantwortlich für:

- Aufnahmeinformationen
- Zeitstempel
- technische Parameter
- zusätzliche Beschreibungen

---

## Local Storage

Verantwortlich für:

- lokale Speicherung von Audiodaten
- Verwaltung temporärer Dateien
- Dateiorganisation

---

## Export Interface

Verantwortlich für:

- Übergabe von Aufnahmen an andere Systeme
- Übergabe an andere NC-PoRe-Komponenten
- Exportformate

---

# Betrachtete Alternativen

## Monolithic Recorder

Eine Implementierung aller Funktionen in einer einzigen Datei oder einem einzigen Modul.

Verworfen wegen:

- schlechter Erweiterbarkeit
- schwieriger Testbarkeit
- höherem Wartungsaufwand

---

## Immediate Cloud Integration

Direkte Verbindung mit Nextcloud bereits in der ersten Entwicklungsphase.

Verworfen wegen:

- unnötiger Kopplung
- erschwerter lokaler Entwicklung
- schlechterer Testbarkeit einzelner Komponenten

---

# Konsequenzen

## Positive Auswirkungen

- klare Verantwortlichkeiten
- bessere Wartbarkeit
- bessere Testbarkeit
- einfachere Erweiterung
- bessere Zusammenarbeit mehrerer Entwickler

---

## Negative Auswirkungen

- zusätzliche Struktur am Anfang
- etwas höherer Planungsaufwand

---

# Nicht durch diese ADR festgelegt

Die konkrete technische Implementierung der Module, Bibliotheken und Frameworks wird durch jeweils erforderliche technische Entscheidungen festgelegt.

---

# Grundprinzip

Der Recorder soll nicht nur funktionieren.

Er soll verständlich, erweiterbar und langfristig wartbar sein.

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-015: Initial Architecture of the NC-PoRe Recorder Client

## Status

Accepted

## Date

2026-07-23

---

# Context

The Recorder Client is an active software component of NC-PoRe.

It provides the foundation for local audio capture and subsequent processing.

The architecture should remain extensible, maintainable, and understandable to other developers.

The recorder should not be built as one large software component, but as clearly separated areas of responsibility.

---

# Decision

The NC-PoRe Recorder is built in a modular way.

The fundamental areas of responsibility are developed separately.

The core areas are:

- Audio Capture
- Session Management
- Metadata Handling
- Local Storage
- Export Interface

The concrete technical implementation of individual modules is defined by technical decisions.

---

# Module Concept

The logical structure of the recorder is organized into modules.

Conceptually:

```text
recorder/
└── src/
    ├── audio/
    ├── session/
    ├── metadata/
    ├── storage/
    ├── export/
    └── main.rs
```

The concrete file organization may be adjusted if practical experience requires it.

---

# Module Responsibilities

## Audio Capture

Responsible for:

- access to audio sources
- recording audio data
- processing audio streams
- buffer management

---

## Session Management

Responsible for:

- managing recording sessions
- starting and stopping recordings
- state management
- session information

---

## Metadata Handling

Responsible for:

- recording information
- timestamps
- technical parameters
- additional descriptions

---

## Local Storage

Responsible for:

- local storage of audio data
- management of temporary files
- file organization

---

## Export Interface

Responsible for:

- handing recordings over to other systems
- handing recordings to other NC-PoRe components
- export formats

---

# Alternatives Considered

## Monolithic Recorder

An implementation of all functions in a single file or module.

Rejected because of:

- poor extensibility
- difficult testing
- higher maintenance effort

---

## Immediate Cloud Integration

Direct integration with Nextcloud during the first development phase.

Rejected because of:

- unnecessary coupling
- more difficult local development
- poorer testability of individual components

---

# Consequences

## Positive Consequences

- clear responsibilities
- better maintainability
- better testability
- easier extension
- better collaboration between multiple developers

---

## Negative Consequences

- additional structure at the beginning
- somewhat higher planning effort

---

# Not Defined by This ADR

The concrete technical implementation of modules, libraries, and frameworks is defined by technical decisions when required.

---

# Final Principle

The recorder should not merely work.

It should be understandable, extensible, and maintainable over the long term.
