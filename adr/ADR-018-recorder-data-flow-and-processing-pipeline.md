# Deutsch ([English version below](#english-version))

# ADR-018: Recorder Data Flow and Processing Pipeline

## Status

Accepted

## Date

2026-07-23

---

# Context

Der NC-PoRe Recorder muss Audiodaten erfassen,
verarbeiten und für spätere Nutzung bereitstellen.

Damit die einzelnen Komponenten klar getrennte
Aufgaben besitzen, benötigt der Recorder eine
definierte Verarbeitungskette.

Die Architektur soll vermeiden, dass einzelne
Komponenten direkt voneinander abhängig werden.

Der Datenfluss muss nachvollziehbar, testbar und
erweiterbar bleiben.

---

# Decision

Der Recorder verwendet eine klar definierte
Verarbeitungspipeline.

Die grundlegende Verarbeitung erfolgt in folgenden
Schritten:

```
Audio Input

    ↓

Audio Capture

    ↓

Buffer Management

    ↓

Session Management

    ↓

Metadata Handling

    ↓

Local Storage

    ↓

Export Interface
```

Jede Komponente besitzt eine klar begrenzte
Verantwortung.

---

# Pipeline Components

## Audio Input

Verantwortlich für:

- Bereitstellung der Audioquelle
- Erkennung verfügbarer Eingabegeräte
- Übergabe von Audiodaten an die Audio-Schicht

---

## Audio Capture

Verantwortlich für:

- Aufnahme des Audio-Streams
- Umwandlung in interne Datenstrukturen
- Weitergabe der Audiodaten

Die Audio-Komponente kennt keine Speicherung
und keine Exportlogik.

---

## Buffer Management

Verantwortlich für:

- Zwischenspeicherung während der Aufnahme
- Ausgleich unterschiedlicher Verarbeitungsgeschwindigkeiten
- stabile Datenübergabe zwischen Komponenten

---

## Session Management

Verantwortlich für:

- Start und Ende einer Aufnahme
- Verwaltung des Aufnahmezustands
- Zuordnung von Daten zu einer Session

Eine Session bildet die logische Einheit einer
Aufnahme.

---

## Metadata Handling

Verantwortlich für:

- Erzeugung und Verwaltung von Metadaten
- technische Aufnahmeinformationen
- zusätzliche Beschreibungen

Metadaten werden getrennt von Audiodaten behandelt.

---

## Local Storage

Verantwortlich für:

- lokale Speicherung von Aufnahmen
- Verwaltung temporärer Daten
- Sicherstellung der Datenintegrität

---

## Export Interface

Verantwortlich für:

- Übergabe fertiger Aufnahmen an externe Systeme
- zukünftige Integration mit Nextcloud
- Export in unterschiedliche Formate

---

# Alternatives Considered

## Direct Audio-To-Storage Pipeline

Audiodaten werden direkt vom Audio-Eingang
in Dateien geschrieben.

Verworfen wegen:

- fehlender Flexibilität
- schlechter Erweiterbarkeit
- schwieriger Verarbeitung von Metadaten

---

## Single Recorder Component

Alle Funktionen befinden sich in einer zentralen
Recorder-Komponente.

Verworfen wegen:

- hoher Kopplung
- schwieriger Wartbarkeit
- schlechter Testbarkeit

---

# Consequences

## Positive Consequences

- klare Verantwortlichkeiten
- bessere Testbarkeit
- einfachere Erweiterung
- nachvollziehbarer Datenfluss
- bessere Fehlerisolierung

---

## Negative Consequences

- zusätzliche Struktur
- Kommunikation zwischen Komponenten muss definiert werden
- höherer anfänglicher Entwicklungsaufwand

---

# Error Handling Principles

Fehler sollen möglichst nahe an ihrer Ursache
behandelt werden.

Beispiele:

- Audiofehler innerhalb der Audio-Schicht
- Speicherfehler innerhalb der Storage-Schicht
- Exportfehler innerhalb der Export-Schicht

Fehlerinformationen sollen für Diagnose und
Benutzerinformation erhalten bleiben.

---

# Future Considerations

Spätere Entscheidungen müssen definieren:

- konkrete Datenstrukturen
- Audioformat intern
- Persistenzformat für Metadaten
- Kommunikation zwischen Modulen
- Verhalten bei Unterbrechungen

---

# Final Principle

Der Recorder soll Daten nicht nur aufnehmen.

Er soll sie nachvollziehbar, sicher und erweiterbar
durch die gesamte Verarbeitungskette führen.

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-018: Recorder Data Flow and Processing Pipeline

## Status

Accepted

## Date

2026-07-23

---

# Context

The NC-PoRe Recorder must capture audio data, process it, and make it available for later use.

To ensure that the individual components have clearly separated responsibilities, the Recorder requires a defined processing chain.

The architecture should prevent individual components from becoming directly dependent on each other.

The data flow must remain understandable, testable, and extensible.

---

# Decision

The Recorder uses a clearly defined processing pipeline.

The basic processing takes place in the following steps:

```
Audio Input

    ↓

Audio Capture

    ↓

Buffer Management

    ↓

Session Management

    ↓

Metadata Handling

    ↓

Local Storage

    ↓

Export Interface
```

Each component has a clearly limited responsibility.

---

# Pipeline Components

## Audio Input

Responsible for:

- providing the audio source
- detecting available input devices
- passing audio data to the audio layer

---

## Audio Capture

Responsible for:

- recording the audio stream
- converting it into internal data structures
- forwarding audio data

The audio component has no knowledge of storage or export logic.

---

## Buffer Management

Responsible for:

- temporary buffering during recording
- balancing different processing speeds
- stable data transfer between components

---

## Session Management

Responsible for:

- starting and ending a recording
- managing the recording state
- assigning data to a session

A session forms the logical unit of a recording.

---

## Metadata Handling

Responsible for:

- creating and managing metadata
- technical recording information
- additional descriptions

Metadata is handled separately from audio data.

---

## Local Storage

Responsible for:

- local storage of recordings
- management of temporary data
- ensuring data integrity

---

## Export Interface

Responsible for:

- passing completed recordings to external systems
- future integration with Nextcloud
- exporting to different formats

---

# Alternatives Considered

## Direct Audio-To-Storage Pipeline

Audio data is written directly from the audio input into files.

Rejected because of:

- lack of flexibility
- poor extensibility
- difficult metadata processing

---

## Single Recorder Component

All functions are contained in one central Recorder component.

Rejected because of:

- high coupling
- difficult maintenance
- poor testability

---

# Consequences

## Positive Consequences

- clear responsibilities
- better testability
- easier extension
- understandable data flow
- better error isolation

---

## Negative Consequences

- additional structure
- communication between components must be defined
- higher initial development effort

---

# Error Handling Principles

Errors should be handled as close to their cause as possible.

Examples:

- audio errors within the audio layer
- storage errors within the storage layer
- export errors within the export layer

Error information should be retained for diagnosis and user information.

---

# Future Considerations

Later decisions must define:

- concrete data structures
- internal audio format
- persistence format for metadata
- communication between modules
- behavior during interruptions

---

# Final Principle

The Recorder should not merely capture data.

It should carry that data through the entire processing chain in a traceable, secure, and extensible manner.
