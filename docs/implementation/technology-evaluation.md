# NC-PoRe Technology Evaluation

## Deutsch ([English version below](#english-version))

Dieses Dokument beschreibt die allgemeinen Kriterien für die Bewertung technischer Optionen im Projekt.

Es ersetzt keine Architecture Decision Records (ADRs) und enthält keine verbindliche Technologie-Roadmap. Eine konkrete Technologieentscheidung wird, sobald sie architektonische Bedeutung hat, im jeweiligen ADR dokumentiert.

---

# 1. Bewertungsprinzipien

NC-PoRe bewertet technische Optionen anhand der tatsächlichen Anforderungen des jeweiligen Problems.

Eine geeignete Lösung soll insbesondere berücksichtigen:

- Wartbarkeit
- Stabilität
- Offenheit und Interoperabilität
- Sicherheit
- Testbarkeit
- angemessenen Entwicklungs- und Betriebsaufwand
- Verhältnismäßigkeit der Komplexität

Technische Entscheidungen sollen nicht allein aufgrund von Popularität, Marketing oder kurzfristigen Trends getroffen werden.

---

# 2. Bewertungskriterien

## Wartbarkeit

- Verständlichkeit
- Verfügbarkeit von Wissen und Werkzeugen
- Fehleranalyse
- langfristige Pflegefähigkeit

## Offenheit

- geeignete Lizenzen
- offene Standards
- klare Schnittstellen
- Interoperabilität

## Technische Eignung

- Erfüllung der fachlichen Anforderungen
- Performance
- Stabilität
- Integrationsfähigkeit
- Beherrschbarkeit der Komplexität

## Sicherheit

- Sicherheitsmodell
- Umgang mit sensiblen Daten
- Update- und Wartbarkeit
- nachvollziehbare Vertrauensgrenzen

## Entwicklungsaufwand

- verfügbare Werkzeuge
- Testbarkeit
- Implementierungsaufwand
- Betriebs- und Wartungsaufwand

---

# 3. Entscheidungsprozess

Technische Optionen werden schrittweise bewertet:

1. konkrete Anforderung bestimmen
2. relevante Optionen identifizieren
3. technische und fachliche Kriterien vergleichen
4. Risiken und Grenzen prüfen
5. Entscheidung dokumentieren
6. bei architektonischer Bedeutung einen ADR erstellen oder einen bestehenden ADR fortschreiben

Die konkrete Auswahl ist immer kontextabhängig. Dieses Dokument legt keine bestimmten Technologien oder Plattformen vorab fest.

---

# 4. Grundsatz

NC-PoRe sucht nicht die technisch spektakulärste Lösung, sondern eine Lösung, die für die konkrete Aufgabe einen guten Nutzen bei vertretbarer Komplexität bietet.

Technologie ist ein Werkzeug. Sie dient dem Produktionsprozess und seinen Nutzern.

---

# English Version

This document describes the general criteria for evaluating technical options in the project.

It does not replace Architecture Decision Records (ADRs) and does not define a binding technology roadmap. When a concrete technology choice has architectural significance, it is documented in the applicable ADR.

---

# 1. Evaluation Principles

NC-PoRe evaluates technical options against the actual requirements of the problem at hand.

A suitable solution should consider in particular:

- maintainability
- stability
- openness and interoperability
- security
- testability
- appropriate development and operational effort
- proportional complexity

Technical choices should not be driven solely by popularity, marketing or short-term trends.

---

# 2. Evaluation Criteria

## Maintainability

- understandability
- availability of knowledge and tooling
- troubleshooting
- long-term maintainability

## Openness

- suitable licensing
- open standards
- clear interfaces
- interoperability

## Technical Suitability

- fulfillment of functional requirements
- performance
- stability
- integration capability
- manageable complexity

## Security

- security model
- handling of sensitive data
- update and maintenance capability
- traceable trust boundaries

## Development Effort

- available tooling
- testability
- implementation effort
- operational and maintenance effort

---

# 3. Decision Process

Technical options are evaluated step by step:

1. determine the concrete requirement
2. identify relevant options
3. compare technical and functional criteria
4. examine risks and boundaries
5. document the decision
6. create or update an ADR when the decision has architectural significance

The concrete choice is always context-dependent. This document does not prescribe particular technologies or platforms in advance.

---

# 4. Principle

NC-PoRe does not seek the most spectacular technical solution, but a solution that provides good value for the concrete task at a reasonable level of complexity.

Technology is a tool. It serves the production process and its users.
