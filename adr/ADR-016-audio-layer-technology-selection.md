# Deutsch ([English version below](#english-version))

# ADR-016: Audio Layer Technology Selection

## Status

Accepted

## Date

2026-07-23

---

# Context

Der NC-PoRe Recorder benötigt eine technische Grundlage
für die Aufnahme und Verarbeitung von Audiodaten.

Die Audio-Schicht ist eine zentrale Komponente,
da sie direkt mit Hardware und Betriebssystemen
interagiert.

Die Lösung muss langfristig wartbar und erweiterbar sein.

NC-PoRe soll nicht auf eine einzelne Plattform
beschränkt werden.

---

# Decision

Die Audio-Schicht wird über eine plattformübergreifende
Abstraktion umgesetzt.

Der Recorder verwendet keine direkten,
plattformabhängigen Audio-Schnittstellen in der
Anwendungslogik.

Die Audio-Hardware wird über eine separate
Audio-Layer-Komponente angesprochen.

Die konkrete Bibliothek wird in einer späteren
technischen Entscheidung festgelegt.

---

# Rationale

Eine Abstraktionsschicht bietet:

- Unterstützung mehrerer Betriebssysteme
- bessere Wartbarkeit
- einfachere Tests
- geringere Abhängigkeit von einzelnen Plattformen
- Möglichkeit zum Austausch der technischen Umsetzung

---

# Audio Architecture Concept

Die geplante Struktur:

```
Audio Hardware

      ↓

Operating System Audio Backend

      ↓

Rust Audio Layer

      ↓

NC-PoRe Recorder Pipeline

      ↓

Storage / Export
```

---

# Alternatives Considered

## Direct Platform APIs

Direkte Verwendung von Betriebssystem-spezifischen
Audio-Schnittstellen.

Beispiele:

- ALSA
- CoreAudio
- WASAPI

Verworfen wegen:

- hoher Plattformabhängigkeit
- höherem Wartungsaufwand
- erschwerter Erweiterbarkeit

---

## Separate Implementations per Platform

Eigene Audio-Implementierungen für jedes
unterstützte Betriebssystem.

Verworfen wegen:

- doppelter Entwicklungsaufwand
- unterschiedlicher Fehlerquellen
- schwieriger langfristiger Pflege

---

# Consequences

## Positive Consequences

- plattformübergreifende Architektur
- klar getrennte Verantwortlichkeiten
- bessere Erweiterbarkeit
- einfachere zukünftige Wartung

---

## Negative Consequences

- zusätzliche Abstraktionsebene
- möglicher Mehraufwand bei speziellen Hardwarefunktionen

---

# Future Considerations

Die konkrete Audio-Bibliothek wird in einer späteren
technischen Entscheidung ausgewählt.

Dabei werden folgende Kriterien bewertet:

- Stabilität
- Plattformunterstützung
- Latenz
- Echtzeitfähigkeit
- Wartbarkeit
- Lizenzmodell

---

# Final Principle

Die Audio-Schicht soll nicht nur heute funktionieren.

Sie soll eine stabile Grundlage für die zukünftige
Entwicklung von NC-PoRe bilden.

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-016: Audio Layer Technology Selection

## Status

Accepted

## Date

2026-07-23

---

# Context

The NC-PoRe Recorder requires a technical foundation
for recording and processing audio data.

The audio layer is a central component because it
interacts directly with hardware and operating systems.

The solution must remain maintainable and extensible over the long term.

NC-PoRe should not be restricted to a single platform.

---

# Decision

The audio layer is implemented through a cross-platform abstraction.

The Recorder does not use direct platform-dependent
audio interfaces in application logic.

Audio hardware is accessed through a separate audio layer component.

The concrete library will be selected in a later technical decision.

---

# Rationale

An abstraction layer provides:

- support for multiple operating systems
- better maintainability
- easier testing
- reduced dependency on individual platforms
- the ability to replace the technical implementation

---

# Audio Architecture Concept

The planned structure:

```
Audio Hardware

      ↓

Operating System Audio Backend

      ↓

Rust Audio Layer

      ↓

NC-PoRe Recorder Pipeline

      ↓

Storage / Export
```

---

# Alternatives Considered

## Direct Platform APIs

Direct use of operating-system-specific
audio interfaces.

Examples:

- ALSA
- CoreAudio
- WASAPI

Rejected because of:

- high platform dependency
- increased maintenance effort
- reduced extensibility

---

## Separate Implementations per Platform

Separate audio implementations for each
supported operating system.

Rejected because of:

- duplicated development effort
- different sources of errors
- more difficult long-term maintenance

---

# Consequences

## Positive Consequences

- cross-platform architecture
- clearly separated responsibilities
- better extensibility
- easier future maintenance

---

## Negative Consequences

- additional abstraction layer
- possible additional effort for specialized hardware functions

---

# Future Considerations

The concrete audio library will be selected in a later technical decision.

The following criteria will be evaluated:

- stability
- platform support
- latency
- real-time capability
- maintainability
- licensing model

---

# Final Principle

The audio layer should not only work today.

It should provide a stable foundation for the future
development of NC-PoRe.
