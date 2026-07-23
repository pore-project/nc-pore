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

* Unterstützung mehrerer Betriebssysteme
* bessere Wartbarkeit
* einfachere Tests
* geringere Abhängigkeit von einzelnen Plattformen
* Möglichkeit zum Austausch der technischen Umsetzung

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

* ALSA
* CoreAudio
* WASAPI

Verworfen wegen:

* hoher Plattformabhängigkeit
* höherem Wartungsaufwand
* erschwerter Erweiterbarkeit

---

## Separate Implementations per Platform

Eigene Audio-Implementierungen für jedes
unterstützte Betriebssystem.

Verworfen wegen:

* doppelter Entwicklungsaufwand
* unterschiedlicher Fehlerquellen
* schwieriger langfristiger Pflege

---

# Consequences

## Positive Consequences

* plattformübergreifende Architektur
* klar getrennte Verantwortlichkeiten
* bessere Erweiterbarkeit
* einfachere zukünftige Wartung

---

## Negative Consequences

* zusätzliche Abstraktionsebene
* möglicher Mehraufwand bei speziellen Hardwarefunktionen

---

# Future Considerations

Die konkrete Audio-Bibliothek wird in einer späteren
technischen Entscheidung ausgewählt.

Dabei werden folgende Kriterien bewertet:

* Stabilität
* Plattformunterstützung
* Latenz
* Echtzeitfähigkeit
* Wartbarkeit
* Lizenzmodell

---

# Final Principle

Die Audio-Schicht soll nicht nur heute funktionieren.

Sie soll eine stabile Grundlage für die zukünftige
Entwicklung von NC-PoRe bilden.
