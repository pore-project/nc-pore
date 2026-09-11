# ADR-030: Synchronization Strategy for Distributed Recordings

* Status: Accepted
* Date: 2026-07-23
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

## Kontext

NC-PoRe wurde als **Nextcloud Podcast Production Environment** definiert.

Die bisherigen Architekturentscheidungen legen fest:

* Die **Production Session** ist die zentrale fachliche Einheit.
* Aufnahmen erfolgen nach dem **Local Recording First** Prinzip.
* Jeder Client erzeugt zunächst eigene lokale Assets.
* Die gemeinsame Produktion entsteht durch Synchronisation und Zusammenarbeit.
* Der Core bleibt unabhängig von konkreten technischen Implementierungen.

ADR-029 hat entschieden, dass NC-PoRe nicht versucht, das Internet wie eine direkte lokale Verbindung zu behandeln.

Stattdessen akzeptiert NC-PoRe die Realität verteilter Systeme mit unterschiedlichen Latenzen, Verbindungen, temporären Offline-Zuständen und Umgebungen.

Die zentrale Frage lautet:

> Wie werden verteilte Aufnahmen zu einer gemeinsamen Production Session zusammengeführt?

---

# Entscheidung

NC-PoRe trennt grundsätzlich zwischen:

1. **Control Synchronization**
2. **Media Synchronization**

Diese beiden Bereiche werden unabhängig voneinander betrachtet.

Leitprinzip:

> NC-PoRe synchronisiert die Produktion, nicht zwangsläufig den Audiostrom.

Die Trennung ist eine fachliche und technische Verantwortungsgrenze: Der Zustand der gemeinsamen Produktion und die Übertragung der erzeugten Mediendaten dürfen unterschiedliche zeitliche Eigenschaften haben. Das Eintreffen oder Ausbleiben des einen darf nicht automatisch die Gültigkeit des anderen bestimmen.

---

# Control Synchronization

Control Synchronization beschreibt Zustand und Steuerung einer Production Session.

Dazu gehören beispielsweise:

* Teilnehmer treten bei
* Aufnahme wird gestartet
* Aufnahme wird beendet
* Marker werden gesetzt
* Session-Zustände ändern sich

Diese Informationen sollen möglichst zeitnah verteilt werden.

Control Synchronization beschreibt dabei den Zustand und die Koordination der gemeinsamen Produktion. Sie ist nicht die technische Übertragung der eigentlichen Audiodaten.

---

# Media Synchronization

Media Synchronization beschreibt die Übertragung und Zusammenführung von Produktionsdaten.

Dazu gehören:

* Audio Assets
* Metadaten
* Synchronisationsinformationen

Mediendaten müssen nicht zwingend in Echtzeit übertragen werden.

Priorität:

> Datenintegrität vor Echtzeitillusion.

Ein lokal vollständig vorliegendes Recording kann daher bereits gültig sein, obwohl sein Asset noch nicht synchronisiert oder zentral verfügbar ist. Umgekehrt darf ein Synchronisations- oder Transferfehler nicht rückwirkend die lokale Aufnahme ungültig machen.

---

# Architektur

```text
                 Production Session

                       |
              Control Synchronization

        -------------------------------

        Client A         Client B        Client C

            |                |               |

        Local Track      Local Track     Local Track

            |                |               |

            -------- Media Synchronization --------

                       |

                       v

                Shared Production Session
```

---

# Begründung

Eine reine Echtzeit-Audio-Synchronisation über das Internet hätte erhebliche Nachteile:

* Netzwerkprobleme beeinflussen die Aufnahme
* Latenzen sind nicht deterministisch
* unterschiedliche Netzbedingungen erschweren die Verarbeitung
* Fehler können zum Verlust von Material führen

NC-PoRe verfolgt deshalb ein robustes Modell:

> Jede lokale Aufnahme bleibt zunächst vollständig und unabhängig erhalten.

Erst danach werden Assets Bestandteil einer gemeinsamen Produktion.

Die Synchronisationsarchitektur folgt damit derselben Trennung wie die Aufnahmearchitektur: Capture und lokale Datenhaltung sichern das erzeugte Material; Synchronisation stellt die gemeinsame Verfügbarkeit und Zuordnung her. Diese beiden Verantwortlichkeiten dürfen nicht so gekoppelt werden, dass ein Fehler im Transport die lokale Aufnahme zerstört.

---

# Synchronisationsdaten

Jede Aufnahme benötigt ausreichende Informationen, um später korrekt eingeordnet werden zu können.

Beispiel:

```text
Recording Asset

├── Session ID
├── Participant ID
├── Start Timestamp
├── Duration
├── Sample Rate
├── Channel Layout
└── Synchronization Metadata
```

---

# V1 Strategie

Für V1 gilt:

* lokale Aufnahme auf jedem Client
* lokale Speicherung während der Aufnahme
* Upload nach oder während der Aufnahme
* Zusammenführung über Session- und Synchronisationsdaten

Die konkrete technische Ausgestaltung der Synchronisation wird durch die zuständigen technischen Entscheidungen bestimmt.

---

# Beziehung zu professionellen Workflows

NC-PoRe ersetzt keine spezialisierten Produktionswerkzeuge.

Lokale Assets bleiben für externe Audio- und Produktionswerkzeuge zugänglich.

NC-PoRe organisiert:

* Zusammenarbeit
* Sessions
* Assets
* Synchronisation

Die konkrete kreative und technische Arbeit mit externen Werkzeugen bleibt außerhalb dieser ADR.

---

# Grundprinzipien

## 1. Datenintegrität vor Echtzeitillusion

Eine vollständige Aufnahme ist wichtiger als eine scheinbar perfekte Live-Verbindung.

## 2. Kontrollinformationen und Mediendaten sind getrennt

Session-Zustände benötigen zeitnahe Kommunikation. Medien benötigen zuverlässige Übertragung.

## 3. Verteilte Systeme werden akzeptiert

NC-PoRe arbeitet mit den Eigenschaften des Internets, nicht gegen sie.

## 4. Einfache Nutzung trotz komplexer Technik

Die interne Architektur darf anspruchsvoll sein. Der Benutzer soll eine funktionierende Produktion erleben.

---

# Konsequenzen

## Vorteile

* robuste Aufnahmen
* geringe Abhängigkeit von Netzwerkqualität
* hohe Audioqualität
* klare Trennung von Steuerung und Mediendaten

## Nachteile

* Synchronisation benötigt zusätzliche Logik
* mehrere Datenzustände müssen verwaltet werden
* lokale Ressourcen werden benötigt

Diese Nachteile werden bewusst akzeptiert.

---

# Nicht-Ziele

Diese Entscheidung bedeutet nicht:

* dass NC-PoRe eine bestimmte Echtzeitübertragung voraussetzt
* dass lokale Aufnahmen abgeschafft werden
* dass professionelle Werkzeuge ersetzt werden
* dass eine konkrete Synchronisationsimplementierung hier festgelegt wird

---

# Leitgedanke

NC-PoRe macht verteilte Podcast-Produktion zuverlässig, indem es die Realität verteilter Systeme akzeptiert.

**Lokal aufnehmen. Gemeinsam synchronisieren. Professionell produzieren.**

---

# English Version ([Deutsche Version oben](#deutsch))

## Context

NC-PoRe has been defined as a **Nextcloud Podcast Production Environment**.

Previous architecture decisions established:

* The **Production Session** is the central domain entity.
* Recording follows the **Local Recording First** principle.
* Each client initially creates local assets.
* Shared production is created through synchronization and collaboration.
* The Core remains independent from concrete technical implementations.

ADR-029 decided that NC-PoRe does not treat the Internet as a direct local connection.

Instead, NC-PoRe accepts distributed-system realities such as different latencies, connection conditions and temporary offline states.

The central question is:

> How are distributed recordings combined into a shared Production Session?

---

# Decision

NC-PoRe separates:

1. **Control Synchronization**
2. **Media Synchronization**

These areas are treated independently.

Guiding principle:

> NC-PoRe synchronizes the production, not necessarily the audio stream.

This separation is a domain and technical responsibility boundary: the state of the shared production and the transfer of produced media may have different timing characteristics. The arrival or absence of one must not automatically determine the validity of the other.

---

# Control Synchronization

Control Synchronization describes Production Session state and control.

It includes, for example:

* participants joining
* recording start
* recording stop
* marker creation
* session state changes

This information should be distributed with appropriate timeliness.

Control Synchronization describes the state and coordination of the shared production. It is not the technical transfer of the actual audio data.

---

# Media Synchronization

Media Synchronization describes transfer and consolidation of production data.

It includes:

* audio assets
* metadata
* synchronization information

Media data does not have to be transferred in real time.

Priority:

> Data integrity before real-time illusion.

A locally complete recording may therefore already be valid while its asset has not yet been synchronized or made centrally available. Conversely, a synchronization or transfer failure must not retroactively invalidate the local recording.

---

# Architecture

```text
                 Production Session

                       |
              Control Synchronization

        -------------------------------

        Client A         Client B        Client C

            |                |               |

        Local Track      Local Track     Local Track

            |                |               |

            -------- Media Synchronization --------

                       |

                       v

                Shared Production Session
```

---

# Rationale

Pure real-time audio synchronization over the Internet has significant disadvantages:

* network problems affect recording
* latency is not deterministic
* varying network conditions complicate processing
* failures can cause material loss

NC-PoRe therefore follows a robust model:

> Each local recording remains complete and independent first.

Only afterwards do assets become part of the shared production.

The synchronization architecture therefore follows the same separation as the recording architecture: capture and local data persistence secure the produced material; synchronization establishes shared availability and association. These responsibilities must not be coupled in a way that allows a transport failure to destroy the local recording.

---

# Synchronization Data

Each recording requires sufficient information for later correct association and synchronization.

Example:

```text
Recording Asset

├── Session ID
├── Participant ID
├── Start Timestamp
├── Duration
├── Sample Rate
├── Channel Layout
└── Synchronization Metadata
```

---

# V1 Strategy

For V1:

* recording is local on each client
* storage is local during recording
* upload occurs during or after recording
* consolidation uses session and synchronization data

The concrete technical synchronization implementation is defined by the relevant technical decisions.

---

# Relationship to Professional Workflows

NC-PoRe does not replace specialized production tools.

Local assets remain accessible to external audio and production tools.

NC-PoRe organizes:

* collaboration
* sessions
* assets
* synchronization

The concrete creative and technical work in external tools is outside the scope of this ADR.

---

# Core Principles

## 1. Data Integrity Before Real-time Illusion

A complete recording is more important than an apparently perfect live connection.

## 2. Control Information and Media Data Are Separate

Session state requires timely communication. Media requires reliable transfer.

## 3. Distributed Systems Are Accepted

NC-PoRe works with the properties of the Internet rather than against them.

## 4. Simple Use Despite Complex Technology

The internal architecture may be complex. Users should experience a working production.

---

# Consequences

## Advantages

* robust recordings
* lower dependency on network quality
* high audio quality
* clear separation of control and media data

## Disadvantages

* synchronization requires additional logic
* multiple data states must be managed
* local resources are required

These disadvantages are consciously accepted.

---

# Non-Goals

This decision does not mean:

* that NC-PoRe requires a specific real-time transport
* that local recording is replaced
* that professional tools are replaced
* that a concrete synchronization implementation is defined here

---

# Guiding Principle

NC-PoRe makes distributed podcast production reliable by accepting the reality of distributed systems.

**Record locally. Synchronize together. Produce professionally.**
