# ADR-039 Recording Architecture and Capture Boundary

* Status: Accepted
* Date: 2026-07-31
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

---

# Kontext

NC-PoRe wurde ursprünglich mit dem Ziel entwickelt,
lokale Audioaufnahme von verteilten Gesprächspartnern
zu ermöglichen.

Die bisherigen Architekturentscheidungen definieren:

* lokale Aufnahme als zentrales Prinzip
* die Production Session als fachliche Einheit
* den Core als Domain Authority
* Trennung zwischen Domäne und technischen Komponenten
* offene Formate und Interoperabilität
* getrennte Behandlung von Control Synchronization und Media Synchronization

Mit der technischen Umsetzung des Core entsteht die Frage,
wie Aufnahme fachlich und technisch in die Architektur
eingeordnet wird.

Die Audioaufnahme selbst ist keine fachliche Regel.

Sie ist eine technische Fähigkeit eines Clients oder
einer Aufnahme-Komponente.

Daher muss die Grenze zwischen:

* fachlichem Recording-Modell
* technischer Audioaufnahme
* Audiodatenverarbeitung

klar definiert werden.

---

# Entscheidung

NC-PoRe trennt das fachliche Recording-Modell von der
technischen Capture-Implementierung.

Der Core beschreibt:

* dass eine Recording-Einheit existiert
* welchen fachlichen Zustand sie besitzt
* zu welcher Production Session sie gehört
* welche fachlichen Regeln für Recordings gelten

Der Core enthält jedoch keine:

* Audio-Hardware-Ansteuerung
* Mikrofonzugriffe
* Audio-Backend-Logik
* Dateiaufzeichnung
* Echtzeit-Audioverarbeitung

---

# Architectural Principle

Ein Recording ist ein fachliches Produktionsobjekt.

Eine Audioaufnahme ist eine technische Operation.

Diese beiden Konzepte werden getrennt modelliert.

Beispiel:

```text
Domain

Production Session

        |
        |
        v

Recording Entity

        |
        |
        v

Capture Boundary

        |
        |
        v

Audio Capture Implementation
```

Die Domain entscheidet über den fachlichen Zustand.

Die Capture-Komponente erzeugt technische Audiodaten.

---

# Core Responsibility

Der Core ist verantwortlich für:

* Recording-Lebenszyklus
* Beziehung zwischen Session und Recording
* fachliche Validierung
* Recording-Metadaten auf Domänenebene

Der Core entscheidet beispielsweise:

* ob ein Recording zu einer Session gehört
* ob ein Recording abgeschlossen werden kann
* welche fachlichen Zustände erlaubt sind

---

# Capture Responsibility

Die Capture-Komponente ist verantwortlich für:

* Zugriff auf Audioquellen
* Aufnahme von Audiodaten
* technische Pufferung
* Audioformatverarbeitung
* lokale Speicherung während der Aufnahme

Sie entscheidet nicht über:

* Production Session Regeln
* Benutzerrollen
* fachliche Zustände
* Produktionsabläufe

---

# Local Recording Principle

Die Aufnahme folgt weiterhin dem Grundsatz:

```text
Lokal aufnehmen

↓

Daten sichern

↓

Nach Abschluss synchronisieren
```

Eine laufende Audioaufnahme darf nicht von einer
stabilen Netzwerkverbindung abhängig sein.

Die technische Capture-Komponente muss daher lokal
arbeitsfähig sein.

---

# Track Model

NC-PoRe verwendet getrennte Audiospuren als
Grundlage der Aufnahmearchitektur.

Die Capture-Schicht kann mehrere lokale Tracks erzeugen.

Beispiele:

* Host Track
* Guest Track
* weitere Teilnehmer-Tracks

Die genaue technische Speicherung und Synchronisation
der Tracks wird durch spätere Entscheidungen definiert.

---

# Interface Boundary

Die Kommunikation zwischen Core und Capture erfolgt
über definierte Schnittstellen.

Beispiel:

```text
Core

start recording

        |
        v

Recording Interface

        |
        v

Capture Provider

        |
        v

Audio Backend
```

Der Core kennt keine konkrete Audio-Technologie.

---

# Technology Independence

Die Auswahl konkreter Audio-Technologien ist nicht
Bestandteil dieser Entscheidung.

Mögliche spätere Entscheidungen:

* Audio Backend
* Plattformintegration
* Codec-Unterstützung
* Dateiformate
* Echtzeitverarbeitung

Diese Entscheidungen müssen die Architekturgrenzen
respektieren.

---

# Consequences

## Positive Consequences

* fachliche Logik bleibt unabhängig von Audio-Technologie
* Audio-Implementierungen können ausgetauscht werden
* Tests des Core benötigen keine Audio-Hardware
* verschiedene Clients können eigene Capture-Lösungen verwenden
* Architekturgrenzen bleiben nachvollziehbar

---

## Negative Consequences

* zusätzliche Schnittstellen notwendig
* Capture-Integration benötigt eigene Architekturentscheidungen
* technische Umsetzung wird komplexer als eine direkte Audiointegration

Diese Nachteile werden bewusst akzeptiert.

Die langfristige Erweiterbarkeit und Wartbarkeit
überwiegen den zusätzlichen Anfangsaufwand.

---

# Alternatives Considered

## Audio Recording Inside the Core

Nicht gewählt.

Begründung:

Dies würde technische Audioabhängigkeiten in die
fachliche Domäne einführen und die Trennung zwischen
Domain und Infrastruktur verletzen.

---

## Client-Owned Recording Without Domain Model

Nicht gewählt.

Begründung:

Eine reine Client-Implementierung würde fachliche
Recording-Zustände und Produktionsbeziehungen aus
dem zentralen Domänenmodell entfernen.

---

# Relationship to Existing Architecture

Diese Entscheidung erweitert:

* ADR-001 Local Recording
* ADR-002 Audio Format and Track Concept
* ADR-015 Recorder Software Architecture
* ADR-018 Recorder Data Flow and Processing Pipeline
* ADR-019 Recording Session Data Model
* ADR-029 Distributed Recording Architecture
* ADR-033 Core Architecture
* ADR-034 Implementation Architecture
* ADR-035 Domain Lifecycle and State Transition Management
* ADR-038 Core Implementation Structure and Module Organization

Sie konkretisiert die Grenze zwischen fachlichem
Recording-Modell und technischer Aufnahme.

---

# Future Considerations

Weitere Entscheidungen werden separat behandelt:

* konkrete Audio-Backend-Auswahl
* Capture Provider Architektur
* Plattformintegration
* lokale Chunk-Speicherung
* Track-Synchronisation
* Exportformate

Diese Entscheidungen erfolgen erst bei konkretem
technischem Bedarf.

---

# Status

Diese Entscheidung definiert die grundlegende Grenze
zwischen Recording-Domäne und Audio-Capture innerhalb
von NC-PoRe.

Die konkrete technische Aufnahmeimplementierung wird
durch spätere Entscheidungen und Implementierungen
festgelegt.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# Context

NC-PoRe was originally designed to enable local audio
recording for distributed conversations.

Previous architecture decisions define:

* local recording as a central principle
* Production Session as the domain entity
* the Core as Domain Authority
* separation between domain and technical components
* open formats and interoperability
* separation of Control Synchronization and Media Synchronization

With Core implementation underway, the architectural
position of recording needs to be clarified.

Audio capture itself is not a domain rule.

It is a technical capability provided by a client or
capture component.

Therefore the boundary between:

* recording domain model
* technical audio capture
* audio data processing

must be explicitly defined.

---

# Decision

NC-PoRe separates the domain Recording model from
technical capture implementation.

The Core defines:

* existence of recording entities
* recording domain states
* relationship to Production Sessions
* domain rules for recordings

The Core does not contain:

* audio hardware access
* microphone handling
* audio backend logic
* file recording
* realtime audio processing

---

# Architectural Principle

A Recording is a domain production object.

Audio capture is a technical operation.

These concepts are modeled separately.

---

# Status

This decision defines the fundamental boundary between
the Recording domain and audio capture within NC-PoRe.

The concrete technical recording implementation will be
defined through later decisions and implementations.
