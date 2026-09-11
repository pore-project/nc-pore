# ADR-029: Distributed Recording Architecture

* Status: Accepted
* Date: 2026-07-23
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

## Kontext

NC-PoRe wurde als **Nextcloud Podcast Production Environment** definiert.

Die bisherigen Architekturentscheidungen legen fest:

* Die **Production Session** ist die zentrale fachliche Einheit.
* Medien werden als Assets innerhalb einer Session verwaltet.
* Clients, Provider und externe Systeme werden über Schnittstellen angebunden.
* Der Core bleibt unabhängig von konkreten technischen Implementierungen.

Ein zentraler Anwendungsfall von NC-PoRe ist die verteilte Podcast-Produktion. Teilnehmer können mit unterschiedlichen Geräten, Hardware- und Netzwerkbedingungen an einer gemeinsamen Production Session arbeiten.

Dabei entstehen insbesondere Herausforderungen durch unterschiedliche Plattformen, Netzwerkbedingungen, Ausfälle einzelner Verbindungen und die Synchronisation mehrerer Aufnahmespuren.

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

NC-PoRe versucht nicht, das Internet wie eine direkte lokale Verbindung zu behandeln. Stattdessen arbeitet NC-PoRe mit den Eigenschaften verteilter Systeme.

Leitgedanke:

> Wir arbeiten mit dem Internet – nicht dagegen.

---

# Architektur

```text
                 Production Session

                       |
              Session Coordination

        -------------------------------

        Client A         Client B        Client C

            |                |               |

        Local Track      Local Track     Local Track

            |                |               |

            -------- Asset Upload --------

                       |

                       v

                  Storage Provider
```

Die konkreten Client- und Provider-Technologien sind nicht Bestandteil dieser ADR.

---

# Begründung für Local Recording First

Eine reine Live-Aufnahme über das Netzwerk hätte erhebliche Nachteile:

* Netzwerkunterbrechungen können Aufnahmen beschädigen
* Audioqualität hängt von der Verbindung ab
* unterschiedliche Netzbedingungen erschweren die Verarbeitung
* Fehleranalyse wird komplexer

Lokale Aufnahmen bieten:

* hohe und kontrollierbare Audioqualität
* Unabhängigkeit von Netzwerkproblemen während der Aufnahme
* bessere Ausfallsicherheit
* klare Trennung zwischen Aufnahme und Synchronisation

Entscheidend ist dabei die Verantwortungsgrenze: Die lokale Aufnahme bleibt auch dann ein gültiges technisches Ergebnis, wenn die gemeinsame Session oder der Upload vorübergehend nicht verfügbar ist. Die Session-Koordination ersetzt weder die lokale Aufnahmedatenhaltung noch macht sie eine laufende Aufnahme von der Netzwerkverbindung abhängig.

---

# Production Session als Koordinator

Die Session ist die gemeinsame Referenz.

Sie verwaltet konzeptionell:

```text
Production Session

├── Participants
├── Recording States
├── Audio Assets
├── Synchronization Metadata
└── Events
```

Die Session koordiniert damit die gemeinsame Produktion, ist aber nicht die technische Capture-Instanz eines einzelnen Clients.

---

# Asset Synchronisation

Jede lokale Aufnahme wird als Asset der Session hinzugefügt.

Assets enthalten die Informationen, die erforderlich sind, um Aufnahme und Session eindeutig zuzuordnen und zu synchronisieren.

Beispielsweise:

```text
Production Session

├── recording asset
├── metadata
└── synchronization data
```

---

# Synchronisation

Die konkrete Synchronisationsmethode wird in dafür zuständigen ADRs definiert.

ADR-029 definiert nur das Architekturprinzip:

> Jede Aufnahme bleibt zunächst lokal gültig und wird anschließend Bestandteil einer gemeinsamen Production Session.

Die technische Übertragung eines Assets darf daher nicht mit der Gültigkeit der lokalen Aufnahme gleichgesetzt werden. Ein Asset kann lokal bereits vollständig vorliegen, obwohl seine Synchronisation noch aussteht.

---

# Umgang mit Verbindungsabbrüchen

Ein Client darf zeitweise nicht erreichbar sein.

Ein Verbindungsabbruch während der Aufnahme darf die lokale Aufnahme nicht automatisch ungültig machen. Die Synchronisation erfolgt, sobald die erforderliche Verbindung wieder verfügbar ist.

---

# Benutzerperspektive

Der Benutzer erlebt grundsätzlich einen einfachen Ablauf:

1. Production Session öffnen
2. Teilnehmer verbinden sich
3. Aufnahme starten
4. Aufnahme durchführen
5. Aufnahme beenden
6. Synchronisation der Assets

Die Komplexität verteilter Systeme bleibt hinter diesem Ablauf verborgen.

---

# Beziehung zu externen Werkzeugen

Die Architektur hält lokale Assets für externe Produktionswerkzeuge zugänglich.

NC-PoRe organisiert die gemeinsame Produktion, ohne die konkrete interne Arbeitsweise externer Werkzeuge vorzugeben.

---

# Grundprinzipien

## 1. Lokal aufnehmen, gemeinsam produzieren

Die Aufnahme findet dort statt, wo die Person arbeitet. Die Produktion entsteht gemeinsam.

## 2. Fehler einzelner Komponenten dürfen nicht die gesamte Produktion zerstören

Ein Problem bei einem Teilnehmer darf nicht automatisch die gesamte Session gefährden.

## 3. Das Internet ist ein verteiltes System

NC-PoRe berücksichtigt Latenzen, Verbindungsabbrüche und unterschiedliche Geräte, statt diese Eigenschaften zu ignorieren.

## 4. Benutzerfreundlichkeit vor technischer Eleganz

Die interne Architektur darf komplex sein. Die Bedienung bleibt einfach.

---

# Konsequenzen

## Vorteile

* robuste verteilte Produktion
* unabhängige Clients
* hohe Aufnahmequalität
* klare Trennung von Aufnahme und Synchronisation

## Nachteile

* Synchronisation ist komplex
* Clients benötigen lokale Ressourcen
* zusätzliche Metadaten müssen verwaltet werden

Diese Nachteile werden bewusst akzeptiert.

---

# Nicht-Ziele

Diese Entscheidung bedeutet nicht:

* dass die konkrete Client-Plattform festgelegt wird
* dass eine bestimmte Netzwerkübertragung vorgeschrieben wird
* dass externe Produktionswerkzeuge ersetzt werden
* dass alle Synchronisationsdetails bereits festgelegt sind

---

# Leitgedanke

NC-PoRe macht verteilte Podcast-Produktion einfach, indem es die Realität verteilter Systeme akzeptiert.

**Lokal aufnehmen. Gemeinsam produzieren. Sicher speichern.**

---

# English Version ([Deutsche Version oben](#deutsch))

## Context

NC-PoRe has been defined as a **Nextcloud Podcast Production Environment**.

Previous architecture decisions established:

* The **Production Session** is the central domain entity.
* Media is managed as assets within a session.
* Clients, providers and external systems connect through interfaces.
* The Core remains independent from concrete technical implementations.

A central use case is distributed podcast production. Participants may work on one Production Session with different devices, hardware and network conditions.

This creates challenges such as different environments, connection failures and synchronization of multiple recordings.

---

# Decision

NC-PoRe uses a:

**Local Recording First Model**

Recording happens locally on the respective client.

The central Production Session coordinates:

* participants
* recording state
* metadata
* synchronization
* asset uploads

NC-PoRe does not attempt to treat the Internet as a direct local connection. Instead, it works with the realities of distributed systems.

Guiding principle:

> We work with the Internet – not against it.

---

# Architecture

```text
                 Production Session

                       |
              Session Coordination

        -------------------------------

        Client A         Client B        Client C

            |                |               |

        Local Track      Local Track     Local Track

            |                |               |

            -------- Asset Upload --------

                       |

                       v

                  Storage Provider
```

Concrete client and provider technologies are not defined by this ADR.

---

# Rationale for Local Recording First

Pure live recording over the network has significant disadvantages:

* connection failures can damage recordings
* audio quality depends on network conditions
* varying network conditions complicate processing
* troubleshooting becomes more complex

Local recordings provide:

* high and controllable audio quality
* independence from network problems during recording
* better resilience
* clear separation between recording and synchronization

The responsibility boundary is important here: the local recording remains a valid technical result even when the shared session or asset upload is temporarily unavailable. Session coordination neither replaces local recording-data persistence nor makes an active recording dependent on network availability.

---

# Production Session as Coordinator

The session is the shared reference.

Conceptually it manages:

```text
Production Session

├── Participants
├── Recording States
├── Audio Assets
├── Synchronization Metadata
└── Events
```

The session therefore coordinates the shared production, but it is not the technical capture instance of an individual client.

---

# Asset Synchronization

Each local recording is added as an asset of the session.

Assets contain the information required to associate the recording with the session and to synchronize it.

Conceptually:

```text
Production Session

├── recording asset
├── metadata
└── synchronization data
```

---

# Synchronization

The concrete synchronization method is defined by dedicated architecture decisions.

ADR-029 defines only the architectural principle:

> Each recording remains locally valid first and subsequently becomes part of a shared Production Session.

Technical asset transfer must therefore not be equated with the validity of the local recording. An asset may already be complete locally while synchronization is still pending.

---

# Handling Connection Loss

A client may temporarily become unreachable.

A connection loss during recording must not automatically invalidate the local recording. Synchronization can occur once the required connection is available again.

---

# User Perspective

The user experiences a simple overall flow:

1. Open Production Session
2. Participants connect
3. Start recording
4. Record
5. Finish recording
6. Synchronize assets

The complexity of distributed systems remains behind this flow.

---

# Relationship to External Tools

The architecture keeps local assets accessible to external production tools.

NC-PoRe organizes collaborative production without prescribing how external tools work internally.

---

# Core Principles

## 1. Record Locally, Produce Together

Recording happens where the person works. The production is shared.

## 2. Failure of One Component Must Not Destroy the Entire Production

A problem affecting one participant must not automatically endanger the whole session.

## 3. The Internet Is a Distributed System

NC-PoRe accounts for latency, connection loss and different devices instead of ignoring these properties.

## 4. User Experience Before Technical Elegance

The internal architecture may be complex. The user experience remains simple.

---

# Consequences

## Advantages

* robust distributed production
* independent clients
* high recording quality
* clear separation of recording and synchronization

## Disadvantages

* synchronization is complex
* clients require local resources
* additional metadata must be managed

These disadvantages are consciously accepted.

---

# Non-Goals

This decision does not mean:

* that a concrete client platform is mandated
* that a specific network transport is required
* that external production tools are replaced
* that all synchronization details are already defined

---

# Guiding Principle

NC-PoRe simplifies distributed podcast production by accepting the reality of distributed systems.

**Record locally. Produce together. Store securely.**
