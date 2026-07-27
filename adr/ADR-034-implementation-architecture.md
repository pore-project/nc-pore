# ADR-034 Implementation Architecture

- Status: Proposed
- Date: 2026-07-24

---

# Deutsch ([English version below](#english-version))

---

# Context

Die bisherigen Architekturentscheidungen definieren die fachlichen
Grundlagen von NC-PoRe.

Sie beschreiben insbesondere:

- die zentralen Domänenkonzepte
- die Verantwortlichkeiten der Systemkomponenten
- die fachlichen Regeln
- den Core als Domain Authority
- die Kommunikationsprinzipien
- die grundlegende Systemarchitektur

Mit Abschluss dieser Architekturphase beginnt die Vorbereitung
der technischen Umsetzung.

Die Implementierungsarchitektur soll die bestehenden fachlichen
Entscheidungen abbilden und nicht durch technische Entscheidungen
ersetzen.

---

# Decision

Die Implementierungsarchitektur folgt der fachlichen Architektur.

Fachliche Grenzen werden in der Software nicht aufgehoben,
sondern bewusst abgebildet.

Technische Entscheidungen müssen die bestehende Architektur
unterstützen und dürfen ihre fachlichen Verantwortlichkeiten
nicht verändern.

---

# Implementation Principles

## Domain before Technology

Die Domäne bestimmt die Struktur der Software.

Technologien, Frameworks und Bibliotheken sind Werkzeuge zur
Umsetzung der fachlichen Architektur.

Sie bestimmen nicht die Architektur selbst.

---

## Clear Module Boundaries

Jedes Modul besitzt eine klar definierte Verantwortung.

Module kommunizieren ausschließlich über dokumentierte
Schnittstellen.

Zyklische Abhängigkeiten zwischen Modulen sollen vermieden werden.

---

## Dependencies Point Inward

Technische Komponenten dürfen von der Domäne abhängen.

Die Domäne darf jedoch nicht von technischen Implementierungen
abhängig werden.

Abhängigkeiten sollen von technischen Details zur fachlichen
Logik zeigen, nicht umgekehrt.

Fachliche Modelle bleiben unabhängig von Infrastruktur,
Speicherung und Benutzeroberflächen.

---

## Replaceable Technologies

Technologien sollen grundsätzlich austauschbar bleiben.

Die Auswahl konkreter Frameworks, Datenbanken oder
Kommunikationsprotokolle darf möglichst geringe Auswirkungen
auf die Domänenlogik haben.

---

# Consequences

Die Implementierung orientiert sich an den fachlichen Grenzen
der Architektur.

Neue technische Entscheidungen werden daran gemessen,
ob sie diese Struktur unterstützen.

Technologieentscheidungen werden in separaten ADRs dokumentiert,
wenn sie langfristige Auswirkungen auf die Architektur besitzen.

---

# Alternatives Considered

## Technology-first Architecture

Nicht gewählt.

Begründung:

Eine technologiegetriebene Struktur würde die fachliche
Architektur langfristig schwächen und spätere Änderungen
erschweren.

---

## Monolithic Technical Structure

Nicht gewählt.

Begründung:

Eine Vermischung fachlicher und technischer Verantwortlichkeiten
würde Wartbarkeit, Erweiterbarkeit und Testbarkeit verschlechtern.

---

# Status

Diese Entscheidung definiert die grundlegenden Prinzipien
für die technische Umsetzung von NC-PoRe.

Sie beschreibt keine konkrete Implementierung,
sondern den Rahmen für zukünftige technische Entscheidungen.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# Context

The previous architecture decisions define the domain foundations
of NC-PoRe.

They describe in particular:

- the central domain concepts
- the responsibilities of the system components
- the domain rules
- the Core as the Domain Authority
- the communication principles
- the overall system architecture

With the completion of the architecture phase,
preparation for implementation begins.

The implementation architecture shall reflect the existing
domain architecture rather than replacing it with technical
decisions.

---

# Decision

The implementation architecture follows the domain architecture.

Domain boundaries are intentionally reflected in the software
instead of being dissolved by technical implementation.

Technical decisions shall support the existing architecture
without changing its domain responsibilities.

---

# Implementation Principles

## Domain before Technology

The domain defines the structure of the software.

Technologies, frameworks and libraries are implementation tools.

They do not define the architecture.

---

## Clear Module Boundaries

Every module has a clearly defined responsibility.

Modules communicate exclusively through documented interfaces.

Cyclic dependencies between modules should be avoided.

---

## Dependencies Point Inward

Technical components may depend on the domain.

The domain must not depend on technical implementations.

Dependencies should point from technical details toward
domain logic, not the other way around.

Domain models remain independent from infrastructure,
storage and user interfaces.

---

## Replaceable Technologies

Technologies should remain replaceable whenever possible.

The selection of frameworks, databases or communication
protocols should have minimal impact on the domain logic.

---

# Consequences

Implementation follows the domain boundaries defined
by the architecture.

Future technical decisions are evaluated by how well
they support this structure.

Technology choices with long-term architectural impact
are documented in separate ADRs.

---

# Alternatives Considered

## Technology-first Architecture

Rejected.

Reason:

A technology-driven structure would weaken the domain
architecture and make future changes more difficult.

---

## Monolithic Technical Structure

Rejected.

Reason:

Mixing domain and technical responsibilities would reduce
maintainability, extensibility and testability.

---

# Status

This decision defines the fundamental principles
for implementing NC-PoRe.

It does not describe a concrete implementation,
but establishes the framework for future technical decisions.
