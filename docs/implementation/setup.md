# NC-PoRe Technical Foundation

## Deutsch ([English version below](#english-version))

Dieses Dokument fasst die technische Grundlage für die Implementierung von NC-PoRe zusammen. Verbindliche Architekturentscheidungen werden ausschließlich über ADRs dokumentiert.

## Technische Grundidee

NC-PoRe verbindet lokale Aufnahme und Verarbeitung mit einer zentralen Umgebung für fachliche Verwaltung und kontrollierte Synchronisation.

Die Aufnahme ist dabei von einer permanenten Netzwerkverbindung unabhängig. Provider- und Host-spezifische Details bleiben an den dafür vorgesehenen Integrationsgrenzen.

## Technische Hauptbereiche

- **Client / Recorder:** lokale Aufnahme, lokale Zustände und lokale Artefaktverarbeitung
- **Core:** fachliche Modelle, Geschäftsregeln, Zustände und Domain-Operationen
- **Application:** Anwendungsgrenzen und die Vermittlung zwischen Domäne und Infrastruktur
- **Infrastructure:** Persistenz, Synchronisation und externe Integrationen
- **Nextcloud App:** Host-Integration und Bereitstellung der Nextcloud-spezifischen Schnittstellen

## Festgelegte technische Richtungen

Die verbindlichen technischen Richtungen ergeben sich aus den ADRs. Dazu gehören insbesondere:

- lokale Aufnahme als Grundprinzip
- Trennung von Aufnahme und Kommunikationspipeline
- Production Session als fachliche Einheit
- klare Core- und Integrationsgrenzen
- kontrollierte lokale Persistenz und Recovery
- getrennte Control- und Media-Synchronisation
- host-spezifische Connectoren an der Integrationsgrenze
- offene und nachvollziehbare Datenrepräsentationen

## Aktuelle Implementierungsstruktur

Die Repository-Struktur folgt der tatsächlichen Aufteilung des Projekts, unter anderem:

```text
nc-pore/
├── adr/
├── application/
├── core/
├── recorder/
├── runtime/
├── infrastructure/
├── web/
├── lib/
├── js/
├── css/
├── appinfo/
├── docs/
└── tools/
```

Die genaue Modulstruktur ist durch den Quellcode und die jeweils einschlägigen ADRs bestimmt.

## Entwicklungsprinzipien

Technische Arbeit erfolgt schrittweise und überprüfbar.

- Änderungen werden auf eine klar abgegrenzte Aufgabe zugeschnitten.
- Vor einer Änderung wird der tatsächliche Mechanismus geprüft.
- Architekturänderungen werden über ADRs nachvollziehbar dokumentiert.
- Tests sind Bestandteil der Entwicklung.
- Unnötige parallele Implementierungen werden vermieden.

## Vertikaler Entwicklungsansatz

Neue Funktionen werden bevorzugt als kleine, vollständige vertikale Schritte umgesetzt. Dabei sollen fachliche Domäne, Anwendungsschnittstelle, technische Infrastruktur und Tests nur soweit erweitert werden, wie es der konkrete Schritt erfordert.

## Grundsatz

NC-PoRe wird nicht durch maximale technische Komplexität definiert.

Technische Qualität entsteht durch klare Grenzen, nachvollziehbare Entscheidungen, verständliche Komponenten und überprüfbare Implementierungen.

---

# English Version

This document summarizes the technical foundation for implementing NC-PoRe. Binding architectural decisions are documented exclusively through ADRs.

## Technical Concept

NC-PoRe combines local recording and processing with a central environment for domain management and controlled synchronization.

Recording is independent of a permanent network connection. Provider- and host-specific details remain at their designated integration boundaries.

## Main Technical Areas

- **Client / Recorder:** local recording, local state and local artifact processing
- **Core:** domain models, business rules, states and domain operations
- **Application:** application boundaries and coordination between domain and infrastructure
- **Infrastructure:** persistence, synchronization and external integrations
- **Nextcloud App:** host integration and Nextcloud-specific interfaces

## Established Technical Directions

Binding technical directions are defined by the ADRs. They include in particular:

- local recording as a core principle
- separation of recording from the communication pipeline
- Production Session as a domain entity
- clear Core and integration boundaries
- controlled local persistence and recovery
- separation of control and media synchronization
- host-specific connectors at the integration boundary
- open and traceable data representations

## Current Implementation Structure

The repository structure follows the actual project decomposition, including:

```text
nc-pore/
├── adr/
├── application/
├── core/
├── recorder/
├── runtime/
├── infrastructure/
├── web/
├── lib/
├── js/
├── css/
├── appinfo/
├── docs/
└── tools/
```

The exact module structure is defined by the source tree and the applicable ADRs.

## Development Principles

Technical work proceeds incrementally and with verification.

- Changes are scoped to a clearly defined task.
- The actual mechanism is inspected before making changes.
- Architectural changes are documented through ADRs.
- Tests are part of development.
- Unnecessary parallel implementations are avoided.

## Vertical Development Approach

New functionality is preferably implemented as small, complete vertical slices. Domain logic, application interfaces, infrastructure and tests are extended only as far as the concrete step requires.

## Principle

NC-PoRe is not defined by maximum technical complexity.

Technical quality comes from clear boundaries, traceable decisions, understandable components and verifiable implementations.
