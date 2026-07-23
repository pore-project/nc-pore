# ADR-017: Audio Backend Library Selection

## Status

Accepted

## Date

2026-07-23

---

# Context

ADR-016 hat entschieden, dass die Audio-Schicht von NC-PoRe
über eine plattformübergreifende Abstraktion umgesetzt wird.

Für die praktische Umsetzung benötigt der Recorder eine
geeignete Audio-Bibliothek.

Die Auswahl muss technische Anforderungen,
langfristige Wartbarkeit und die Open-Source-Philosophie
von NC-PoRe berücksichtigen.

NC-PoRe soll langfristig auf verschiedenen Plattformen
einsetzbar sein.

Dabei werden sowohl klassische Desktop-Systeme als auch
mobile Plattformen berücksichtigt.

---

# Decision

NC-PoRe verwendet zunächst die Rust-Audio-Bibliothek
`cpal` als technische Grundlage für die Audioaufnahme.

`cpal` wird als Audio-Abstraktionsschicht zwischen
Betriebssystem-Audio-Backends und der Recorder-Logik
eingesetzt.

Die Audio-Implementierung wird so gestaltet, dass die
Abhängigkeit von der Bibliothek möglichst gering bleibt.

Ein späterer Austausch der Audio-Bibliothek bleibt möglich.

---

# Target Platforms

NC-PoRe berücksichtigt folgende Zielplattformen:

## Desktop

* Linux
* Windows
* macOS

## Mobile

* Android
* iOS

Die konkrete Audio-Anbindung kann je nach Plattform
unterschiedliche technische Backends verwenden.

Die Recorder-Architektur bleibt dabei möglichst
plattformübergreifend.

---

# Selection Criteria

Die Auswahl wurde anhand folgender Kriterien getroffen:

## Plattformunterstützung

Die Lösung soll verschiedene Betriebssysteme unterstützen.

Bewertet werden:

* Desktop-Plattformen
* mobile Plattformen
* langfristige Erweiterbarkeit

---

## Rust Integration

Die Bibliothek soll nativ in das Rust-Ökosystem passen.

Vorteile:

* gute Integration mit Cargo
* moderne Entwicklungsumgebung
* gute Wartbarkeit

---

## Open Source

Die verwendete Software muss mit der Open-Source-Philosophie
von NC-PoRe vereinbar sein.

Bewertet werden:

* Lizenz
* Transparenz
* Community
* langfristige Verfügbarkeit

---

## Echtzeitfähigkeit

Die Audioaufnahme benötigt:

* niedrige Latenz
* stabile Audio-Streams
* zuverlässige Buffer-Verarbeitung

---

# Alternatives Considered

## Direct Platform APIs

Direkte Nutzung von:

* ALSA unter Linux
* CoreAudio unter macOS
* WASAPI unter Windows
* nativen Mobile Audio APIs

Verworfen als primäre Architektur wegen:

* hoher Plattformabhängigkeit
* mehrfacher Implementierung
* höherem Wartungsaufwand

---

## Separate Audio Implementations per Platform

Eigene Audio-Implementierungen für jede Plattform.

Verworfen wegen:

* doppeltem Entwicklungsaufwand
* unterschiedlichen Fehlerquellen
* schwieriger langfristiger Pflege

---

## Andere Rust Audio Libraries

Andere Lösungen wurden betrachtet.

Sie bleiben bei zukünftigen Anforderungen eine mögliche
Alternative.

---

# Consequences

## Positive Consequences

* plattformübergreifende Entwicklung
* gute Rust-Integration
* klare Trennung zwischen Audio-Hardware und Recorder-Logik
* bessere Wartbarkeit
* Möglichkeit zur späteren Erweiterung

---

## Negative Consequences

* zusätzliche Abhängigkeit
* Abhängigkeit von einem externen Open-Source-Projekt
* mögliche Einschränkungen bei sehr speziellen Hardwareanforderungen

---

# Implementation Strategy

Die erste Implementierung verwendet `cpal` nur innerhalb
einer klar abgegrenzten Audio-Komponente.

Die restliche Recorder-Architektur soll keine direkten
Abhängigkeiten von `cpal` enthalten.

Plattformspezifische Anpassungen werden innerhalb der
Audio-Schicht gekapselt.

---

# Future Considerations

Nach ersten praktischen Erfahrungen werden folgende Punkte
bewertet:

* Audioqualität
* Stabilität bei langen Aufnahmen
* CPU-Auslastung
* Latenz
* Plattformverhalten
* Verhalten auf mobilen Geräten

Bei grundlegenden Änderungen wird eine neue ADR erstellt.

---

# Final Principle

Die Audio-Technologie soll eine stabile Grundlage bieten,
ohne die langfristige Flexibilität von NC-PoRe einzuschränken.
