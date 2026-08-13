# Deutsch ([English version below](#english-version))

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

- Linux
- Windows
- macOS

## Mobile

- Android
- iOS

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

- Desktop-Plattformen
- mobile Plattformen
- langfristige Erweiterbarkeit

---

## Rust Integration

Die Bibliothek soll nativ in das Rust-Ökosystem passen.

Vorteile:

- gute Integration mit Cargo
- moderne Entwicklungsumgebung
- gute Wartbarkeit

---

## Open Source

Die verwendete Software muss mit der Open-Source-Philosophie
von NC-PoRe vereinbar sein.

Bewertet werden:

- Lizenz
- Transparenz
- Community
- langfristige Verfügbarkeit

---

## Echtzeitfähigkeit

Die Audioaufnahme benötigt:

- niedrige Latenz
- stabile Audio-Streams
- zuverlässige Buffer-Verarbeitung

---

# Alternatives Considered

## Direct Platform APIs

Direkte Nutzung von:

- ALSA unter Linux
- CoreAudio unter macOS
- WASAPI unter Windows
- nativen Mobile Audio APIs

Verworfen als primäre Architektur wegen:

- hoher Plattformabhängigkeit
- mehrfacher Implementierung
- höherem Wartungsaufwand

---

## Separate Audio Implementations per Platform

Eigene Audio-Implementierungen für jede Plattform.

Verworfen wegen:

- doppeltem Entwicklungsaufwand
- unterschiedlichen Fehlerquellen
- schwieriger langfristiger Pflege

---

## Andere Rust Audio Libraries

Andere Lösungen wurden betrachtet.

Sie bleiben bei zukünftigen Anforderungen eine mögliche
Alternative.

---

# Consequences

## Positive Consequences

- plattformübergreifende Entwicklung
- gute Rust-Integration
- klare Trennung zwischen Audio-Hardware und Recorder-Logik
- bessere Wartbarkeit
- Möglichkeit zur späteren Erweiterung

---

## Negative Consequences

- zusätzliche Abhängigkeit
- Abhängigkeit von einem externen Open-Source-Projekt
- mögliche Einschränkungen bei sehr speziellen Hardwareanforderungen

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

- Audioqualität
- Stabilität bei langen Aufnahmen
- CPU-Auslastung
- Latenz
- Plattformverhalten
- Verhalten auf mobilen Geräten

Bei grundlegenden Änderungen wird eine neue ADR erstellt.

---

# Final Principle

Die Audio-Technologie soll eine stabile Grundlage bieten,
ohne die langfristige Flexibilität von NC-PoRe einzuschränken.

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-017: Audio Backend Library Selection

## Status

Accepted

## Date

2026-07-23

---

# Context

ADR-016 decided that the NC-PoRe audio layer would be implemented through a cross-platform abstraction.

For the practical implementation, the Recorder requires a suitable audio library.

The selection must take technical requirements, long-term maintainability, and the open-source philosophy of NC-PoRe into account.

NC-PoRe should remain usable across different platforms over the long term.

Both conventional desktop systems and mobile platforms are considered.

---

# Decision

NC-PoRe initially uses the Rust audio library `cpal` as the technical basis for audio recording.

`cpal` is used as the audio abstraction layer between operating-system audio backends and Recorder logic.

The audio implementation is designed to keep dependency on the library as limited as possible.

Replacing the audio library at a later point remains possible.

---

# Target Platforms

NC-PoRe considers the following target platforms:

## Desktop

- Linux
- Windows
- macOS

## Mobile

- Android
- iOS

The concrete audio integration may use different technical backends depending on the platform.

The Recorder architecture remains as platform-independent as possible.

---

# Selection Criteria

The selection was made according to the following criteria:

## Platform Support

The solution should support multiple operating systems.

Evaluated:

- desktop platforms
- mobile platforms
- long-term extensibility

---

## Rust Integration

The library should fit natively into the Rust ecosystem.

Advantages:

- good integration with Cargo
- modern development environment
- good maintainability

---

## Open Source

The software used must be compatible with the open-source philosophy of NC-PoRe.

Evaluated:

- license
- transparency
- community
- long-term availability

---

## Real-Time Capability

Audio recording requires:

- low latency
- stable audio streams
- reliable buffer processing

---

# Alternatives Considered

## Direct Platform APIs

Direct use of:

- ALSA on Linux
- CoreAudio on macOS
- WASAPI on Windows
- native mobile audio APIs

Rejected as the primary architecture because of:

- high platform dependency
- multiple implementations
- increased maintenance effort

---

## Separate Audio Implementations per Platform

Separate audio implementations for each platform.

Rejected because of:

- duplicated development effort
- different sources of errors
- more difficult long-term maintenance

---

## Other Rust Audio Libraries

Other solutions were considered.

They remain possible alternatives if future requirements make them appropriate.

---

# Consequences

## Positive Consequences

- cross-platform development
- good Rust integration
- clear separation between audio hardware and Recorder logic
- better maintainability
- possibility for later extension

---

## Negative Consequences

- additional dependency
- dependency on an external open-source project
- possible limitations for highly specialized hardware requirements

---

# Implementation Strategy

The initial implementation uses `cpal` only inside a clearly separated audio component.

The rest of the Recorder architecture should not contain direct dependencies on `cpal`.

Platform-specific adaptations are encapsulated within the audio layer.

---

# Future Considerations

After initial practical experience, the following aspects will be evaluated:

- audio quality
- stability during long recordings
- CPU utilization
- latency
- platform behavior
- behavior on mobile devices

A new ADR will be created if fundamental changes are required.

---

# Final Principle

The audio technology should provide a stable foundation without restricting the long-term flexibility of NC-PoRe.
