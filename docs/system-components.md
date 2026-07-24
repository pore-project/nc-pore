# NC-PoRe System Components

* Version: 1.0
* Date: 2026-07-24

---

# Deutsch ([English version below](#english-version))

---

# 1. Zweck dieses Dokuments

Dieses Dokument beschreibt die zentralen Systemkomponenten von NC-PoRe
und deren Verantwortlichkeiten.

Es definiert keine konkrete Softwarestruktur und keine Technologieauswahl.

Ziel ist es, die fachlichen und technischen Verantwortlichkeiten klar
zu trennen, bevor konkrete Implementierungsentscheidungen getroffen werden.

Technologieentscheidungen werden auf Basis dieser Struktur bewertet und
bei langfristiger architektonischer Bedeutung über ADRs dokumentiert.

---

# 2. Systemübersicht

NC-PoRe basiert auf einer verteilten Systemarchitektur.

Die zentralen Bereiche sind:

```text
NC-PoRe

+---------------------------+
| Client                    |
| Local Production          |
+-------------+-------------+
              |
              |
+-------------v-------------+
| Core                      |
| Domain Authority          |
+-------------+-------------+
              |
              |
+-------------v-------------+
| Storage Layer             |
| Data and Assets           |
+-------------+-------------+
              |
              |
+-------------v-------------+
| External Integrations     |
| Nextcloud and Services    |
+---------------------------+
```

Die Komponenten haben klar getrennte Verantwortlichkeiten.

---

# 3. Client

## Verantwortung

Der Client ermöglicht lokale Produktionsabläufe.

Der Client ist verantwortlich für:

* Audioaufnahme
* lokale Verarbeitung
* lokale Zwischenspeicherung
* Benutzerinteraktion
* Anzeige von Produktionsinformationen
* Kommunikation mit dem Core

---

## Der Client ist nicht verantwortlich für

Der Client entscheidet nicht über:

* globale Geschäftsregeln
* endgültige Berechtigungen
* zentrale Datenintegrität
* fachliche Wahrheit einer Production Session

Diese Verantwortung liegt beim Core.

---

## Grundprinzip

Der Client ermöglicht Produktion.

Der Core verwaltet die Bedeutung der Produktion.

---

# 4. Core

## Verantwortung

Der Core ist die fachliche Autorität von NC-PoRe.

Der Core verwaltet:

* Production Sessions
* Teilnehmer
* Rollen
* Zustände
* Berechtigungen
* fachliche Regeln

---

## Der Core ist nicht verantwortlich für

Der Core übernimmt nicht:

* direkte Hardwaresteuerung
* lokale Benutzerinteraktion
* konkrete Audioaufnahme

Diese Aufgaben bleiben beim Client.

---

## Grundprinzip

Der Core entscheidet, was fachlich gültig ist.

---

# 5. Storage Layer

## Verantwortung

Der Storage Layer verwaltet dauerhafte Speicherung.

Verantwortlichkeiten:

* Audio Assets
* Produktionsdaten
* Metadaten
* Session-Daten
* Versionen und Zustände

---

## Der Storage Layer ist nicht verantwortlich für

Der Storage entscheidet nicht über:

* Geschäftsregeln
* Berechtigungen
* Produktionsabläufe

Speicherung und Bedeutung bleiben getrennt.

---

# 6. External Integrations

## Verantwortung

Externe Integrationen verbinden NC-PoRe mit anderen Systemen.

Beispiele:

* Nextcloud
* weitere Speicheranbieter
* externe Dienste

---

## Grundprinzip

Integrationen erweitern NC-PoRe.

Sie definieren nicht die Kernlogik.

---

# 7. Kommunikation zwischen Komponenten

Die Kommunikation erfolgt über klar definierte Schnittstellen.

Grundprinzipien:

* API-basierte Kommunikation
* nachvollziehbare Datenflüsse
* getrennte Steuerungs- und Mediendaten
* dokumentierte Schnittstellen

---

# 8. Datenverantwortung

NC-PoRe unterscheidet zwischen:

## Fachlichen Daten

Beispiele:

* Production Session
* Rollen
* Teilnehmer
* Produktionsstatus

Verantwortlich:

Core

---

## Medien-Daten

Beispiele:

* Audioaufnahmen
* Assets
* Exportdateien

Verantwortlich:

Storage Layer in Zusammenarbeit mit Clients und Core.

---

## Lokalen Daten

Beispiele:

* temporäre Aufnahmen
* lokale Zwischenspeicherung
* Offline-Zustände

Verantwortlich:

Client

---

# 9. MVP-Komponenten

Für das erste MVP werden folgende Komponenten benötigt:

## Client

* lokale Aufnahme
* Session-Anbindung
* Upload-Funktion

## Core

* Session-Verwaltung
* einfache Rollenverwaltung
* zentrale Zustände

## Storage

* Speicherung von Audio und Metadaten

## Communication

* grundlegende API-Kommunikation

Nicht Bestandteil des ersten MVP:

* vollständige Integrationslandschaft
* komplexe Automatisierung
* maximale Skalierung

---

# 10. Architekturprinzip

NC-PoRe trennt bewusst:

* Benutzeroberfläche
* lokale Produktion
* fachliche Logik
* Speicherung
* Integrationen

Diese Trennung ermöglicht:

* Erweiterbarkeit
* bessere Wartbarkeit
* unabhängige Weiterentwicklung

Technische Komponenten sollen klare Aufgaben erfüllen.

Keine Komponente soll mehr Verantwortung übernehmen als notwendig.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# 1. Purpose of this Document

This document describes the central system components of NC-PoRe
and their responsibilities.

It does not define a concrete software structure or technology choice.

The goal is to clearly separate technical and domain responsibilities
before implementation decisions are made.

Technology decisions are evaluated based on this structure and documented
through ADRs when they have long-term architectural impact.

---

# 2. System Overview

NC-PoRe is based on a distributed system architecture.

The central areas are:

```text
NC-PoRe

+---------------------------+
| Client                    |
| Local Production          |
+-------------+-------------+
              |
              |
+-------------v-------------+
| Core                      |
| Domain Authority          |
+-------------+-------------+
              |
              |
+-------------v-------------+
| Storage Layer             |
| Data and Assets           |
+-------------+-------------+
              |
              |
+-------------v-------------+
| External Integrations     |
| Nextcloud and Services    |
+---------------------------+
```

The components have clearly separated responsibilities.

---

# 3. Client

## Responsibility

The Client enables local production workflows.

The Client is responsible for:

* audio recording
* local processing
* local temporary storage
* user interaction
* displaying production information
* communication with the Core

---

## The Client is not responsible for

The Client does not decide:

* global business rules
* final permissions
* central data integrity
* domain truth of a Production Session

These responsibilities belong to the Core.

---

## Principle

The Client enables production.

The Core manages the meaning of production.

---

# 4. Core

## Responsibility

The Core is the domain authority of NC-PoRe.

The Core manages:

* Production Sessions
* participants
* roles
* states
* permissions
* domain rules

---

## The Core is not responsible for

The Core does not handle:

* direct hardware control
* local user interaction
* actual audio recording

These tasks remain with the Client.

---

## Principle

The Core decides what is valid from a domain perspective.

---

# 5. Storage Layer

## Responsibility

The Storage Layer manages persistent storage.

Responsibilities:

* audio assets
* production data
* metadata
* session data
* versions and states

---

## The Storage Layer is not responsible for

The Storage Layer does not decide:

* business rules
* permissions
* production workflows

Storage and meaning remain separated.

---

# 6. External Integrations

## Responsibility

External integrations connect NC-PoRe with other systems.

Examples:

* Nextcloud
* additional storage providers
* external services

---

## Principle

Integrations extend NC-PoRe.

They do not define the core logic.

---

# 7. Communication Between Components

Communication happens through clearly defined interfaces.

Principles:

* API-based communication
* traceable data flows
* separation of control and media data
* documented interfaces

---

# 8. Data Responsibility

NC-PoRe distinguishes between:

## Domain Data

Examples:

* Production Session
* roles
* participants
* production status

Responsible:

Core

---

## Media Data

Examples:

* audio recordings
* assets
* export files

Responsible:

Storage Layer together with Clients and Core.

---

## Local Data

Examples:

* temporary recordings
* local caching
* offline states

Responsible:

Client

---

# 9. MVP Components

The first MVP requires:

## Client

* local recording
* session connection
* upload functionality

## Core

* session management
* basic role management
* central states

## Storage

* storage of audio and metadata

## Communication

* basic API communication

Not part of the first MVP:

* complete integration landscape
* complex automation
* maximum scalability

---

# 10. Architecture Principle

NC-PoRe deliberately separates:

* user interface
* local production
* domain logic
* storage
* integrations

This separation enables:

* extensibility
* better maintainability
* independent evolution

Technical components should have clear responsibilities.

No component should take on more responsibility than necessary.
