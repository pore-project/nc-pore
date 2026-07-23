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

* Bereitstellung der Audioquelle
* Erkennung verfügbarer Eingabegeräte
* Übergabe von Audiodaten an die Audio-Schicht

---

## Audio Capture

Verantwortlich für:

* Aufnahme des Audio-Streams
* Umwandlung in interne Datenstrukturen
* Weitergabe der Audiodaten

Die Audio-Komponente kennt keine Speicherung
und keine Exportlogik.

---

## Buffer Management

Verantwortlich für:

* Zwischenspeicherung während der Aufnahme
* Ausgleich unterschiedlicher Verarbeitungsgeschwindigkeiten
* stabile Datenübergabe zwischen Komponenten

---

## Session Management

Verantwortlich für:

* Start und Ende einer Aufnahme
* Verwaltung des Aufnahmezustands
* Zuordnung von Daten zu einer Session

Eine Session bildet die logische Einheit einer
Aufnahme.

---

## Metadata Handling

Verantwortlich für:

* Erzeugung und Verwaltung von Metadaten
* technische Aufnahmeinformationen
* zusätzliche Beschreibungen

Metadaten werden getrennt von Audiodaten behandelt.

---

## Local Storage

Verantwortlich für:

* lokale Speicherung von Aufnahmen
* Verwaltung temporärer Daten
* Sicherstellung der Datenintegrität

---

## Export Interface

Verantwortlich für:

* Übergabe fertiger Aufnahmen an externe Systeme
* zukünftige Integration mit Nextcloud
* Export in unterschiedliche Formate

---

# Alternatives Considered

## Direct Audio-To-Storage Pipeline

Audiodaten werden direkt vom Audio-Eingang
in Dateien geschrieben.

Verworfen wegen:

* fehlender Flexibilität
* schlechter Erweiterbarkeit
* schwieriger Verarbeitung von Metadaten

---

## Single Recorder Component

Alle Funktionen befinden sich in einer zentralen
Recorder-Komponente.

Verworfen wegen:

* hoher Kopplung
* schwieriger Wartbarkeit
* schlechter Testbarkeit

---

# Consequences

## Positive Consequences

* klare Verantwortlichkeiten
* bessere Testbarkeit
* einfachere Erweiterung
* nachvollziehbarer Datenfluss
* bessere Fehlerisolierung

---

## Negative Consequences

* zusätzliche Struktur
* Kommunikation zwischen Komponenten muss definiert werden
* höherer anfänglicher Entwicklungsaufwand

---

# Error Handling Principles

Fehler sollen möglichst nahe an ihrer Ursache
behandelt werden.

Beispiele:

* Audiofehler innerhalb der Audio-Schicht
* Speicherfehler innerhalb der Storage-Schicht
* Exportfehler innerhalb der Export-Schicht

Fehlerinformationen sollen für Diagnose und
Benutzerinformation erhalten bleiben.

---

# Future Considerations

Spätere Entscheidungen müssen definieren:

* konkrete Datenstrukturen
* Audioformat intern
* Persistenzformat für Metadaten
* Kommunikation zwischen Modulen
* Verhalten bei Unterbrechungen

---

# Final Principle

Der Recorder soll Daten nicht nur aufnehmen.

Er soll sie nachvollziehbar, sicher und erweiterbar
durch die gesamte Verarbeitungskette führen.
