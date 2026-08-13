# Deutsch ([English version below](#english-version))

# ADR-008: Client Architecture

## Status

Accepted

## Date

2026-07-22

---

# Kontext

NC-PoRe benötigt eine zuverlässige lokale Audioaufnahme.

Die Aufnahmequalität darf nicht von einem Browser, einer
Serververbindung oder externen Diensten abhängig sein.

Gleichzeitig soll NC-PoRe möglichst einfach zugänglich sein,
insbesondere für Gäste und gelegentliche Teilnehmer.

Daraus entsteht ein Zielkonflikt:

* maximale technische Kontrolle für professionelle Aufnahmen
* möglichst einfacher Zugang für Teilnehmer

---

# Entscheidung

NC-PoRe verwendet eine modulare Client-Architektur.

Die lokale Aufnahme erfolgt durch einen spezialisierten
Recorder-Client.

Der Recorder ist für folgende Aufgaben verantwortlich:

* Zugriff auf Audiohardware
* lokale Aufnahme
* Chunk-Verwaltung
* Metadaten-Erzeugung
* lokale Sicherheit
* Upload-Vorbereitung

Der Server übernimmt keine primäre Audioaufnahme.

---

# Client-Varianten

NC-PoRe unterstützt perspektivisch unterschiedliche
Client-Varianten.

## Professional Recorder

Für regelmäßige Podcaster und Produktionsumgebungen.

Eigenschaften:

* maximale Audioqualität
* erweiterte Einstellungen
* zuverlässige lokale Speicherung
* professionelle Workflows

---

## Guest Recorder

Für externe Teilnehmer.

Ziel:

* möglichst einfache Teilnahme
* geringe Einstiegshürde
* sichere Session-Teilnahme

Der Gast benötigt keine umfangreiche Verwaltung.

---

# Architekturmodell

```
                Nextcloud Server

                    |
                    |
          Session Management
                    |
        +-----------+-----------+
        |                       |
        |                       |
 Professional Client      Guest Client

        |                       |
        +-----------+-----------+

              lokale Aufnahme

                    |

             Upload nach Session-Ende
```

---

# Browser-basierte Aufnahme

Eine reine Browser-Aufnahme wird nicht als
Primärarchitektur verwendet.

Gründe:

* eingeschränkte Kontrolle über Hardwarezugriff
* abhängig vom Browserverhalten
* schwierigeres Fehlerhandling
* eingeschränkte Möglichkeiten für professionelle Workflows

Browserbasierte Teilnahme kann jedoch zukünftig als
vereinfachter Zugang unterstützt werden.

---

# Konsequenzen

## Positive Auswirkungen

* professionelle Aufnahmequalität möglich
* klare Trennung zwischen Aufnahme und Server
* bessere Erweiterbarkeit
* geeignet für verschiedene Nutzergruppen
* unabhängiger von Browserherstellern

---

## Negative Auswirkungen

* zusätzliche Softwarekomponente erforderlich
* Installation kann notwendig sein
* mehrere Clients müssen gepflegt werden

---

# Betrachtete Alternativen

## Ausschließliche Web-App

Verworfen als Hauptlösung.

Grund:

Eine Web-App bietet nicht die notwendige Kontrolle für
professionelle lokale Audioaufnahme.

---

## Ausschließlicher Desktop-Client

Nicht ausreichend.

Grund:

Gelegenheitsnutzer und Gäste benötigen einen einfacheren
Zugang.

---

# Hinweise

Die Client-Architektur unterstützt das Grundprinzip von NC-PoRe:

> Professionelle Werkzeuge für diejenigen, die sie benötigen,
> einfache Teilnahme für diejenigen, die nur beitragen.

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-008: Client Architecture

## Status

Accepted

## Date

2026-07-22

---

# Context

NC-PoRe requires reliable local audio recording.

Recording quality must not depend on a browser, a server connection, or external services.

At the same time, NC-PoRe should be as easy to access as possible, especially for guests and occasional participants.

This creates a conflict of objectives:

* maximum technical control for professional recordings
* access for participants should be as simple as possible

---

# Decision

NC-PoRe uses a modular client architecture.

Local recording is performed by a specialized recorder client.

The recorder is responsible for the following tasks:

* access to audio hardware
* local recording
* chunk management
* metadata generation
* local security
* upload preparation

The server does not perform the primary audio recording.

---

# Client Variants

NC-PoRe is intended to support different client variants.

## Professional Recorder

For regular podcasters and production environments.

Characteristics:

* maximum audio quality
* advanced settings
* reliable local storage
* professional workflows

---

## Guest Recorder

For external participants.

Goal:

* participation should be as simple as possible
* low entry barrier
* secure session participation

The guest does not need extensive administration capabilities.

---

# Architecture Model

```
                Nextcloud Server

                    |
                    |
          Session Management
                    |
        +-----------+-----------+
        |                       |
        |                       |
 Professional Client      Guest Client

        |                       |
        +-----------+-----------+

              local recording

                    |

             Upload after session end
```

---

# Browser-Based Recording

Pure browser-based recording is not used as the primary architecture.

Reasons:

* limited control over hardware access
* dependent on browser behavior
* more difficult error handling
* limited options for professional workflows

Browser-based participation may nevertheless be supported in the future as a simplified access method.

---

# Consequences

## Positive Effects

* professional recording quality is possible
* clear separation between recording and server
* better extensibility
* suitable for different user groups
* independent of browser vendors

---

## Negative Effects

* additional software component required
* installation may be necessary
* multiple clients must be maintained

---

# Alternatives Considered

## Web App Only

Rejected as the primary solution.

Reason:

A web app does not provide the necessary control for professional local audio recording.

---

## Desktop Client Only

Insufficient.

Reason:

Occasional users and guests need simpler access.

---

# Notes

The client architecture supports the fundamental principle of NC-PoRe:

> Professional tools for those who need them,
> simple participation for those who only contribute.
