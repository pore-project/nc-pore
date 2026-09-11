# NC-PoRe Roadmap

- Version: 1.3
- Date: 2026-09-11

---

<a id="deutsch"></a>

# Deutsch

## Zweck

Diese Roadmap beschreibt den aktuell absehbaren Entwicklungsschwerpunkt von NC-PoRe.

Sie ist eine Orientierung, keine starre Verpflichtung. Technische Erkenntnisse, reale Nutzung und Rückmeldungen können die Reihenfolge und Ausgestaltung ändern.

---

## Entwicklungsprinzip

NC-PoRe wird schrittweise entwickelt.

Jeder Entwicklungsschritt soll einen konkreten Nutzen bieten:

- für Anwender
- für Entwickler
- für die Community

Neue Funktionen werden nicht allein nach technischer Machbarkeit bewertet, sondern nach ihrem tatsächlichen Nutzen und ihrer Auswirkung auf die Gesamtarchitektur.

Dabei gilt:

> Komplexität soll innerhalb des Systems gelöst werden und nicht beim Anwender entstehen.

---

# Aktueller Schwerpunkt – V1

Der aktuelle Schwerpunkt liegt auf einer stabilen, lokalen und nachvollziehbaren Recording-Basis.

Dazu gehören insbesondere:

- Production Sessions als zentrale fachliche Einheit
- lokale Audioaufnahme
- Erhalt der Aufnahmequalität unabhängig von der Kommunikationsverbindung
- Recording-Lifecycle und Synchronisation
- Teilnehmer- und Session-Zustände
- lokale und kontrollierte Persistenz
- Artifact-Erzeugung und Wiederherstellung
- definierte Schnittstellen zwischen Core, Clients und Integrationen
- erste Host-Integration über eine klar abgegrenzte Connector-Architektur

Die technische Architektur und ihre verbindlichen Entscheidungen werden in den ADRs dokumentiert.

---

## Entwicklung nach Bedarf

Weitere Funktionen und Integrationen werden nicht als langfristige Funktionsliste vorab festgeschrieben.

Neue Anforderungen werden anhand ihres konkreten Nutzens, ihrer technischen Auswirkungen und ihrer Vereinbarkeit mit den bestehenden Architekturprinzipien bewertet.

Dabei bleiben insbesondere folgende Grundsätze erhalten:

- lokale Aufnahmen bleiben unabhängig von der Kommunikationspipeline
- der Core bleibt fachliche Autorität
- Integrationen bleiben an klaren Grenzen gekapselt
- offene Daten und Interoperabilität werden bevorzugt
- bestehende Architektur wird erweitert, statt unnötig parallel neu aufgebaut zu werden

---

## Community und Erweiterbarkeit

NC-PoRe soll Erweiterungen durch andere Entwickler ermöglichen.

Dafür werden stabile Schnittstellen und klar abgegrenzte Verantwortlichkeiten bevorzugt.

Die Roadmap beschreibt bewusst keine vollständige Liste möglicher zukünftiger Erweiterungen. Welche Erweiterungen tatsächlich sinnvoll sind, wird anhand realer Anforderungen und technischer Erkenntnisse entschieden.

---

## Grundsatz

Die Roadmap beschreibt den derzeitigen Entwicklungsschwerpunkt.

Die ADRs beschreiben die getroffenen Architekturentscheidungen.

Beides wird getrennt betrachtet:

> Nicht alles, was die Architektur ermöglicht, ist deshalb bereits geplant.

---

<a id="english-version"></a>

# English Version

## Purpose

This roadmap describes the currently foreseeable development focus of NC-PoRe.

It is guidance, not a rigid commitment. Technical findings, real-world use, and feedback may change priorities and implementation details.

---

## Development Principle

NC-PoRe is developed step by step.

Each development step should provide concrete value:

- for users
- for developers
- for the community

Features are evaluated not only by technical feasibility, but also by their actual value and their impact on the overall architecture.

The guiding principle is:

> Complexity should be solved inside the system, not transferred to the user.

---

# Current Focus – V1

The current focus is a stable, local, and traceable recording foundation.

This includes in particular:

- Production Sessions as the central domain entity
- local audio recording
- preservation of recording quality independently of the communication connection
- recording lifecycle and synchronization
- participant and session states
- local and controlled persistence
- artifact creation and recovery
- defined interfaces between Core, clients, and integrations
- an initial host integration through a clearly separated connector architecture

The technical architecture and its binding decisions are documented in the ADRs.

---

## Development Based on Need

Further functions and integrations are deliberately not specified as a complete long-term feature list in advance.

New requirements are evaluated according to their concrete value, technical impact, and compatibility with the existing architectural principles.

The following principles remain important:

- local recordings remain independent from the communication pipeline
- the Core remains the domain authority
- integrations remain encapsulated behind clear boundaries
- open data and interoperability are preferred
- existing architecture is extended rather than unnecessarily duplicated

---

## Community and Extensibility

NC-PoRE is intended to allow extensions by other developers.

Stable interfaces and clearly separated responsibilities are therefore preferred.

The roadmap deliberately does not provide a complete list of possible future extensions. Which extensions are actually useful will be decided based on real requirements and technical findings.

---

## Principle

The roadmap describes the current development focus.

The ADRs describe architectural decisions that have actually been made.

They are considered separately:

> Not everything the architecture makes possible is therefore already planned.
