# NC-PoRe Implementation Plan

## Deutsche Version (english version below)

---

# Zweck

Dieses Dokument beschreibt den Weg von der abgeschlossenen Architekturgrundlage zur ersten nutzbaren Version von NC-PoRe.

Der Implementation Plan beschreibt:

* in welcher Reihenfolge technische Komponenten entstehen
* welche Ergebnisse einzelne Phasen liefern sollen
* welche Entscheidungen vor der Umsetzung getroffen werden müssen
* welche Abhängigkeiten zwischen Komponenten bestehen

Dieses Dokument ersetzt keine Architecture Decision Records (ADRs).

ADRs beantworten:

> Warum wurde eine Entscheidung getroffen?

Der Implementation Plan beantwortet:

> In welcher Reihenfolge werden diese Entscheidungen umgesetzt?

---

# Grundsätze der Umsetzung

Die Umsetzung von NC-PoRe folgt den bestehenden Projektprinzipien.

## Präzision vor Marketing

Technische Dokumentation beschreibt die Realität des Systems.

Sie soll:

* präzise Begriffe verwenden
* Entscheidungen nachvollziehbar machen
* Auswirkungen von Entscheidungen beschreiben
* zwischen vorhandenen Fähigkeiten und zukünftigen Zielen unterscheiden

Technische Dokumentation ist kein Marketingmaterial.

---

## Architektur zuerst, Code danach

Die Architekturgrundlagen wurden bewusst vor Beginn der Implementierung definiert.

Die Umsetzung folgt dieser Struktur:

```text
Architektur

↓

Implementation Plan

↓

Technische Entscheidungen

↓

Code
```

---

## Kleine vollständige Schritte

NC-PoRe wird bevorzugt über vollständige vertikale Schritte entwickelt.

Ein kleiner vollständiger Ablauf ist wertvoller als viele isolierte Komponenten.

Beispiel:

```text
Production Session erzeugen

↓

Session speichern

↓

Session über API verfügbar machen

↓

Session im Client anzeigen
```

---

## Verantwortlichkeiten bleiben getrennt

Die Architekturprinzipien bleiben während der Umsetzung erhalten:

* Der Core enthält die fachliche Logik.
* Clients kümmern sich um Benutzerinteraktion.
* APIs definieren Kommunikationsgrenzen.
* Storage übernimmt persistente Datenhaltung.

---

## Keine vorzeitige Komplexität

Komplexität wird erst eingeführt, wenn ein konkreter Bedarf besteht.

NC-PoRe vermeidet:

* Optimierung ohne Messung
* Abstraktionen ohne Nutzen
* Skalierungsmechanismen ohne Anforderung
* technische Lösungen ohne konkreten Anwendungsfall

---

# Phase 1: Technische Projektgrundlage

## Ziel

Eine reproduzierbare Entwicklungsumgebung schaffen.

## Ergebnis

* Repository-Struktur definiert
* Entwicklungsumgebung dokumentiert
* Build-Prozess eingerichtet
* automatische Prüfungen möglich
* Entwicklungsworkflow beschrieben

## Zu klärende Entscheidungen

* Programmiersprachen
* Buildsystem
* Workspace-Struktur
* Entwicklungswerkzeuge

---

# Phase 2: Core-Implementierung

## Ziel

Die fachliche Grundlage von NC-PoRe implementieren.

Der Core ist die erste ausführbare Umsetzung der Architektur.

## Ergebnis

* Production Session kann erzeugt werden
* zentrale Domänenobjekte existieren
* Geschäftsregeln sind testbar
* Core funktioniert unabhängig von Clients

## Schwerpunkt

* Production Session
* Participants
* Roles
* Recordings
* Assets
* Activity History

---

# Phase 3: Kommunikationsschicht

## Ziel

Definierte Schnittstellen zwischen Systemkomponenten schaffen.

## Ergebnis

* Core-Funktionalität ist über Schnittstellen erreichbar
* Clients können mit dem System kommunizieren
* API-Prinzipien aus ADR-028 werden umgesetzt

## Schwerpunkt

* API-Design
* Fehlerbehandlung
* Authentifizierung
* Ereignisse und Zustandsänderungen

---

# Phase 4: Erster Client

## Ziel

Die Architektur durch einen realen Client validieren.

Der erste Client muss nicht vollständig sein.

Er soll zeigen, dass:

* Kommunikation funktioniert
* Sessions verwaltet werden können
* die Trennung zwischen Client und Core funktioniert

---

# Phase 5: Lokale Aufnahme

## Ziel

Das zentrale NC-PoRe-Prinzip technisch umsetzen:

> Lokal aufnehmen. Danach synchronisieren.

## Ergebnis

* lokale Audioaufnahme
* lokale Speicherung
* Aufnahmemetadaten
* Vorbereitung für Synchronisation

## Grundsatz

Die Aufnahme darf nicht von einer aktiven Netzwerkverbindung abhängig sein.

---

# Phase 6: Synchronisation

## Ziel

Lokale Produktionsdaten mit der zentralen Umgebung verbinden.

## Ergebnis

* Assets können übertragen werden
* Synchronisationszustände sind nachvollziehbar
* Konflikte können behandelt werden

## Bezug

* ADR-029 Distributed Recording Architecture
* ADR-030 Synchronization Strategy

---

# Phase 7: Produktionsworkflow

## Ziel

Die einzelnen Komponenten zu einem vollständigen Arbeitsablauf verbinden.

## Ergebnis

Ein vollständiger Produktionsablauf:

```text
Production Session erstellen

↓

Teilnehmer verwalten

↓

Lokal aufnehmen

↓

Assets synchronisieren

↓

Produktionsstatus prüfen

↓

Ergebnis exportieren
```

---

# Phase 8: Erste nutzbare Version

## Ziel

Eine Version erstellen, die für reale Podcast-Produktion eingesetzt werden kann.

## Ergebnis

* stabiler Aufnahmeprozess
* zuverlässige Synchronisation
* nutzbare Clients
* dokumentierte Installation

---

# Entscheidungspunkte

Während der Umsetzung werden neue Entscheidungen als ADR dokumentiert, wenn sie:

* mehrere Komponenten betreffen
* langfristige Auswirkungen haben
* bestehende Architekturprinzipien verändern

---

# Was wir bewusst noch nicht tun

NC-PoRe vermeidet bewusst:

* vollständige Benutzeroberflächen vor stabiler Kernlogik
* Optimierung vor realer Nutzung
* technische Komplexität ohne Bedarf
* Ersatz bestehender Werkzeuge ohne Grund
* Lösungen für hypothetische Probleme

---

# Aktueller Umsetzungsstatus

Status:

## Nicht begonnen

Die Architekturgrundlage ist abgeschlossen.

Der nächste Schritt ist die technische Umsetzungsvorbereitung.

---

# Beziehung zu anderen Dokumenten

```text
README.md

↓

project-status.md

↓

implementation-plan.md

↓

ADR-Dokumente

↓

Source Code
```

Die Dokumente haben unterschiedliche Aufgaben:

* README beschreibt das Projekt.
* project-status beschreibt den aktuellen Zustand.
* implementation-plan beschreibt den Weg zur Umsetzung.
* ADRs erklären Entscheidungen.
* Source Code implementiert die Ergebnisse.

---

# Leitgedanke

NC-PoRe wird nicht durch möglichst schnelles Schreiben von Code entwickelt.

NC-PoRe wird durch nachvollziehbare Entscheidungen entwickelt.

Diese Entscheidungen werden anschließend in zuverlässige Software umgesetzt.

---

# English Version (german version above)

---

# Purpose

This document describes the path from the completed architecture foundation to the first usable version of NC-PoRe.

The Implementation Plan describes:

* the order in which technical components are created
* expected results of individual phases
* decisions required before implementation
* dependencies between components

This document does not replace Architecture Decision Records (ADRs).

ADRs answer:

> Why was a decision made?

The Implementation Plan answers:

> In which order are these decisions implemented?

---

# Implementation Principles

NC-PoRe implementation follows the established project principles.

## Precision over Marketing

Technical documentation describes the actual system.

It should:

* use precise terminology
* make decisions understandable
* describe consequences of decisions
* distinguish implemented capabilities from future goals

Technical documentation is not marketing material.

---

# Current Implementation Status

Status:

## Not started

The architecture foundation is complete.

The next step is technical implementation preparation.
