# ADR-013: Technology Stack

## Status

Accepted

## Date

2026-07-22

---

# Context

NC-PoRe besteht aus mehreren technisch unterschiedlichen
Bereichen.

Die Plattform benötigt:

- einen lokalen Audio-Recorder
- eine Serverintegration
- Benutzer- und Rechteverwaltung
- Metadatenverwaltung
- offene Schnittstellen

Die Komponenten sollen unabhängig voneinander
weiterentwickelt werden können.

---

# Decision

NC-PoRe wird als modulare Architektur mit getrennten
Komponenten umgesetzt.

Die Hauptkomponenten sind:

```
NC-PoRe

├── Recorder Client
│
├── Nextcloud Application
│
├── Backend Services
│
└── Export Layer
```

---

# Recorder Client

## Entscheidung

Der Recorder wird als eigenständige Anwendung entwickelt.

Begründung:

- direkter Hardwarezugriff
- stabile Audioverarbeitung
- unabhängig vom Browser
- bessere Kontrolle über Ressourcen
- professionelle Einsatzmöglichkeiten

---

# Recorder Technology

Für den Recorder wird eine native oder
native-nahe Technologie bevorzugt.

Bewertungskriterien:

- Audioqualität
- Stabilität
- Plattformunterstützung
- FOSS-Eignung
- langfristige Wartbarkeit

Mögliche Technologien:

- Rust
- C++
- Qt-basierte Anwendungen
- andere geeignete FOSS-Technologien

Die endgültige Auswahl erfolgt nach Prototyping.

---

# Server Integration

## Entscheidung

Die Serverintegration erfolgt als
Nextcloud-Anwendung.

Aufgaben:

- Projekte
- Sessions
- Benutzer
- Rollen
- Metadaten
- Dateiverwaltung

Die Nextcloud-App ist nicht für die primäre
Audioaufnahme verantwortlich.

---

# Communication

Recorder und Server kommunizieren über definierte
Schnittstellen.

Beispiele:

- Session-Erzeugung
- Authentifizierung
- Upload
- Statusinformationen
- Metadaten

---

# Database and Storage

Die Speicherung orientiert sich an den bestehenden
Nextcloud-Mechanismen.

NC-PoRe nutzt:

- offene Datenstrukturen
- dokumentierte Formate
- nachvollziehbare Metadaten

---

# Open Source Principles

Der Technologiestack soll unterstützen:

- freie Entwicklungswerkzeuge
- offene Standards
- Community-Beiträge
- langfristige Wartbarkeit

---

# Consequences

## Positive Auswirkungen

- klare Trennung der Verantwortlichkeiten
- professionelle Audioarchitektur möglich
- bessere Erweiterbarkeit
- geringere Abhängigkeiten

---

## Negative Auswirkungen

- mehrere Komponenten müssen gepflegt werden
- höherer initialer Entwicklungsaufwand
- Schnittstellen müssen sauber definiert werden

---

# Alternatives considered

## Alles als Nextcloud-App

Verworfen.

Grund:

Nextcloud ist nicht für Echtzeit-Audiohardware
optimiert.

---

## Alles als Desktop-Anwendung

Verworfen.

Grund:

Benutzerverwaltung und Zusammenarbeit wären
unnötig kompliziert.

---

## Browser als alleiniger Recorder

Verworfen als Hauptlösung.

Grund:

Nicht ausreichend kontrollierbar für professionelle
Aufnahmen.

---

# Notes

Die Architektur folgt dem Prinzip:

> Das richtige Werkzeug für die richtige Aufgabe.

Der Recorder macht Audio.
Nextcloud macht Zusammenarbeit.
```