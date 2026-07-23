# ADR-029: Distributed Recording Architecture

* Status: Accepted
* Date: 2026-07-23
* Decision Type: Architecture

---

# Deutsch (English version below)

## Kontext

NC-PoRe wurde als **Nextcloud Podcast Production Environment** definiert.

Die bisher getroffenen Architekturentscheidungen legen fest:

* Die **Production Session** ist die zentrale fachliche Einheit.
* Medien werden als Assets innerhalb einer Session verwaltet.
* Clients, Provider und externe Systeme werden über Schnittstellen angebunden.
* Der Core bleibt unabhängig von konkreten technischen Implementierungen.

Ein zentraler Anwendungsfall von NC-PoRe ist die verteilte Podcast-Produktion:

Beispiel:

```text
Host 1  → macOS
Host 2  → Linux
Host 3  → Windows
Gast 1  → iOS
Gast 2  → Android
```

Alle Teilnehmer sollen gemeinsam an einer Production Session arbeiten können.

Dabei entstehen Herausforderungen:

* unterschiedliche Plattformen
* unterschiedliche Hardware
* unterschiedliche Netzwerkbedingungen
* Ausfälle einzelner Verbindungen
* Synchronisation mehrerer Aufnahmespuren

---

# Entscheidung

NC-PoRe verwendet ein:

**Local Recording First Modell**

Die Aufnahme erfolgt grundsätzlich lokal auf dem jeweiligen Client.

Die zentrale Production Session koordiniert:

* Teilnehmer
* Aufnahmezustand
* Metadaten
* Synchronisation
* Upload der Assets

NC-PoRe versucht nicht, das Internet wie eine direkte lokale Verbindung zu behandeln.

Stattdessen arbeitet NC-PoRe mit den Eigenschaften verteilter Systeme.

Leitgedanke:

> Wir arbeiten mit dem Internet – nicht dagegen.

---

# Architektur

```text
                 Production Session

                       |
                       |
             Session Coordination

        ---------------------------------

        Mac Client       Linux Client      Mobile Client

             |                |                  |

        Local Track      Local Track       Local Track

             |                |                  |

             -------- Asset Upload --------

                       |

                       ↓

                  Storage Provider

                  (Nextcloud V1)
```

---

# Begründung für Local Recording First

Eine reine Live-Aufnahme über das Netzwerk hätte erhebliche Nachteile:

* Netzwerkunterbrechungen können Aufnahmen beschädigen
* Audioqualität hängt von der Verbindung ab
* mobile Teilnehmer sind besonders betroffen
* Fehleranalyse wird komplexer

Lokale Aufnahmen bieten:

* maximale Audioqualität
* Unabhängigkeit von Netzwerkproblemen
* bessere Ausfallsicherheit
* einfache Nutzung externer Werkzeuge

---

# Production Session als Koordinator

Die Session ist die gemeinsame Referenz.

Sie verwaltet:

```text
Production Session

├── Participants
│
├── Recording States
│
├── Audio Assets
│
├── Video Assets (future)
│
├── Synchronization Metadata
│
└── Events
```

---

# Asset Synchronisation

Jede lokale Aufnahme wird als Asset der Session hinzugefügt.

Beispiel:

```text
Production Session

├── host_1.wav
├── host_2.wav
├── guest_1.wav
│
├── metadata
│
└── synchronization data
```

Assets enthalten notwendige Informationen:

* Session ID
* Participant ID
* Aufnahmeinformationen
* Zeitinformationen
* technische Metadaten

---

# Synchronisation

Die konkrete Synchronisationsmethode wird in späteren ADRs definiert.

Mögliche Mechanismen:

* gemeinsamer Session-Start
* Zeitstempel
* Synchronisationsmarker
* automatische Analyse von Audiosignalen

ADR-029 definiert nur das Architekturprinzip:

> Jede Aufnahme bleibt zunächst lokal gültig und wird anschließend Bestandteil einer gemeinsamen Production Session.

---

# Umgang mit Verbindungsabbrüchen

Ein Client darf zeitweise nicht erreichbar sein.

Beispiel:

Ein Gast verliert während der Aufnahme die Internetverbindung.

Das Ergebnis:

Nicht:

> Aufnahme verloren.

Sondern:

> Die lokale Aufnahme wird später synchronisiert.

---

# Benutzerperspektive

Der Benutzer erlebt einen einfachen Ablauf:

1. Production Session öffnen
2. Teilnehmer verbinden sich
3. Aufnahme starten
4. Aufnahme durchführen
5. Aufnahme beenden
6. Synchronisation erfolgt automatisch

Die Komplexität verteilter Systeme bleibt verborgen.

---

# Beziehung zu externen Werkzeugen

Die Architektur unterstützt weiterhin externe Produktionswerkzeuge.

Lokale Assets können verwendet werden für:

* Audacity
* Ardour
* weitere Audio- und Videowerkzeuge

NC-PoRe organisiert die Produktion, ersetzt jedoch nicht automatisch spezialisierte Werkzeuge.

---

# Erweiterbarkeit

Das Local Recording First Prinzip ermöglicht spätere Erweiterungen:

* Videoaufnahme
* Bildschirmaufnahme
* zusätzliche Audiospuren
* bessere Synchronisationsverfahren
* weitere Clients

---

# Grundprinzipien

## 1. Lokal aufnehmen, gemeinsam produzieren

Die Aufnahme findet dort statt, wo die Person arbeitet.

Die Produktion entsteht gemeinsam.

---

## 2. Fehler einzelner Komponenten dürfen nicht die gesamte Produktion zerstören

Ein Problem bei einem Teilnehmer darf nicht automatisch die gesamte Session gefährden.

---

## 3. Das Internet ist ein verteiltes System

NC-PoRe berücksichtigt:

* Latenzen
* Verbindungsabbrüche
* unterschiedliche Geräte

statt diese Realität zu ignorieren.

---

## 4. Benutzerfreundlichkeit vor technischer Eleganz

Die interne Architektur darf komplex sein.

Die Bedienung bleibt einfach.

---

# Konsequenzen

## Vorteile

* hohe Aufnahmequalität
* robuste verteilte Produktion
* unabhängige Clients
* gute Erweiterbarkeit
* kompatibel mit professionellen Workflows

## Nachteile

* Synchronisation ist komplex
* Clients benötigen lokale Ressourcen
* zusätzliche Metadaten müssen verwaltet werden

Diese Nachteile werden bewusst akzeptiert.

---

# Nicht-Ziele

Diese Entscheidung bedeutet nicht:

* dass NC-PoRe sofort perfekte Echtzeitübertragung anbieten muss
* dass lokale Aufnahmen abgeschafft werden
* dass externe Produktionswerkzeuge ersetzt werden
* dass alle Synchronisationsprobleme bereits gelöst sind

---

# Leitgedanke

NC-PoRe macht verteilte Podcast-Produktion einfach, indem es die Realität verteilter Systeme akzeptiert.

**Lokal aufnehmen. Gemeinsam produzieren. Sicher speichern.**

---

# English (Deutsche Version oben)

## Context

NC-PoRe has been defined as a **Nextcloud Podcast Production Environment**.

Previous architecture decisions established:

* The **Production Session** is the central domain entity.
* Media is managed as assets within sessions.
* Clients, providers and external systems connect through interfaces.
* The Core remains independent from concrete implementations.

A central use case is distributed podcast production:

Example:

```text
Host 1  → macOS
Host 2  → Linux
Host 3  → Windows
Guest 1  → iOS
Guest 2  → Android
```

All participants should work together on one Production Session.

Challenges include:

* different platforms
* different hardware
* different network conditions
* individual connection failures
* synchronization of multiple recordings

---

# Decision

NC-PoRe uses a:

**Local Recording First Model**

Recording happens locally on each client.

The central Production Session coordinates:

* participants
* recording state
* metadata
* synchronization
* asset uploads

NC-PoRe does not attempt to treat the Internet as a local connection.

Instead, NC-PoRe works with the realities of distributed systems.

Guiding principle:

> We work with the Internet – not against it.

---

# Architecture

```text
                 Production Session

                       |
                       |
             Session Coordination

        ---------------------------------

        Mac Client       Linux Client      Mobile Client

             |                |                  |

        Local Track      Local Track       Local Track

             |                |                  |

             -------- Asset Upload --------

                       |

                       ↓

                  Storage Provider

                  (Nextcloud V1)
```

---

# Rationale for Local Recording First

Pure live recording over the network has disadvantages:

* connection failures can damage recordings
* audio quality depends on network conditions
* mobile participants are especially affected
* troubleshooting becomes complex

Local recordings provide:

* maximum audio quality
* independence from network problems
* better reliability
* compatibility with external tools

---

# Guiding Principle

NC-PoRe simplifies distributed podcast production by accepting the reality of distributed systems.

**Record locally. Produce together. Store securely.**
