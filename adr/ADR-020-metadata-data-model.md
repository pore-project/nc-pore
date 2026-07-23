# ADR-020: Metadata Data Model

## Status

Accepted

## Date

2026-07-23

---

# Context

NC-PoRe behandelt eine Aufnahme nicht nur als
Audiodatei, sondern als verwaltbare Einheit.

ADR-019 definiert die Recording Session als
zentrale Einheit einer Aufnahme.

Eine Session benötigt zusätzliche Informationen,
um nachvollziehbar gespeichert, verwaltet und
später verarbeitet werden zu können.

Diese Informationen werden als Metadaten geführt.

---

# Decision

NC-PoRe verwendet ein separates Metadata Model.

Metadaten werden unabhängig von den eigentlichen
Audiodaten behandelt.

Das Metadata Model beschreibt technische,
organisatorische und optionale beschreibende
Informationen einer Aufnahme.

---

# Metadata Categories

Metadaten werden in mehrere Bereiche unterteilt.

---

# Technical Metadata

Technische Informationen über die Aufnahme.

Beispiele:

* Audioformat
* Sample Rate
* Anzahl der Kanäle
* Bit-Tiefe
* Aufnahmedauer
* verwendete Recorder-Version

Diese Informationen werden möglichst automatisch
erzeugt.

---

# Session Metadata

Informationen zur Verwaltung der Aufnahme.

Beispiele:

* Session ID
* Erstellungszeitpunkt
* Startzeit
* Endzeit
* Status

Diese Informationen werden durch das
Session Management verwaltet.

---

# User Metadata

Informationen, die durch Benutzer oder
Anwendungen ergänzt werden können.

Beispiele:

* Titel
* Beschreibung
* Tags
* Notizen

Diese Informationen sind optional.

---

# System Metadata

Informationen über die technische Umgebung.

Beispiele:

* Betriebssystem
* verwendetes Gerät
* Anwendungsversion

Diese Informationen unterstützen Diagnose
und Support.

---

# Data Model Concept

Die grundsätzliche Struktur:

```text
RecordingSession

    |
    |
    +-- Metadata

          |
          +-- Technical Metadata

          +-- Session Metadata

          +-- User Metadata

          +-- System Metadata
```

---

# Decision Principles

Das Metadata Model folgt diesen Prinzipien:

* klare Trennung von Audio und Beschreibung
* Erweiterbarkeit ohne Änderung vorhandener Daten
* optionale Felder ermöglichen zukünftige Funktionen
* maschinenlesbare Speicherung

---

# Alternatives Considered

## Metadata Only Inside Audio Files

Metadaten werden ausschließlich in Audiodateien
gespeichert.

Verworfen wegen:

* Abhängigkeit von Dateiformaten
* eingeschränkter Erweiterbarkeit
* schlechter Trennung von Daten und Beschreibung

---

## No Separate Metadata Model

Alle Informationen werden direkt in einzelnen
Komponenten verwaltet.

Verworfen wegen:

* fehlender Übersichtlichkeit
* schwieriger Synchronisation
* hoher Kopplung

---

# Consequences

## Positive Consequences

* klare Datenstruktur
* bessere Suche und Verwaltung
* Grundlage für Nextcloud-Integration
* einfache Erweiterbarkeit

---

## Negative Consequences

* zusätzliche Datenhaltung
* Modell muss langfristig gepflegt werden

---

# Future Considerations

Spätere Entscheidungen müssen definieren:

* konkrete Rust-Strukturen
* Serialisierungsformat
* Datenbank- oder Dateispeicherung
* Synchronisationsmodell
* Datenschutzaspekte

---

# Final Principle

Metadaten machen aus einer Audiodatei eine
nachvollziehbare und verwaltbare Aufnahme.

NC-PoRe behandelt Informationen über eine Aufnahme
als gleichwertigen Bestandteil der Architektur.
