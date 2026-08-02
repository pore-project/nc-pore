# Recorder Architecture Foundation

* Date: 2026-07-31
* Milestone Type: Recorder Architecture Foundation
* Status: Completed

---

# Deutsch ([English version below](#english-version))

---

# Zweck

Dieser Milestone dokumentiert die technische Grundlage des lokalen Recorder-Workflows von NC-PoRe.

Nach Abschluss der ersten Core-Implementierung wurde die Recorder-Architektur um die notwendigen technischen Grenzen und Modelle erweitert.

Ziel dieses Schrittes war nicht die konkrete Audioaufnahme, sondern die Vorbereitung einer stabilen Architektur für zukünftige Aufnahmeimplementierungen.

---

# Erreichte Ergebnisse

## Capture Boundary

Definiert:

* technische Grenze zwischen Recorder Workflow und Audio Capture
* Abstraktion konkreter Aufnahmeimplementierungen
* Möglichkeit für austauschbare Capture Provider

Die Recorder-Logik bleibt unabhängig von konkreter Audio-Hardware und Aufnahmebibliotheken.

---

## Recorder Workflow Architecture

Implementiert:

* Workflow Coordination Layer
* technische Koordination zwischen Session, Capture und Artifact
* Trennung zwischen Workflow-Steuerung und Medienverarbeitung

Der Workflow koordiniert technische Abläufe, enthält jedoch keine konkrete Audioverarbeitung.

---

## Recording Artifact Model

Definiert:

* Recording Artifact als eigenständiges technisches Modell
* Trennung zwischen fachlichen Domainobjekten und technischen Aufnahmeergebnissen
* eigener technischer Lifecycle

Ein Recording Artifact besitzt einen eigenen Lebenszyklus unabhängig vom fachlichen Session Lifecycle.

---

## Artifact Lifecycle Management

Implementiert:

* Erstellung von Recording Artifacts
* Lifecycle-Übergänge
* technische Zustandsverwaltung

Die Lifecycle-Verwaltung bleibt unabhängig von konkreten Speicher- oder Aufnahmeimplementierungen.

---

# Architekturprinzipien

Mit diesem Milestone wurden folgende Prinzipien technisch vorbereitet:

* lokale Aufnahme ohne Abhängigkeit von Netzwerkverfügbarkeit
* Trennung zwischen Domain und technischen Artefakten
* technische Austauschbarkeit
* explizite Lebenszyklen
* klare Verantwortungsgrenzen

---

# Relevante ADRs

* ADR-039 Recording Architecture and Capture Boundary
* ADR-040 Recorder Workflow and Capture Lifecycle Coordination
* ADR-041 Local Recording Artifact and Storage Boundary
* ADR-042 Recording Artifact Model and Lifecycle Boundary

---

# Bedeutung für die weitere Entwicklung

Dieser Milestone bildet die Grundlage für:

```text
Recorder Workflow

↓

Recording Artifact

↓

Persistence Boundary

↓

Local Storage
```

Die konkrete Audioaufnahme und die konkrete Speicherung bleiben nachfolgende Implementierungsschritte.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# Purpose

This milestone documents the technical foundation of the NC-PoRe local recorder workflow.

After completing the first Core implementation, the Recorder architecture was extended with the required technical boundaries and models.

The goal of this step was not implementing concrete audio capture, but establishing a stable architecture for future recording implementations.

---

# Achieved Results

## Capture Boundary

Defined:

* technical boundary between Recorder Workflow and Audio Capture
* abstraction of concrete recording implementations
* support for replaceable Capture Providers

The Recorder logic remains independent from concrete audio hardware and recording libraries.

---

## Recorder Workflow Architecture

Implemented:

* Workflow Coordination Layer
* technical coordination between Session, Capture and Artifact
* separation between workflow control and media processing

The workflow coordinates technical processes but does not contain concrete audio processing.

---

## Recording Artifact Model

Defined:

* Recording Artifact as an independent technical model
* separation between domain objects and technical recording results
* dedicated technical lifecycle

A Recording Artifact has its own lifecycle independent from the domain Session lifecycle.

---

## Artifact Lifecycle Management

Implemented:

* creation of Recording Artifacts
* lifecycle transitions
* technical state management

Lifecycle management remains independent from concrete storage or recording implementations.

---

# Architecture Principles

This milestone technically prepares the following principles:

* local recording without dependency on network availability
* separation between domain and technical artifacts
* technical replaceability
* explicit lifecycles
* clear responsibility boundaries

---

# Related ADRs

* ADR-039 Recording Architecture and Capture Boundary
* ADR-040 Recorder Workflow and Capture Lifecycle Coordination
* ADR-041 Local Recording Artifact and Storage Boundary
* ADR-042 Recording Artifact Model and Lifecycle Boundary

---

# Importance for Further Development

This milestone establishes the foundation for:

```text
Recorder Workflow

↓

Recording Artifact

↓

Persistence Boundary

↓

Local Storage
```

Concrete audio capture and concrete storage remain future implementation steps.
