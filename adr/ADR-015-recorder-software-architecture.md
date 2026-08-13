# Deutsch ([English version below](#english-version))

# ADR-015: Initial Architecture of the NC-PoRe Recorder Client

## Status

Accepted

## Date

2026-07-23

---

# Kontext

Der Recorder Client ist die erste aktive Softwarekomponente von NC-PoRe.

Er bildet die Grundlage für die lokale Audioerfassung und spätere Verarbeitung.

Die Architektur muss langfristig erweiterbar, wartbar und für weitere Entwickler verständlich sein.

Der Recorder soll nicht als einzelne große Softwarekomponente entstehen, sondern aus klar getrennten Verantwortungsbereichen bestehen.

---

# Entscheidung

Der NC-PoRe Recorder wird modular aufgebaut.

Die grundlegenden Verantwortungsbereiche werden getrennt voneinander entwickelt.

Geplante Kernbereiche:

- Audio Capture
- Session Management
- Metadata Handling
- Local Storage
- Export Interface

Die konkrete technische Umsetzung einzelner Module wird durch spätere technische Entscheidungen festgelegt.

---

# Module Concept

Die logische Struktur des Recorders wird modular organisiert.

Geplante Module:

```
recorder/
└── src/
    ├── audio/
    ├── session/
    ├── metadata/
    ├── storage/
    ├── export/
    └── main.rs
```

Die konkrete Dateiorganisation kann während der Entwicklung angepasst werden, wenn praktische Erfahrungen dies erforderlich machen.

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
- zukünftige Integration mit NC-PoRe-Komponenten
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

# Zukünftige Überlegungen

Die konkrete technische Implementierung der Module wird durch weitere ADRs und Entwicklungsentscheidungen festgelegt.

Bibliotheken und Frameworks werden erst ausgewählt, wenn die technischen Anforderungen ausreichend klar sind.

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

The Recorder Client is the first active software component of NC-PoRe.

It provides the foundation for local audio capture and subsequent processing.

The architecture must remain extensible, maintainable, and understandable to future developers.

The recorder should not be built as one large software component, but as clearly separated areas of responsibility.

---

# Decision

The NC-PoRe Recorder will be built in a modular way.

The fundamental areas of responsibility will be developed separately.

Planned core areas:

- Audio Capture
- Session Management
- Metadata Handling
- Local Storage
- Export Interface

The concrete technical implementation of individual modules will be defined by later technical decisions.

---

# Module Concept

The logical structure of the recorder is organized into modules.

Planned modules:

```
recorder/
└── src/
    ├── audio/
    ├── session/
    ├── metadata/
    ├── storage/
    ├── export/
    └── main.rs
```

The concrete file organization may be adjusted during development if practical experience makes this necessary.

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
- future integration with NC-PoRe components
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

# Future Considerations

The concrete technical implementation of the modules will be defined by further ADRs and development decisions.

Libraries and frameworks will only be selected once the technical requirements are sufficiently clear.

---

# Final Principle

The recorder should not merely work.

It should be understandable, extensible, and maintainable over the long term.
