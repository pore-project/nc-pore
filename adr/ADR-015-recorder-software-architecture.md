# ADR-015: Initial Architecture of the NC-PoRe Recorder Client

## Status

Accepted

## Date

2026-07-23

---

# Context

Der Recorder Client ist die erste aktive Softwarekomponente von NC-PoRe.

Er bildet die Grundlage für die lokale Audioerfassung und spätere Verarbeitung.

Die Architektur muss langfristig erweiterbar, wartbar und für weitere Entwickler verständlich sein.

Der Recorder soll nicht als einzelne große Softwarekomponente entstehen, sondern aus klar getrennten Verantwortungsbereichen bestehen.

---

# Decision

Der NC-PoRe Recorder wird modular aufgebaut.

Die grundlegenden Verantwortungsbereiche werden getrennt voneinander entwickelt.

Geplante Kernbereiche:

* Audio Capture
* Session Management
* Metadata Handling
* Local Storage
* Export Interface

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

* Zugriff auf Audioquellen
* Aufnahme von Audiodaten
* Verarbeitung von Audio-Streams
* Buffer-Verwaltung

---

## Session Management

Verantwortlich für:

* Verwaltung von Aufnahmesitzungen
* Start und Stop von Aufnahmen
* Zustandsverwaltung
* Sitzungsinformationen

---

## Metadata Handling

Verantwortlich für:

* Aufnahmeinformationen
* Zeitstempel
* technische Parameter
* zusätzliche Beschreibungen

---

## Local Storage

Verantwortlich für:

* lokale Speicherung von Audiodaten
* Verwaltung temporärer Dateien
* Dateiorganisation

---

## Export Interface

Verantwortlich für:

* Übergabe von Aufnahmen an andere Systeme
* zukünftige Integration mit NC-PoRe-Komponenten
* Exportformate

---

# Alternatives Considered

## Monolithic Recorder

Eine Implementierung aller Funktionen in einer einzigen Datei oder einem einzigen Modul.

Verworfen wegen:

* schlechter Erweiterbarkeit
* schwieriger Testbarkeit
* höherem Wartungsaufwand

---

## Immediate Cloud Integration

Direkte Verbindung mit Nextcloud bereits in der ersten Entwicklungsphase.

Verworfen wegen:

* unnötiger Kopplung
* erschwerter lokaler Entwicklung
* schlechterer Testbarkeit einzelner Komponenten

---

# Consequences

## Positive Consequences

* klare Verantwortlichkeiten
* bessere Wartbarkeit
* bessere Testbarkeit
* einfachere Erweiterung
* bessere Zusammenarbeit mehrerer Entwickler

---

## Negative Consequences

* zusätzliche Struktur am Anfang
* etwas höherer Planungsaufwand

---

# Future Considerations

Die konkrete technische Implementierung der Module wird durch weitere ADRs und Entwicklungsentscheidungen festgelegt.

Bibliotheken und Frameworks werden erst ausgewählt, wenn die technischen Anforderungen ausreichend klar sind.

---

# Final Principle

Der Recorder soll nicht nur funktionieren.

Er soll verständlich, erweiterbar und langfristig wartbar sein.
