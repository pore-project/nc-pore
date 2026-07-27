# NC-PoRe Technical Decisions

* Version: 1.0
* Date: 2026-07-24

---

# Deutsch ([English version below](#english-version))

---

# 1. Zweck dieses Dokuments

Dieses Dokument beschreibt die offenen technischen Entscheidungen,
die für die Umsetzung von NC-PoRe getroffen werden müssen.

Es dient als Übersicht und Entscheidungslandkarte.

Es ersetzt keine Architecture Decision Records (ADRs).

Sobald eine technische Entscheidung langfristige Auswirkungen auf
die Architektur hat, wird sie durch ein eigenes ADR dokumentiert.

---

# 2. Grundprinzip

Technische Entscheidungen werden nicht isoliert getroffen.

Jede Entscheidung muss die bestehenden Grundlagen berücksichtigen:

* Projektvision
* Architekturprinzipien
* Systemkomponenten
* MVP-Definition
* langfristige Wartbarkeit

Die wichtigste Frage lautet:

> Welche Lösung unterstützt NC-PoRe langfristig am besten?

Nicht:

> Welche Technologie ist aktuell am beliebtesten?

---

# 3. Offene technische Entscheidungen

---

# 3.1 Core-Technologie

## Fragestellung

Welche Technologie bildet die Grundlage für den NC-PoRe Core?

## Zu bewerten:

* Unterstützung komplexer Geschäftslogik
* Testbarkeit
* Wartbarkeit
* Stabilität
* Schnittstellenfähigkeit
* langfristige Verfügbarkeit

## Abhängigkeiten:

* Core-Verantwortlichkeiten
* API-Architektur
* Datenmodell

---

# 3.2 Client-Technologie

## Fragestellung

Welche Technologie ermöglicht plattformübergreifende lokale Produktionsclients?

## Zu bewerten:

* Audio-Hardwarezugriff
* Performance
* Plattformunterstützung
* Benutzererfahrung
* Wartbarkeit

## Abhängigkeiten:

* Local Recording First
* Audio-Architektur
* Zielplattformen

---

# 3.3 Audio-Technologie

## Fragestellung

Welche technischen Komponenten werden für professionelle Audioaufnahme benötigt?

## Zu bewerten:

* Aufnahmequalität
* Latenz
* Mehrspurunterstützung
* Plattformunterstützung
* offene Formate

## Abhängigkeiten:

* Client-Technologie
* Audioformatstrategie

---

# 3.4 Datenhaltung und Persistenz

## Fragestellung

Wie werden fachliche Daten und Metadaten gespeichert?

## Zu bewerten:

* Datenmodell
* Migrationen
* Konsistenz
* Backupfähigkeit
* Skalierbarkeit

## Abhängigkeiten:

* Production Session Modell
* Storage Layer

---

# 3.5 API-Architektur

## Fragestellung

Wie kommunizieren Client und Core?

## Zu bewerten:

* API-Stil
* Versionierung
* Fehlerbehandlung
* Authentifizierung
* Erweiterbarkeit

## Abhängigkeiten:

* Core
* Client
* Synchronisation

---

# 3.6 Event- und Synchronisationsmodell

## Fragestellung

Welche Ereignisse und Zustandsänderungen müssen zwischen Komponenten ausgetauscht werden?

## Zu bewerten:

* Nachvollziehbarkeit
* Offline-Fähigkeit
* Wiederholbarkeit
* Konsistenz

## Abhängigkeiten:

* ADR-030 Synchronization Strategy
* Activity History

---

# 3.7 Build- und Entwicklungsumgebung

## Fragestellung

Welche Werkzeuge unterstützen nachhaltige Entwicklung?

## Zu bewerten:

* Reproduzierbarkeit
* Entwicklerfreundlichkeit
* Automatisierung
* Plattformunterstützung

## Abhängigkeiten:

* Repository-Struktur
* CI/CD

---

# 3.8 Teststrategie

## Fragestellung

Wie wird Qualität langfristig sichergestellt?

## Zu bewerten:

* Unit Tests
* Integration Tests
* System Tests
* Audio Tests
* automatisierte Prüfungen

## Abhängigkeiten:

* Architektur
* Komponentenstruktur

---

# 3.9 Deployment und Betrieb

## Fragestellung

Wie wird NC-PoRe betrieben und verteilt?

## Zu bewerten:

* Selbsthosting
* Updates
* Konfiguration
* Monitoring
* Sicherheit

## Abhängigkeiten:

* Nextcloud-Integration
* Storage
* Core

---

# 3.10 Packaging und Veröffentlichung

## Fragestellung

Wie werden Komponenten verteilt?

## Zu bewerten:

* Installationsprozess
* Versionierung
* Releases
* Dokumentation

## Abhängigkeiten:

* Release-Strategie
* Plattformen

---

# 4. Reihenfolge der Entscheidungen

Nicht alle Entscheidungen müssen gleichzeitig getroffen werden.

Empfohlene Reihenfolge:

```text
System Components
        |
        v
Core Architecture
        |
        v
Data Model
        |
        v
API Design
        |
        v
Client Architecture
        |
        v
Implementation Technologies
```

Die Reihenfolge reduziert unnötige technische Festlegungen.

---

# 5. Entscheidungsprinzip

NC-PoRe bevorzugt:

* nachvollziehbare Entscheidungen
* offene Technologien
* langfristige Stabilität
* geringe unnötige Komplexität

Technische Entscheidungen sollen dokumentiert werden.

Nicht nur das Ergebnis ist wichtig.

Auch der Entscheidungsweg ist Teil der Softwarequalität.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# 1. Purpose of this Document

This document describes the open technical decisions
that need to be made for the implementation of NC-PoRe.

It serves as an overview and decision map.

It does not replace Architecture Decision Records (ADRs).

When a technical decision has long-term architectural impact,
it will be documented through a dedicated ADR.

---

# 2. Fundamental Principle

Technical decisions are not made in isolation.

Every decision must consider:

* project vision
* architecture principles
* system components
* MVP definition
* long-term maintainability

The most important question is:

> Which solution supports NC-PoRe best in the long term?

Not:

> Which technology is currently most popular?

---

# 3. Open Technical Decisions

---

# 3.1 Core Technology

## Question

Which technology forms the foundation of the NC-PoRe Core?

## Evaluation:

* support for complex business logic
* testability
* maintainability
* stability
* interface capabilities
* long-term availability

## Dependencies:

* Core responsibilities
* API architecture
* data model

---

# 3.2 Client Technology

## Question

Which technology enables cross-platform local production clients?

## Evaluation:

* audio hardware access
* performance
* platform support
* user experience
* maintainability

## Dependencies:

* Local Recording First
* audio architecture
* target platforms

---

# 3.3 Audio Technology

## Question

Which technical components are required for professional audio recording?

## Evaluation:

* recording quality
* latency
* multi-track support
* platform support
* open formats

## Dependencies:

* client technology
* audio format strategy

---

# 3.4 Data Storage and Persistence

## Question

How are domain data and metadata stored?

## Evaluation:

* data model
* migrations
* consistency
* backup capability
* scalability

## Dependencies:

* Production Session model
* Storage Layer

---

# 3.5 API Architecture

## Question

How do Client and Core communicate?

## Evaluation:

* API style
* versioning
* error handling
* authentication
* extensibility

## Dependencies:

* Core
* Client
* synchronization

---

# 3.6 Event and Synchronization Model

## Question

Which events and state changes must be exchanged between components?

## Evaluation:

* traceability
* offline capability
* repeatability
* consistency

## Dependencies:

* ADR-030 Synchronization Strategy
* Activity History

---

# 3.7 Build and Development Environment

## Question

Which tools support sustainable development?

## Evaluation:

* reproducibility
* developer experience
* automation
* platform support

## Dependencies:

* repository structure
* CI/CD

---

# 3.8 Testing Strategy

## Question

How is long-term quality ensured?

## Evaluation:

* unit tests
* integration tests
* system tests
* audio tests
* automated validation

## Dependencies:

* architecture
* component structure

---

# 3.9 Deployment and Operations

## Question

How is NC-PoRe operated and distributed?

## Evaluation:

* self-hosting
* updates
* configuration
* monitoring
* security

## Dependencies:

* Nextcloud integration
* storage
* Core

---

# 3.10 Packaging and Release

## Question

How are components distributed?

## Evaluation:

* installation process
* versioning
* releases
* documentation

## Dependencies:

* release strategy
* platforms

---

# 4. Decision Order

Not all decisions need to be made at the same time.

Recommended order:

```text
System Components
        |
        v
Core Architecture
        |
        v
Data Model
        |
        v
API Design
        |
        v
Client Architecture
        |
        v
Implementation Technologies
```

This order reduces unnecessary technical commitments.

---

# 5. Decision Principle

NC-PoRe prefers:

* traceable decisions
* open technologies
* long-term stability
* low unnecessary complexity

Technical decisions should be documented.

The result matters.

The decision process itself is also part of software quality.

