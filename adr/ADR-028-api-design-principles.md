# ADR-028: API Design Principles

* Status: Accepted
* Date: 2026-07-23
* Decision Type: Architecture

---

# Deutsch (English version below)

## Kontext

NC-PoRe wurde als modulare Architektur definiert.

Die bisherigen Entscheidungen legen fest:

* Die **Production Session** ist die zentrale fachliche Einheit.
* Der **Core** enthält die fachliche Wahrheit von NC-PoRe.
* Clients, Speicher und externe Systeme werden über definierte Schnittstellen angebunden.
* Der Core bleibt unabhängig von konkreten technischen Implementierungen.

Damit diese Architektur langfristig funktioniert, benötigt NC-PoRe eine klare API-Strategie.

Die API ist die Verbindung zwischen:

* Core
* Clients
* externen Systemen
* zukünftigen Erweiterungen

---

# Entscheidung

NC-PoRe entwickelt APIs nach folgenden Grundprinzipien:

1. API First
2. Domain-orientierte Schnittstellen
3. stabile und erweiterbare Verträge
4. Ereignisorientierung
5. Sicherheit als Grundprinzip
6. Dokumentation als Bestandteil der API

Die API beschreibt Fähigkeiten von NC-PoRe, nicht interne Implementierungsdetails.

---

# API First Prinzip

Die API ist die stabile Grenze zwischen Core und Umgebung.

Clients bestimmen nicht die interne Struktur des Systems.

Stattdessen:

> Der Core definiert Fähigkeiten. Clients nutzen diese Fähigkeiten.

Beispiel:

Nicht:

```text id="5k0x9m"
LinuxClient.startRecording()
```

Sondern:

```text id="8w2v4p"
startRecording(session)
```

Die konkrete technische Umsetzung kann später unterschiedlich erfolgen.

---

# Domain-orientierte API

Die API orientiert sich an der Fachlichkeit von NC-PoRe.

Nicht:

```text id="g0h4pa"
createFile()
uploadBlob()
saveData()
```

Sondern:

```text id="w7y2cp"
createSession()
addParticipant()
attachAsset()
startRecording()
finishSession()
```

Die API spricht die Sprache des Produkts.

Sie bildet nicht die interne Speicherstruktur ab.

---

# Trennung von API und Implementierung

Eine API beschreibt:

* was möglich ist
* welche Daten benötigt werden
* welche Ergebnisse entstehen

Eine API beschreibt nicht:

* wie eine Funktion intern implementiert wird
* welche Datenbank verwendet wird
* welcher Provider aktiv ist

Interne Änderungen sollen möglich bleiben, ohne Clients zu brechen.

---

# API-Versionierung und Stabilität

NC-PoRe verfolgt eine evolutionäre API-Strategie.

Ziel:

* Erweiterung statt Austausch
* Rückwärtskompatibilität
* klare Änderungsprozesse

Eine neue Funktion soll möglichst bestehende Clients nicht unnötig beeinflussen.

---

# Events als Bestandteil der Architektur

NC-PoRe wird ereignisorientierte Konzepte unterstützen.

Eine Production Session erzeugt Ereignisse wie:

```text id="q9j1hc"
SessionCreated

ParticipantJoined

RecordingStarted

RecordingFinished

AssetCreated

ExportCompleted
```

Events ermöglichen:

* Synchronisation
* verteilte Clients
* nachvollziehbare Produktionsabläufe
* zukünftige Erweiterungen

---

# API ist ein Konzept, kein Protokoll

Die API beschreibt eine architektonische Grenze.

Die konkrete technische Umsetzung bleibt offen.

Mögliche Implementierungen können sein:

* REST
* WebSocket
* gRPC
* lokale Schnittstellen
* interne Funktionsaufrufe

Die Architektur legt nicht vorzeitig ein bestimmtes Protokoll fest.

---

# Sicherheit als Grundprinzip

APIs werden von Anfang an mit Sicherheitsanforderungen entwickelt.

Dazu gehören:

* Authentifizierung
* Autorisierung
* minimale Berechtigungen
* sichere Kommunikation
* nachvollziehbare Zugriffe

Sicherheit wird nicht nachträglich ergänzt.

---

# Dokumentation als Teil der API

Eine API ohne verständliche Dokumentation ist kein vollständiges Produkt.

Jede öffentliche oder interne stabile API benötigt:

* Beschreibung der Funktionen
* Datenmodelle
* Beispiele
* Änderungsregeln
* Versionsinformationen

Die Dokumentation wird entsprechend der Internationalisierungsstrategie zweisprachig geführt:

* Deutsch
* Englisch

---

# Grundprinzipien

## 1. Die API ist die Sprache des Systems

Die API verbindet Menschen, Clients und Systeme.

Sie beschreibt Möglichkeiten, nicht technische Details.

---

## 2. Der Benutzer denkt in Aufgaben, nicht Funktionen

Die API soll die tatsächlichen Arbeitsabläufe unterstützen.

Beispiele:

Nicht:

> Datei speichern

Sondern:

> Aufnahme zur Session hinzufügen

---

## 3. Erweiterbarkeit vor kurzfristiger Optimierung

Eine gute API erlaubt zukünftige Entwicklungen:

* neue Clients
* neue Provider
* neue Medienformate
* neue Kommunikationssysteme

---

## 4. Einfachheit für den Nutzer

Technische Komplexität bleibt hinter der API verborgen.

Der Benutzer soll nicht wissen müssen:

* welche Speichertechnik verwendet wird
* welches Kommunikationssystem aktiv ist
* welche Plattform genutzt wird

---

# Konsequenzen

## Vorteile

* klare Trennung zwischen Core und Umgebung
* mehrere Clients möglich
* bessere Wartbarkeit
* einfachere Erweiterungen
* Grundlage für verteilte Systeme
* bessere Testbarkeit

## Nachteile

* zusätzlicher Entwicklungsaufwand
* APIs benötigen langfristige Pflege
* Entscheidungen müssen sorgfältig getroffen werden

Diese Nachteile werden bewusst akzeptiert.

---

# Nicht-Ziele

Diese Entscheidung bedeutet nicht:

* dass sofort eine öffentliche Entwickler-API angeboten werden muss
* dass ein bestimmtes Netzwerkprotokoll festgelegt wird
* dass alle zukünftigen Funktionen bereits bekannt sein müssen
* dass interne Implementierungen niemals geändert werden dürfen

---

# Leitgedanke

Die API ist die stabile Sprache von NC-PoRe.

Sie verbindet den Core mit der Welt außerhalb des Systems.

Sie beschreibt, was NC-PoRe kann — nicht, wie NC-PoRe intern funktioniert.

---

# English (Deutsche Version oben)

## Context

NC-PoRe has been defined as a modular architecture.

Previous decisions established:

* The **Production Session** is the central domain entity.
* The **Core** contains the domain truth of NC-PoRe.
* Clients, storage and external systems connect through defined interfaces.
* The Core remains independent from concrete implementations.

To make this architecture sustainable, NC-PoRe requires a clear API strategy.

The API connects:

* Core
* clients
* external systems
* future extensions

---

# Decision

NC-PoRe designs APIs according to the following principles:

1. API First
2. Domain-oriented interfaces
3. stable and extensible contracts
4. event orientation
5. security as a foundation
6. documentation as part of the API

The API describes NC-PoRe capabilities, not internal implementation details.

---

# API First Principle

The API is the stable boundary between Core and the outside world.

Clients do not define the internal system structure.

Instead:

> The Core defines capabilities. Clients use these capabilities.

---

# Domain-oriented API

The API follows the domain language of NC-PoRe.

Not:

```text id="4l8vma"
createFile()
uploadBlob()
saveData()
```

But:

```text id="n3r6tx"
createSession()
addParticipant()
attachAsset()
startRecording()
finishSession()
```

The API speaks the language of the product.

---

# Separation of API and Implementation

An API describes:

* what is possible
* required data
* resulting behavior

An API does not describe:

* internal implementation
* database technology
* active provider

Internal changes should remain possible without breaking clients.

---

# API Versioning and Stability

NC-PoRe follows an evolutionary API strategy.

Goals:

* extension instead of replacement
* backward compatibility
* clear change processes

---

# Events as Part of Architecture

NC-PoRe will support event-oriented concepts.

A Production Session creates events such as:

```text id="p1m7zc"
SessionCreated

ParticipantJoined

RecordingStarted

RecordingFinished

AssetCreated

ExportCompleted
```

Events enable:

* synchronization
* distributed clients
* traceable workflows
* future extensions

---

# API Is a Concept, Not a Protocol

The API defines an architectural boundary.

The technical implementation remains open.

Possible implementations:

* REST
* WebSocket
* gRPC
* local interfaces
* internal function calls

---

# Security as a Foundation

APIs are designed with security requirements from the beginning.

Including:

* authentication
* authorization
* minimal permissions
* secure communication
* traceable access

---

# Documentation as Part of the API

An API without understandable documentation is not a complete product.

Stable APIs require:

* function descriptions
* data models
* examples
* change rules
* version information

Documentation follows the internationalization strategy:

* German
* English

---

# Guiding Principle

The API is the stable language of NC-PoRe.

It connects the Core with the world outside the system.

It describes what NC-PoRe can do — not how NC-PoRe works internally.
