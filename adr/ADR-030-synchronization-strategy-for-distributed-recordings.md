# ADR-030: Synchronization Strategy for Distributed Recordings

* Status: Accepted
* Date: 2026-07-23
* Decision Type: Architecture

---

# Deutsch (English version below)

## Kontext

NC-PoRe wurde als **Nextcloud Podcast Production Environment** definiert.

Die bisherigen Architekturentscheidungen legen fest:

* Die **Production Session** ist die zentrale fachliche Einheit.
* Aufnahmen erfolgen nach dem **Local Recording First** Prinzip.
* Jeder Client erzeugt zunächst eigene lokale Assets.
* Die gemeinsame Produktion entsteht durch Synchronisation und Zusammenarbeit.
* Der Core bleibt unabhängig von konkreten technischen Implementierungen.

ADR-029 hat entschieden, dass NC-PoRe nicht versucht, das Internet wie eine direkte lokale Verbindung zu behandeln.

Stattdessen akzeptiert NC-PoRe die Realität verteilter Systeme:

* unterschiedliche Latenzen
* schwankende Verbindungen
* temporäre Offline-Zustände
* unterschiedliche Plattformen

Damit entsteht die zentrale Frage:

> Wie werden verteilte Aufnahmen zu einer gemeinsamen Production Session zusammengeführt?

---

# Entscheidung

NC-PoRe trennt grundsätzlich zwischen:

1. **Control Synchronization**
2. **Media Synchronization**

Diese beiden Bereiche werden unabhängig voneinander betrachtet.

Leitprinzip:

> NC-PoRe synchronisiert die Produktion, nicht zwangsläufig den Audiostrom.

---

# Control Synchronization

Control Synchronization beschreibt den Zustand und die Steuerung einer Production Session.

Dazu gehören:

* Teilnehmer treten bei
* Aufnahme wird gestartet
* Aufnahme wird beendet
* Marker werden gesetzt
* Session-Zustände ändern sich

Diese Informationen sollen möglichst zeitnah verteilt werden.

Beispiele:

```text id="9wqk2p"
SessionCreated

ParticipantJoined

RecordingStarted

MarkerCreated

RecordingFinished
```

---

# Media Synchronization

Media Synchronization beschreibt die Übertragung und Zusammenführung von Produktionsdaten.

Dazu gehören:

* Audio Assets
* Video Assets (zukünftig)
* Metadaten
* Synchronisationsinformationen

Diese Daten müssen nicht zwingend in Echtzeit übertragen werden.

Priorität:

> Datenintegrität vor Echtzeitillusion.

---

# Architektur

```text id="h4x9qp"
                 Production Session

                       |
              Control Synchronization

        ---------------------------------

        Mac Client       Linux Client      Mobile Client

             |                |                  |

        Local Track      Local Track       Local Track

             |                |                  |

             -------- Media Synchronization --------

                       |

                       ↓

                Shared Production Session
```

---

# Begründung

Eine reine Echtzeit-Audio-Synchronisation über das Internet hätte erhebliche Nachteile:

* Netzwerkprobleme beeinflussen die Aufnahme
* Latenzen sind nicht deterministisch
* mobile Teilnehmer sind besonders betroffen
* Fehler können zum Verlust von Material führen

NC-PoRe verfolgt deshalb ein robustes Modell:

> Jede lokale Aufnahme bleibt zunächst vollständig und unabhängig erhalten.

Erst danach werden Assets Bestandteil einer gemeinsamen Produktion.

---

# Synchronisationsdaten

Jede Aufnahme benötigt ausreichende Informationen, um später korrekt eingeordnet werden zu können.

Beispiel:

```text id="4xj3qb"
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

Für die erste Version gilt:

* lokale Aufnahme auf jedem Client
* lokale Speicherung während der Aufnahme
* Upload nach oder während der Aufnahme
* Zusammenführung über Session- und Synchronisationsdaten

Nicht Bestandteil von V1:

* perfekte automatische Synchronisation aller Spuren
* Echtzeit-Netzwerk-Mixing
* KI-basierte automatische Ausrichtung

---

# Erweiterbarkeit

Die Architektur ermöglicht spätere Erweiterungen:

* automatische Wellenformanalyse
* intelligente Spurausrichtung
* Video-Synchronisation
* professionelle Timecode-Unterstützung
* Live-Monitoring

Diese Funktionen werden nicht verhindert, sondern auf einer stabilen Basis aufgebaut.

---

# Beziehung zu professionellen Workflows

NC-PoRe ersetzt keine spezialisierten Produktionswerkzeuge.

Bestehende Werkzeuge wie:

* Ardour
* Audacity
* weitere Audio- und Videowerkzeuge

bleiben Bestandteil professioneller Workflows.

NC-PoRe organisiert:

* Zusammenarbeit
* Sessions
* Assets
* Synchronisation

Die kreative Produktion bleibt beim Menschen.

---

# Grundprinzipien

## 1. Datenintegrität vor Echtzeitillusion

Eine vollständige Aufnahme ist wichtiger als eine scheinbar perfekte Live-Verbindung.

---

## 2. Kontrollinformationen und Mediendaten sind getrennt

Session-Zustände benötigen schnelle Kommunikation.

Medien benötigen zuverlässige Übertragung.

---

## 3. Verteilte Systeme werden akzeptiert

NC-PoRe arbeitet mit den Eigenschaften des Internets.

Nicht gegen sie.

---

## 4. Einfache Nutzung trotz komplexer Technik

Die interne Architektur darf anspruchsvoll sein.

Der Benutzer soll nur eine funktionierende Produktion erleben.

---

# Konsequenzen

## Vorteile

* robuste Aufnahmen
* geringe Abhängigkeit von Netzwerkqualität
* professionelle Audioqualität
* gute Erweiterbarkeit
* kompatible Workflows

## Nachteile

* Synchronisation benötigt zusätzliche Logik
* mehrere Datenzustände müssen verwaltet werden
* lokale Ressourcen werden benötigt

Diese Nachteile werden bewusst akzeptiert.

---

# Nicht-Ziele

Diese Entscheidung bedeutet nicht:

* dass NC-PoRe sofort eine vollständige Live-Broadcast-Plattform wird
* dass lokale Aufnahmen abgeschafft werden
* dass professionelle Werkzeuge ersetzt werden
* dass alle zukünftigen Synchronisationsverfahren bereits festgelegt sind

---

# Leitgedanke

NC-PoRe macht verteilte Podcast-Produktion zuverlässig, indem es die Realität verteilter Systeme akzeptiert.

**Lokal aufnehmen. Gemeinsam synchronisieren. Professionell produzieren.**

---

# English (Deutsche Version oben)

## Context

NC-PoRe has been defined as a **Nextcloud Podcast Production Environment**.

Previous architecture decisions established:

* The **Production Session** as the central domain entity.
* Recording based on the **Local Recording First** principle.
* Each client initially creates local assets.
* The shared production is created through synchronization and collaboration.
* The Core remains independent from concrete implementations.

ADR-029 decided that NC-PoRe does not treat the Internet as a direct local connection.

Instead, NC-PoRe accepts distributed system realities:

* different latencies
* unstable connections
* temporary offline states
* different platforms

The central question is:

> How are distributed recordings combined into one Production Session?

---

# Decision

NC-PoRe separates:

1. **Control Synchronization**
2. **Media Synchronization**

These areas are treated independently.

Guiding principle:

> NC-PoRe synchronizes the production, not necessarily the audio stream.

---

# Control Synchronization

Control Synchronization manages Production Session state.

Examples:

* participants joining
* recording start
* recording stop
* markers
* session state changes

---

# Media Synchronization

Media Synchronization manages production data:

* audio assets
* future video assets
* metadata
* synchronization information

Real-time transfer is not required.

Priority:

> Data integrity before real-time illusion.

---

# Consequences

Benefits:

* reliable recordings
* lower dependency on network quality
* professional audio quality
* future extensibility
* compatible workflows

Costs:

* synchronization requires additional logic
* multiple states must be managed
* local resources are required

These costs are consciously accepted.

---

# Guiding Principle

NC-PoRe makes distributed podcast production reliable by accepting the reality of distributed systems.

**Record locally. Synchronize together. Produce professionally.**
