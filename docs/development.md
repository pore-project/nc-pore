# NC-PoRe Development Guide

## Version

0.2

## Date

2026-07-22

---

# Purpose

Dieses Dokument beschreibt die grundlegenden Entwicklungsregeln, Arbeitsweisen und die Entwicklungsumgebung für NC-PoRe.

Ziel ist eine nachvollziehbare, wartbare und gemeinschaftsfähige Entwicklung.

---

# Development Principles

NC-PoRe folgt diesen Grundprinzipien:

* Open Source first
* nachvollziehbare Entscheidungen
* kleine, überprüfbare Änderungen
* offene Standards
* saubere Dokumentation
* Qualität vor Geschwindigkeit

---

# Repository Structure

Die grundlegende Struktur:

```
nc-pore/

├── README.md
├── LICENSE
│
├── docs/
│   ├── vision.md
│   ├── requirements.md
│   ├── architecture.md
│   ├── project-status.md
│   ├── mvp.md
│   └── development.md
│
├── adr/
│   └── ADR-xxx-description.md
│
├── recorder/
│   └── Recorder Client Source
│
├── nextcloud-app/
│   └── Nextcloud Application Source
│
└── tests/
    └── Test Resources
```

Die genaue Struktur kann während der Entwicklung angepasst werden.

---

# Branch Strategy

NC-PoRe verwendet ein einfaches Branch-Modell.

## Main Branch

```
main
```

Enthält stabile und getestete Versionen.

Direkte Entwicklung auf `main` soll vermieden werden.

---

## Development Branch

```
develop
```

Enthält aktuelle Entwicklungsstände.

Neue Funktionen und technische Änderungen werden zunächst hier entwickelt.

---

## Feature Branches

Neue Funktionen werden separat entwickelt.

Beispiele:

```
feature/audio-recorder
feature/session-management
feature/export-audacity
```

---

# Commit Guidelines

Commits sollen:

* eine klare Aufgabe beschreiben
* möglichst klein bleiben
* nachvollziehbar sein

Beispiele:

Gut:

```
Add local WAV recorder prototype
```

```
Implement session metadata structure
```

Schlecht:

```
changes
```

```
updates
```

---

# Documentation Rules

Architekturentscheidungen werden als ADR dokumentiert.

Grundlegende Projektinformationen gehören nach:

```
docs/
```

Code-Kommentare erklären:

* warum etwas so gelöst wurde
* nicht nur was der Code macht

---

# Coding Principles

NC-PoRe-Code soll:

* lesbar
* modular
* testbar
* dokumentiert

sein.

Komplexität soll nur entstehen, wenn sie einen echten Nutzen bringt.

---

# Testing Strategy

Tests werden Bestandteil der Entwicklung.

Geplante Ebenen:

## Unit Tests

Einzelne Funktionen.

Beispiele:

* Audioformatprüfung
* Metadatenverarbeitung
* Chunkverwaltung

---

## Integration Tests

Zusammenspiel von Komponenten.

Beispiele:

* Recorder und Upload
* Sessionverwaltung
* Export

---

## Real World Tests

Praktische Tests:

* lange Aufnahmen
* unterschiedliche Hardware
* schlechte Netzwerkbedingungen

---

# Development Environment

Die Entwicklungsumgebung soll bevorzugt auf freien Werkzeugen basieren.

Referenzumgebung:

```
Linux Mint
```

Die Entwicklung erfolgt unter einem separaten Entwicklerkonto:

```
developer
```

Grundanforderungen:

* Git
* Entwicklungseditor
* Build-Werkzeuge
* Testumgebung

Die konkreten Technologien werden in separaten Architekturentscheidungen festgelegt.

---

# Developer Setup

## Repository Access

NC-PoRe verwendet Git zur Versionsverwaltung.

Der Zugriff auf das zentrale Repository erfolgt über SSH.

Repository:

```
git@github.com:pore-project/nc-pore.git
```

Der private SSH-Schlüssel verbleibt ausschließlich auf dem jeweiligen Entwicklungsrechner.

Nur der öffentliche Schlüssel wird beim Repository-Anbieter hinterlegt.

---

## Local Development Directory

Das Repository kann lokal in einem Entwicklungsverzeichnis liegen.

Beispiel:

```
/home/developer/projects/nc-pore
```

---

## Git Configuration

Die lokale Git-Konfiguration verwendet eine Projektidentität.

Beispiel:

```
PoRe Project
```

Private Entwicklerdaten werden nicht in der öffentlichen Projektdokumentation veröffentlicht.

---

## Development Workflow

Vor Änderungen sollte der aktuelle Zustand geprüft werden:

```
git status
```

Der aktive Branch kann geprüft werden mit:

```
git branch --show-current
```

Typischer Ablauf:

```
Änderung
   |
   v
lokaler Test
   |
   v
Commit
   |
   v
Push nach develop
   |
   v
Review / Integration
```

---

# Issue Management

Aufgaben und Fehler werden nachvollziehbar dokumentiert.

Jede größere Änderung sollte eine klare Begründung besitzen.

---

# Release Philosophy

NC-PoRe verwendet nachvollziehbare Versionen.

Beispiel:

```
0.1.x
```

Experimentelle Entwicklungsstände.

```
1.0.0
```

Erste stabile produktive Version.

---

# Contribution Philosophy

Beiträge von außen sind erwünscht.

Voraussetzungen:

* nachvollziehbarer Code
* dokumentierte Änderungen
* Einhaltung der Projektprinzipien

---

# Security Development

Sicherheitsrelevante Änderungen werden besonders behandelt.

Besondere Aufmerksamkeit:

* Zugangsdaten
* Audiodaten
* Uploads
* Berechtigungen

---

# Final Principle

NC-PoRe soll nicht nur funktionieren.

Es soll verständlich, überprüfbar und langfristig weiterentwickelbar sein.
