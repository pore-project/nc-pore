# NC-PoRe Technology Evaluation

* Version: 1.0
* Date: 2026-07-24

---

# Deutsch ([English version below](#english-version))

---

# 1. Zweck dieses Dokuments

Dieses Dokument beschreibt den Prozess zur strukturierten Bewertung
technischer Optionen für NC-PoRe.

Es trifft noch keine endgültigen Technologieentscheidungen.

Ziel ist es, mögliche Technologien anhand nachvollziehbarer Kriterien
zu untersuchen und eine fundierte Grundlage für spätere Entscheidungen
zu schaffen.

Technologieentscheidungen mit langfristiger architektonischer Bedeutung
werden weiterhin über Architecture Decision Records (ADRs) dokumentiert.

---

# 2. Grundprinzip

NC-PoRe wählt Technologien nicht nach kurzfristiger Popularität,
persönlichen Vorlieben oder Marketingversprechen aus.

Die Auswahl orientiert sich an den Anforderungen des Systems.

Eine moderne Technologie ist nicht automatisch die beste Lösung.

Eine geeignete Technologie unterstützt:

* langfristige Wartbarkeit
* Stabilität
* Offenheit
* Sicherheit
* Erweiterbarkeit
* nachhaltige Entwicklung

---

# 3. Bewertungskriterien

Technologien werden anhand folgender Kriterien bewertet.

---

## 3.1 Langfristige Wartbarkeit

Bewertet wird:

* Verständlichkeit der Technologie
* Verfügbarkeit von Wissen
* langfristige Unterstützung
* einfache Fehleranalyse

Eine Lösung muss auch Jahre nach der Einführung nachvollziehbar bleiben.

---

## 3.2 Offenheit und FOSS-Kompatibilität

Bewertet wird:

* Lizenzmodell
* offene Standards
* Integrationsmöglichkeiten
* Community-Unterstützung

NC-PoRe bevorzugt offene und nachvollziehbare Lösungen.

---

## 3.3 Plattformunterstützung

Bewertet wird:

* Unterstützung relevanter Betriebssysteme
* Qualität plattformübergreifender Entwicklung
* Zugriff auf notwendige Systemfunktionen

Besonders relevant:

* Linux
* Windows
* macOS
* iOS
* Android

---

## 3.4 Technische Eignung

Bewertet wird:

* Erfüllung der fachlichen Anforderungen
* Performance
* Stabilität
* Komplexität
* Integrationsfähigkeit

Eine Technologie muss zur Aufgabe passen.

---

## 3.5 Sicherheit

Bewertet wird:

* Sicherheitsmodell
* bekannte Schwachstellen
* Updatefähigkeit
* Umgang mit sensiblen Daten

---

## 3.6 Entwicklungsaufwand

Bewertet wird:

* Einstiegshürde
* Entwicklungsproduktivität
* Verfügbarkeit geeigneter Werkzeuge
* Wartungsaufwand

---

# 4. Technische Bereiche

Die Bewertung erfolgt getrennt nach technischen Bereichen.

---

# 4.1 Client-Technologien

Der Client ist verantwortlich für lokale Produktionsaufgaben.

Zu bewerten sind unter anderem:

* Zugriff auf Audio-Hardware
* Echtzeitverarbeitung
* lokale Speicherung
* Benutzeroberfläche
* Plattformintegration

Offene Fragen:

* Welche Plattformen müssen zuerst unterstützt werden?
* Welche Anforderungen entstehen durch professionelle Audioaufnahme?
* Welche Technologien ermöglichen eine gute Nutzererfahrung?

---

# 4.2 Core-Technologien

Der Core bildet die fachliche Autorität von NC-PoRe.

Zu bewerten sind unter anderem:

* Modellierung komplexer Geschäftslogik
* Testbarkeit
* Stabilität
* Schnittstellenfähigkeit
* langfristige Wartbarkeit

Offene Fragen:

* Welche Technologie unterstützt klare Domänenmodelle?
* Welche Laufzeitumgebung ist geeignet?
* Wie werden Erweiterungen umgesetzt?

---

# 4.3 Datenhaltung

Zu bewerten sind:

* Datenbanktechnologien
* lokale Datenspeicherung
* zentrale Speicherung
* Migrationen
* Backup-Strategien

Offene Fragen:

* Welche Datenmengen entstehen?
* Welche Daten müssen transaktional verwaltet werden?
* Welche Daten gehören in relationale oder andere Speicher?

---

# 4.4 Kommunikation

Zu bewerten sind:

* API-Technologien
* Event-Kommunikation
* Synchronisationsmechanismen
* Authentifizierung

Offene Fragen:

* Wie kommunizieren Clients mit dem Core?
* Wie werden Offline-Zustände behandelt?
* Wie werden Änderungen nachvollziehbar übertragen?

---

# 4.5 Audio-Technologien

Zu bewerten sind:

* Audio-Bibliotheken
* Plattformunterstützung
* Aufnahmequalität
* Latenz
* Formatunterstützung

Offene Fragen:

* Welche Audioformate werden benötigt?
* Welche Bibliotheken sind langfristig geeignet?
* Welche Anforderungen entstehen durch professionelle Produktion?

---

# 5. Entscheidungsprozess

Technologieentscheidungen erfolgen schrittweise.

Der Prozess:

1. Anforderungen definieren
2. mögliche Technologien identifizieren
3. Vor- und Nachteile bewerten
4. technische Risiken betrachten
5. Entscheidung dokumentieren
6. bei architektonischer Bedeutung ADR erstellen

---

# 6. Grundsatz

NC-PoRe sucht nicht die technisch spektakulärste Lösung.

Gesucht wird die Lösung, die langfristig den besten Nutzen bietet.

Technologie ist ein Werkzeug.

Sie dient den Menschen, die mit NC-PoRe arbeiten.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# 1. Purpose of this Document

This document describes the process for structured evaluation
of technical options for NC-PoRe.

It does not make final technology decisions yet.

The goal is to examine possible technologies based on transparent
criteria and create a solid foundation for future decisions.

Technology decisions with long-term architectural impact continue
to be documented through Architecture Decision Records (ADRs).

---

# 2. Fundamental Principle

NC-PoRe does not select technologies based on short-term popularity,
personal preferences or marketing claims.

Selection is based on system requirements.

A modern technology is not automatically the best solution.

A suitable technology supports:

* long-term maintainability
* stability
* openness
* security
* extensibility
* sustainable development

---

# 3. Evaluation Criteria

Technologies are evaluated according to the following criteria.

---

## 3.1 Long-Term Maintainability

Evaluation includes:

* understandability
* availability of knowledge
* long-term support
* ease of troubleshooting

A solution must remain understandable years after introduction.

---

## 3.2 Openness and FOSS Compatibility

Evaluation includes:

* licensing model
* open standards
* integration possibilities
* community support

NC-PoRe prefers open and transparent solutions.

---

## 3.3 Platform Support

Evaluation includes:

* support for relevant operating systems
* quality of cross-platform development
* access to required system functions

Especially relevant:

* Linux
* Windows
* macOS
* iOS
* Android

---

## 3.4 Technical Suitability

Evaluation includes:

* fulfillment of functional requirements
* performance
* stability
* complexity
* integration capability

A technology must fit the task.

---

## 3.5 Security

Evaluation includes:

* security model
* known vulnerabilities
* update capability
* handling of sensitive data

---

## 3.6 Development Effort

Evaluation includes:

* learning curve
* development productivity
* available tooling
* maintenance effort

---

# 4. Technical Areas

Evaluation is performed separately for technical areas.

---

# 4.1 Client Technologies

The client is responsible for local production tasks.

Evaluation includes:

* audio hardware access
* real-time processing
* local storage
* user interface
* platform integration

Open questions:

* Which platforms must be supported first?
* Which requirements result from professional audio recording?
* Which technologies enable good user experience?

---

# 4.2 Core Technologies

The Core represents the domain authority of NC-PoRe.

Evaluation includes:

* modeling complex business logic
* testability
* stability
* interface capabilities
* long-term maintainability

Open questions:

* Which technology supports clear domain models?
* Which runtime environment is suitable?
* How are extensions implemented?

---

# 4.3 Data Management

Evaluation includes:

* database technologies
* local data storage
* central storage
* migrations
* backup strategies

Open questions:

* Which data volumes are expected?
* Which data requires transactional management?
* Which data belongs in relational or other storage systems?

---

# 4.4 Communication

Evaluation includes:

* API technologies
* event communication
* synchronization mechanisms
* authentication

Open questions:

* How do clients communicate with the Core?
* How are offline states handled?
* How are changes transferred traceably?

---

# 4.5 Audio Technologies

Evaluation includes:

* audio libraries
* platform support
* recording quality
* latency
* format support

Open questions:

* Which audio formats are required?
* Which libraries are suitable long term?
* Which requirements result from professional production?

---

# 5. Decision Process

Technology decisions are made step by step.

The process:

1. define requirements
2. identify possible technologies
3. evaluate advantages and disadvantages
4. consider technical risks
5. document the decision
6. create an ADR when architectural impact exists

---

# 6. Principle

NC-PoRe does not search for the most spectacular technical solution.

It searches for the solution that provides the greatest long-term value.

Technology is a tool.

It serves the people working with NC-PoRe.
