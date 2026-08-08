# ADR-039 Recording Architecture and Capture Boundary

* Status: Accepted
* Date: 2026-08-08
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

---

# Kontext

NC-PoRe wurde mit dem Ziel entwickelt, verteilte
Podcast-Produktionen mit lokaler Aufnahme zu ermöglichen.

Die bisherigen Architekturentscheidungen definieren:

* lokale Aufnahme als grundlegendes Architekturprinzip
* die Production Session als zentrale fachliche Einheit
* den Core als Domain Authority
* die Trennung von fachlichen und technischen Verantwortlichkeiten
* getrennte Audiospuren pro Teilnehmer
* offene Formate und Interoperabilität
* die Trennung von Control Synchronization und Media Synchronization
* die Möglichkeit, technische Komponenten und Provider auszutauschen

Mit der technischen Umsetzung des Core und des Recorders
wird die Grenze zwischen fachlichem Recording-Modell und
technischer Aufnahme konkret relevant.

Dabei müssen mehrere unterschiedliche Konzepte voneinander
getrennt werden:

* das fachliche Recording
* die technische Audioaufnahme
* das daraus entstehende Recording Artifact
* die lokale Verwaltung und Persistenz des Artifacts
* die spätere Verarbeitung und Synchronisation

Diese Konzepte haben unterschiedliche Verantwortlichkeiten
und dürfen daher nicht zu einer einzigen technischen oder
fachlichen Einheit zusammengefasst werden.

Insbesondere ist ein Recording nicht mit einer Audiodatei
gleichzusetzen.

Ein Recording beschreibt ein fachliches Produktionsobjekt.

Die während einer Aufnahme erzeugten Audiodaten sind ein
technisches Ergebnis dieses Vorgangs.

---

# Entscheidung

NC-PoRe trennt das fachliche Recording-Modell von der
technischen Capture-Implementierung.

Der Core beschreibt das fachliche Recording.

Die Capture-Schicht erzeugt daraus technische
Aufnahmedaten.

Die erzeugten Daten werden als Recording Artifact
behandelt und über definierte technische Grenzen an
Registry, Persistence und weitere Verarbeitungskomponenten
übergeben.

Damit entstehen folgende klar getrennte Verantwortungen:

```text
Domain

Production Session
        |
        v
Recording
        |
        | fachliche Beziehung
        v
Technical Recording Flow
        |
        v
Capture Boundary
        |
        v
Recording Artifact
        |
        +--------------------+
        |                    |
        v                    v
Artifact Registry      Persistence
        |
        v
Artifact Processing
```

Die konkrete technische Implementierung kann diese
Grenzen unterschiedlich realisieren.

Die fachlichen Verantwortlichkeiten bleiben davon
unabhängig.

---

# Recording als fachliches Produktionsobjekt

Ein `Recording` ist Bestandteil des fachlichen
Produktionsmodells.

Das Recording beschreibt insbesondere:

* dass eine Aufnahme innerhalb einer Production Session
  vorgesehen oder erfolgt ist
* zu welcher Production Session das Recording gehört
* welche fachlichen Zustände für das Recording gelten
* welche fachlichen Beziehungen zu Teilnehmern bestehen
* welche fachlichen Metadaten für das Recording relevant sind

Der Core ist für die fachliche Integrität dieses Modells
verantwortlich.

Ein Recording ist daher nicht lediglich ein Verweis auf
eine Datei.

Es besitzt eine eigene fachliche Identität und einen
eigenen fachlichen Lebenszyklus.

---

# Verantwortung des Core

Der Core ist verantwortlich für die fachliche Bedeutung
eines Recordings.

Dazu gehören insbesondere:

* Recording-Lebenszyklus
* Beziehung zwischen Recording und Production Session
* fachliche Beziehung zwischen Recording und Participant
* fachliche Validierung
* erlaubte fachliche Zustandsübergänge
* fachliche Recording-Metadaten
* fachliche Ereignisse im Zusammenhang mit Recordings

Der Core entscheidet beispielsweise:

* ob ein Recording innerhalb einer Session gültig ist
* welchem Teilnehmer ein Recording fachlich zugeordnet ist
* ob ein Recording gestartet, beendet oder abgeschlossen
  werden darf
* welche fachlichen Zustände für ein Recording erlaubt sind

Der Core entscheidet dagegen nicht, wie Audiodaten
technisch erzeugt oder gespeichert werden.

---

# Nicht-Verantwortung des Core

Der Core enthält ausdrücklich keine konkrete technische
Audioaufnahme.

Dazu gehören insbesondere:

* Audio-Hardware-Ansteuerung
* Mikrofonzugriffe
* plattformspezifische Audio-APIs
* konkrete Audio-Backends
* Audio-Buffer im Capture-Prozess
* Echtzeit-Audioverarbeitung
* konkrete Dateischreibvorgänge
* konkrete Filesystem-Operationen
* technische Artifact-Verwaltung

Diese Verantwortlichkeiten gehören zu technischen
Komponenten außerhalb des Domain-Modells.

---

# Capture als technische Operation

Die Audioaufnahme ist eine technische Operation.

Die Capture-Komponente ist dafür verantwortlich,
Audiodaten von einer technischen Audioquelle zu erfassen.

Dazu gehören insbesondere:

* Zugriff auf verfügbare Audioquellen
* Initialisierung des Audio-Captures
* Erfassung des Audio-Streams
* technische Pufferung während des Captures
* Übergabe erfasster Audiodaten an den Recording Workflow
* technische Behandlung von Capture-Fehlern

Die Capture-Komponente kennt die technische
Repräsentation der Audioaufnahme.

Sie definiert jedoch nicht die fachlichen Regeln
des Recordings.

Sie entscheidet insbesondere nicht über:

* Production Session Regeln
* Benutzerrollen
* fachliche Berechtigungen
* fachliche Recording-Zustände
* Produktionsabläufe

---

# Capture Boundary

Die Grenze zwischen fachlichem Recording und technischer
Audioaufnahme wird durch eine definierte Capture Boundary
gebildet.

Vereinfacht:

```text
Core

Recording Lifecycle
        |
        v
Capture Boundary
        |
        v
Capture Implementation
        |
        v
Audio Backend
        |
        v
Audio Source
```

Der Core verwendet damit eine definierte technische
Grenze, ohne von einem konkreten Audio-Backend abhängig
zu werden.

Die Capture-Implementierung kann daher ausgetauscht
werden, ohne die fachliche Recording-Logik neu zu
implementieren.

---

# Recording Artifact

Das Ergebnis der technischen Aufnahme wird nicht direkt
zum fachlichen Recording.

Es entsteht ein technisches `Recording Artifact`.

Ein Artifact repräsentiert die tatsächlich erzeugten
technischen Aufnahmedaten.

Beispielsweise kann ein Artifact enthalten:

* eine lokale Audiodatei
* technische Dateiinformationen
* technische Aufnahmeeigenschaften
* Informationen über die zugehörige Session
* Informationen über den zugehörigen Participant
* Informationen über die Entstehung des Artifacts

Die genaue Struktur und der Lebenszyklus von Artifacts
werden durch die entsprechenden technischen
Architekturentscheidungen definiert.

Insbesondere wird hier nicht festgelegt, wie Artifacts
persistiert, registriert oder verarbeitet werden.

Diese Verantwortlichkeiten liegen außerhalb dieser ADR.

---

# Trennung von Recording und Artifact

Die zentrale Trennung lautet:

```text
Recording

= fachliches Produktionsobjekt


Recording Artifact

= technische Repräsentation
  tatsächlich erzeugter Aufnahmedaten
```

Ein Recording kann daher fachlich existieren, ohne dass
bereits ein fertiges Artifact vorhanden ist.

Umgekehrt ist ein Artifact technisch identifizierbar,
ohne selbst die fachliche Bedeutung eines Recordings
zu definieren.

Die Verbindung zwischen beiden wird über definierte
fachliche beziehungsweise technische Referenzen hergestellt.

Dadurch bleibt die Domain unabhängig von:

* Dateiformaten
* Dateinamen
* konkreten Speicherorten
* Filesystemen
* Storage Providern
* Audio-Backends

---

# Artifact Registry und Persistence

Die technische Verwaltung eines Recording Artifacts
ist nicht Aufgabe des Capture Providers.

Nach seiner Erzeugung wird ein Artifact über die dafür
definierten technischen Grenzen weitergegeben.

Dabei bleiben insbesondere folgende Verantwortlichkeiten
getrennt:

```text
Capture

erzeugt Aufnahmedaten

        ↓

Artifact

repräsentiert die technischen Daten

        ↓

Artifact Registry

kennt vorhandene Artifacts

        ↓

Persistence

speichert und lädt Artifacts

        ↓

Artifact Processing

verarbeitet vorhandene Artifacts
```

Die Capture-Komponente muss daher nicht gleichzeitig:

* Registry-Verantwortung übernehmen
* Persistenzlogik implementieren
* Artifact Discovery durchführen
* Artifact Processing durchführen

Diese Trennung verhindert, dass der Capture-Code zu einer
zentralen technischen Sammelkomponente wird.

---

# Local Recording Principle

Die Aufnahme folgt weiterhin dem grundlegenden
Architekturprinzip aus ADR-001 und ADR-029:

```text
Lokal aufnehmen

↓

Aufnahmedaten lokal sichern

↓

Aufnahme fachlich abschließen

↓

Artifact kontrolliert weiterverarbeiten

↓

Später synchronisieren
```

Eine laufende Audioaufnahme darf nicht von einer
stabilen Netzwerkverbindung abhängig sein.

Die technische Capture-Komponente muss daher lokal
arbeitsfähig sein.

Insbesondere darf der Ausfall der Netzwerkverbindung
während des Captures nicht dazu führen, dass die lokale
Audioaufnahme technisch abgebrochen werden muss.

Die konkrete Persistenz und Recovery der lokalen
Aufnahmedaten wird durch die dafür definierten technischen
Grenzen behandelt.

---

# Track Model

NC-PoRe verwendet getrennte Audiospuren als Grundlage
der Aufnahmearchitektur.

Ein technischer Capture Workflow kann daher mehrere
lokale Tracks erzeugen.

Beispiele:

* Host Track
* Guest Track
* Co-Host Track
* weitere Participant Tracks

Ein Track ist dabei ein technisches Aufnahmeergebnis
und nicht selbst das fachliche Recording.

Die fachliche Zuordnung eines Recordings zu einer
Production Session und zu Participants erfolgt im
Domain-Modell.

Die technische Repräsentation der einzelnen Tracks,
ihre Speicherung und ihre spätere Synchronisation
werden durch nachgelagerte Architekturentscheidungen
definiert.

---

# Recording Workflow

Der technische Ablauf wird konzeptionell als
Zusammenspiel mehrerer Grenzen betrachtet:

```text
Production Session
        |
        v
Recording Domain Object
        |
        v
Recording Workflow
        |
        v
Capture Boundary
        |
        v
Capture Implementation
        |
        v
Recording Artifact
        |
        v
Artifact Registry
        |
        v
Persistence
        |
        v
Artifact Processing
        |
        v
Synchronization / Export
```

Nicht jede Implementierung muss diese Schritte als
separate Softwarekomponenten ausführen.

Die Verantwortlichkeiten müssen jedoch getrennt bleiben.

---

# Fehler- und Ausfallverhalten

Fehler in einer technischen Komponente dürfen nicht
automatisch die fachliche Bedeutung des Recordings
zerstören.

Beispielsweise sind unterschiedliche Situationen
möglich:

```text
Capture erfolgreich
        |
        v
Artifact erzeugt
        |
        v
Persistence fehlgeschlagen
```

oder:

```text
Recording fachlich beendet
        |
        v
Artifact vorhanden
        |
        v
Synchronisation noch nicht erfolgt
```

Diese Zustände sind technisch unterschiedlich und
müssen entsprechend behandelt werden.

Insbesondere bedeutet ein fehlgeschlagener Upload
nicht automatisch, dass das lokale Recording verloren
ist.

Die Recovery- und Konsistenzregeln für solche Situationen
werden durch die entsprechenden technischen
Architekturentscheidungen definiert.

---

# Interface Boundary

Die Kommunikation zwischen Core und Capture erfolgt
über definierte Schnittstellen.

Konzeptionell:

```text
Core

Recording Operation

        |
        v

Capture Boundary

        |
        v

Capture Provider

        |
        v

Audio Backend
```

Der Core kennt dabei keine konkrete Audio-Technologie.

Ebenso kennt der Capture Provider keine fachliche
Implementierung der Production Session.

Die Schnittstelle bildet damit eine echte
Verantwortungsgrenze und nicht lediglich eine
technische Abstraktion um eine konkrete Bibliothek.

---

# Technology Independence

Die Auswahl konkreter Audio-Technologien ist nicht
Bestandteil dieser Entscheidung.

Mögliche spätere Entscheidungen betreffen beispielsweise:

* Audio Backend
* Plattformintegration
* Audio Device Handling
* Codec-Unterstützung
* interne Audio-Datenstrukturen
* Dateiformate
* Chunking
* Echtzeitverarbeitung
* technische Track-Repräsentation

Diese Entscheidungen müssen die hier definierten
Verantwortungsgrenzen respektieren.

Eine konkrete Technologie darf insbesondere nicht
dazu führen, dass technische Audioabhängigkeiten in
das fachliche Recording-Modell gelangen.

---

# Consequences

## Positive Consequences

* fachliche Recording-Logik bleibt unabhängig von
  Audio-Technologie
* Capture-Implementierungen können ausgetauscht werden
* Core-Tests benötigen keine Audio-Hardware
* technische Audiofehler bleiben außerhalb der Domain
* Artifacts können unabhängig von ihrer Entstehung
  verwaltet werden
* Registry und Persistence bleiben unabhängig von Capture
* verschiedene Clients können eigene Capture Provider
  verwenden
* lokale Aufnahme bleibt unabhängig von
  Netzwerkverfügbarkeit
* die Architektur unterstützt Recovery und spätere
  Synchronisation
* Verantwortungsgrenzen bleiben nachvollziehbar

---

## Negative Consequences

* zusätzliche Schnittstellen müssen definiert werden
* Recording Workflow und Capture müssen koordiniert werden
* Artifact-Erzeugung benötigt eigene technische Regeln
* Capture, Registry und Persistence müssen miteinander
  integriert werden
* Fehlerzustände müssen über mehrere technische Grenzen
  hinweg nachvollziehbar behandelt werden
* die technische Umsetzung ist komplexer als eine direkte
  Audioaufnahme innerhalb des Domain-Modells

Diese Nachteile werden bewusst akzeptiert.

Sie entstehen aus der notwendigen Trennung fachlicher
und technischer Verantwortlichkeiten und sind Teil der
Architekturentscheidung.

---

# Alternatives Considered

## Audio Recording Inside the Core

Nicht gewählt.

Begründung:

Dies würde technische Audioabhängigkeiten in die
fachliche Domäne einführen.

Der Core würde dadurch von Hardware,
Audio-Backends, Betriebssystemen und konkreten
Dateiformaten abhängig.

Dies verletzt die in ADR-033 und ADR-034 definierten
Architekturprinzipien.

---

## Client-Owned Recording Without Domain Model

Nicht gewählt.

Begründung:

Eine reine Client-Implementierung würde das fachliche
Recording aus dem zentralen Domain-Modell entfernen.

Dadurch würden:

* Recording-Zustände
* Session-Beziehungen
* fachliche Validierungen
* Teilnehmerzuordnungen

teilweise in technische Clients verlagert.

Dies würde die Core-Verantwortung als Domain Authority
schwächen.

---

## Capture Provider Owns Artifact Persistence

Nicht gewählt.

Begründung:

Wenn der Capture Provider gleichzeitig für die
dauerhafte Speicherung verantwortlich wäre, würden
Capture und Persistence unnötig gekoppelt.

Dadurch würden technische Audioaufnahme und
Storage-Technologie zu einer gemeinsamen
Verantwortlichkeit.

NC-PoRe trennt daher:

```text
Capture

≠

Artifact Registry

≠

Persistence
```

---

## Recording as Audio File

Nicht gewählt.

Begründung:

Eine Audiodatei ist ein technisches Ergebnis einer
Aufnahme und nicht die vollständige fachliche
Repräsentation eines Recordings.

Ein fachliches Recording benötigt insbesondere:

* Identität
* Session-Beziehung
* fachlichen Lebenszyklus
* fachliche Metadaten
* Teilnehmerbeziehung

Diese Informationen dürfen nicht an eine konkrete
Dateirepräsentation gebunden werden.

---

# Relationship to Existing Architecture

Diese Entscheidung erweitert und konkretisiert:

* ADR-001 Local Recording as Fundamental Architecture Principle
* ADR-002 Audio Format and Track Concept
* ADR-015 Initial Architecture of the NC-PoRe Recorder Client
* ADR-018 Recorder Data Flow and Processing Pipeline
* ADR-019 Recording Session Data Model
* ADR-029 Distributed Recording Architecture
* ADR-033 Core Architecture
* ADR-034 Implementation Architecture
* ADR-035 Domain Lifecycle and State Transition Management
* ADR-038 Core Implementation Structure and Module Organization

Die Entscheidung bildet insbesondere die Grenze zwischen
fachlichem Recording und technischer Aufnahme explizit ab.

Die inzwischen separat definierten technischen Grenzen
für Artifact Management, Persistence, Processing und
Recovery ergänzen diese Entscheidung, ersetzen sie jedoch
nicht.

---

# Future Considerations

Weitere Entscheidungen werden separat behandelt:

* konkrete Audio-Backend-Auswahl
* Capture Provider Architektur
* Plattformintegration
* Audio Device Management
* lokale Chunk-Speicherung
* konkrete Artifact-Struktur
* Artifact Lifecycle
* Artifact Registry
* Persistence Provider
* Artifact Processing
* Recovery und Konsistenz
* Track-Synchronisation
* Media Synchronization
* Exportformate

Diese Entscheidungen erfolgen erst bei konkretem
technischem Bedarf.

Sie müssen die in dieser ADR definierte Trennung
zwischen Domain, Capture und technischen Artifact-
Komponenten einhalten.

---

# Status

Diese Entscheidung definiert die grundlegende
Architekturgrenze zwischen dem fachlichen Recording
innerhalb des NC-PoRe Core und der technischen
Audioaufnahme.

Sie definiert außerdem die fachliche und technische
Trennung zwischen:

```text
Recording
    |
    | domain
    v
Capture
    |
    | technical
    v
Recording Artifact
    |
    +---- Artifact Registry
    |
    +---- Persistence
    |
    +---- Artifact Processing
```

Die konkrete technische Aufnahmeimplementierung,
Artifact-Verwaltung, Persistenz, Verarbeitung,
Recovery und Synchronisation werden durch separate
Architekturentscheidungen und Implementierungen
festgelegt.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# Context

NC-PoRe was designed to enable distributed podcast
production with local recording.

Previous architecture decisions define:

* local recording as a fundamental architecture principle
* Production Session as the central domain entity
* the Core as Domain Authority
* separation of domain and technical responsibilities
* separate audio tracks per participant
* open formats and interoperability
* separation of Control Synchronization and Media Synchronization
* replaceable technical components and providers

With the implementation of the Core and Recorder,
the boundary between the domain Recording model and
technical capture becomes concrete.

Several distinct concepts must be kept separate:

* the domain Recording
* technical audio capture
* the resulting Recording Artifact
* local Artifact management and persistence
* subsequent processing and synchronization

These concepts have different responsibilities and must
therefore not be collapsed into a single domain or
technical component.

In particular, a Recording is not the same thing as an
audio file.

A Recording is a domain production object.

The audio data generated during capture is a technical
result of that operation.

---

# Decision

NC-PoRe separates the domain Recording model from
technical capture implementation.

The Core defines the domain Recording.

The Capture layer produces technical recording data.

The resulting data is represented as a Recording Artifact
and passed through defined technical boundaries to the
Registry, Persistence and further processing components.

The responsibilities are therefore separated as follows:

```text
Domain

Production Session
        |
        v
Recording
        |
        | domain relationship
        v
Technical Recording Flow
        |
        v
Capture Boundary
        |
        v
Recording Artifact
        |
        +--------------------+
        |                    |
        v                    v
Artifact Registry      Persistence
        |
        v
Artifact Processing
```

The concrete technical implementation may realize these
boundaries in different ways.

The domain responsibilities remain independent of that
implementation.

---

# Recording as a Domain Production Object

A `Recording` is part of the domain production model.

The Recording describes in particular:

* that a recording exists or is intended within a
  Production Session
* which Production Session it belongs to
* which domain states apply to the Recording
* which domain relationships exist with Participants
* which domain metadata is relevant to the Recording

The Core is responsible for the domain integrity of this
model.

A Recording is therefore not merely a reference to a file.

It has its own domain identity and domain lifecycle.

---

# Core Responsibility

The Core is responsible for the domain meaning of a
Recording.

This includes in particular:

* Recording lifecycle
* relationship between Recording and Production Session
* domain relationship between Recording and Participant
* domain validation
* allowed domain state transitions
* domain Recording metadata
* domain events related to Recordings

The Core decides, for example:

* whether a Recording is valid within a Session
* which Participant a Recording belongs to
* whether a Recording may be started, stopped or completed
* which domain states are valid for a Recording

The Core does not decide how audio data is technically
generated or stored.

---

# Core Non-Responsibilities

The Core explicitly contains no concrete audio capture
implementation.

This includes:

* audio hardware access
* microphone access
* platform-specific audio APIs
* concrete audio backends
* capture-process buffers
* realtime audio processing
* concrete file-writing operations
* concrete filesystem operations
* technical Artifact management

These responsibilities belong to technical components
outside the domain model.

---

# Capture as a Technical Operation

Audio capture is a technical operation.

The Capture component is responsible for acquiring audio
data from a technical audio source.

This includes in particular:

* access to available audio sources
* capture initialization
* audio stream acquisition
* technical buffering during capture
* passing captured data into the Recording workflow
* technical handling of capture failures

The Capture component knows the technical representation
of the audio recording.

It does not define the domain rules of the Recording.

It does not decide:

* Production Session rules
* user roles
* domain permissions
* domain Recording states
* production workflows

---

# Capture Boundary

The boundary between the domain Recording and technical
audio capture is defined by a Capture Boundary.

Conceptually:

```text
Core

Recording Lifecycle
        |
        v
Capture Boundary
        |
        v
Capture Implementation
        |
        v
Audio Backend
        |
        v
Audio Source
```

The Core therefore uses a defined technical boundary
without depending on a concrete audio backend.

The Capture implementation can consequently be replaced
without reimplementing domain Recording logic.

---

# Recording Artifact

The result of technical capture is not itself the
domain Recording.

It produces a technical `Recording Artifact`.

An Artifact represents the actual technical recording
data that was produced.

For example, an Artifact may contain:

* a local audio file
* technical file information
* technical recording properties
* information about the associated Session
* information about the associated Participant
* information about the creation of the Artifact

The exact structure and lifecycle of Artifacts are defined
by the corresponding technical architecture decisions.

This ADR does not define how Artifacts are persisted,
registered or processed.

Those responsibilities remain outside this ADR.

---

# Separation of Recording and Artifact

The central distinction is:

```text
Recording

= domain production object


Recording Artifact

= technical representation
  of actually produced recording data
```

A Recording may therefore exist as a domain object before
a completed Artifact exists.

Conversely, an Artifact can be technically identifiable
without defining the domain meaning of a Recording.

The relationship between both is established through
defined domain and technical references.

This keeps the domain independent from:

* file formats
* file names
* concrete storage locations
* filesystems
* Storage Providers
* audio backends

---

# Artifact Registry and Persistence

Technical management of a Recording Artifact is not the
responsibility of the Capture Provider.

After creation, an Artifact is passed through the
defined technical boundaries.

The following responsibilities remain separate:

```text
Capture

produces recording data

        ↓

Artifact

represents the technical data

        ↓

Artifact Registry

knows existing Artifacts

        ↓

Persistence

stores and loads Artifacts

        ↓

Artifact Processing

processes existing Artifacts
```

The Capture component therefore does not also need to:

* own Registry responsibility
* implement persistence logic
* perform Artifact discovery
* perform Artifact processing

This separation prevents Capture code from becoming a
central technical aggregation component.

---

# Local Recording Principle

Recording continues to follow the fundamental architecture
principle defined by ADR-001 and ADR-029:

```text
Record locally

↓

Secure recording data locally

↓

Complete the recording at the domain level

↓

Process the Artifact in a controlled manner

↓

Synchronize later
```

A running audio recording must not depend on a stable
network connection.

The technical Capture component must therefore be able
to operate locally.

In particular, a network outage during capture must not
require the local audio recording to be technically
aborted.

The concrete persistence and recovery of local recording
data are handled by the corresponding technical boundaries.

---

# Track Model

NC-PoRe uses separate audio tracks as the basis of the
recording architecture.

A technical Capture workflow may therefore produce
multiple local tracks.

Examples:

* Host Track
* Guest Track
* Co-Host Track
* additional Participant Tracks

A Track is a technical recording result and is not itself
the domain Recording.

The domain relationship between a Recording,
Production Session and Participants is defined by the
domain model.

The technical representation of individual tracks,
their storage and their later synchronization are defined
by subsequent architecture decisions.

---

# Recording Workflow

The technical flow is conceptually a cooperation between
multiple boundaries:

```text
Production Session
        |
        v
Recording Domain Object
        |
        v
Recording Workflow
        |
        v
Capture Boundary
        |
        v
Capture Implementation
        |
        v
Recording Artifact
        |
        v
Artifact Registry
        |
        v
Persistence
        |
        v
Artifact Processing
        |
        v
Synchronization / Export
```

An implementation does not have to represent every step
as a separate software component.

The responsibilities, however, must remain separate.

---

# Error and Failure Handling

Failures in a technical component must not automatically
destroy the domain meaning of the Recording.

For example, different situations may occur:

```text
Capture successful
        |
        v
Artifact created
        |
        v
Persistence failed
```

or:

```text
Recording completed at domain level
        |
        v
Artifact exists
        |
        v
Synchronization not yet performed
```

These are technically different situations and must be
handled accordingly.

In particular, a failed upload does not automatically
mean that the local Recording has been lost.

Recovery and consistency rules for such situations are
defined by the corresponding technical architecture
decisions.

---

# Interface Boundary

Communication between the Core and Capture occurs through
defined interfaces.

Conceptually:

```text
Core

Recording Operation

        |
        v

Capture Boundary

        |
        v

Capture Provider

        |
        v

Audio Backend
```

The Core does not know any concrete audio technology.

Likewise, the Capture Provider does not contain the
domain implementation of the Production Session.

The interface therefore represents a real responsibility
boundary, not merely a technical abstraction around a
specific library.

---

# Technology Independence

The selection of concrete audio technologies is outside
the scope of this decision.

Future decisions may concern, for example:

* audio backend
* platform integration
* audio device handling
* codec support
* internal audio data structures
* file formats
* chunking
* realtime processing
* technical track representation

These decisions must respect the responsibility
boundaries defined here.

In particular, a concrete technology must not introduce
technical audio dependencies into the domain Recording
model.

---

# Consequences

## Positive Consequences

* domain Recording logic remains independent of audio technology
* Capture implementations can be replaced
* Core tests do not require audio hardware
* technical audio failures remain outside the domain
* Artifacts can be managed independently of their origin
* Registry and Persistence remain independent of Capture
* different clients can use their own Capture Providers
* local recording remains independent of network availability
* the architecture supports recovery and later synchronization
* responsibility boundaries remain traceable

---

## Negative Consequences

* additional interfaces have to be defined
* Recording Workflow and Capture have to be coordinated
* Artifact creation requires its own technical rules
* Capture, Registry and Persistence have to be integrated
* failure states must remain traceable across several
  technical boundaries
* the technical implementation is more complex than
  direct audio recording inside the domain model

These disadvantages are deliberately accepted.

They result from the necessary separation of domain and
technical responsibilities and are part of the architecture
decision.

---

# Alternatives Considered

## Audio Recording Inside the Core

Rejected.

Reason:

This would introduce technical audio dependencies into
the domain.

The Core would become dependent on hardware,
audio backends, operating systems and concrete file
formats.

This violates the architecture principles defined in
ADR-033 and ADR-034.

---

## Client-Owned Recording Without Domain Model

Rejected.

Reason:

A pure client implementation would remove the domain
Recording from the central domain model.

This would move:

* Recording states
* Session relationships
* domain validation
* Participant relationships

partly into technical clients.

This would weaken the Core's responsibility as
Domain Authority.

---

## Capture Provider Owns Artifact Persistence

Rejected.

Reason:

If the Capture Provider were also responsible for
persistent storage, Capture and Persistence would become
unnecessarily coupled.

Technical audio capture and Storage technology would
become one responsibility.

NC-PoRe therefore separates:

```text
Capture

≠

Artifact Registry

≠

Persistence
```

---

## Recording as Audio File

Rejected.

Reason:

An audio file is a technical result of a recording
operation and not the complete domain representation
of a Recording.

A domain Recording requires in particular:

* identity
* Session relationship
* domain lifecycle
* domain metadata
* Participant relationship

These concepts must not be tied to a concrete file
representation.

---

# Relationship to Existing Architecture

This decision extends and clarifies:

* ADR-001 Local Recording as Fundamental Architecture Principle
* ADR-002 Audio Format and Track Concept
* ADR-015 Initial Architecture of the NC-PoRe Recorder Client
* ADR-018 Recorder Data Flow and Processing Pipeline
* ADR-019 Recording Session Data Model
* ADR-029 Distributed Recording Architecture
* ADR-033 Core Architecture
* ADR-034 Implementation Architecture
* ADR-035 Domain Lifecycle and State Transition Management
* ADR-038 Core Implementation Structure and Module Organization

This decision explicitly defines the boundary between
domain Recording and technical capture.

The technical boundaries subsequently defined for
Artifact Management, Persistence, Processing and Recovery
complement this decision but do not replace it.

---

# Future Considerations

Further decisions will be handled separately:

* concrete audio backend selection
* Capture Provider architecture
* platform integration
* audio device management
* local chunk storage
* concrete Artifact structure
* Artifact lifecycle
* Artifact Registry
* Persistence Provider
* Artifact Processing
* Recovery and consistency
* Track synchronization
* Media Synchronization
* export formats

These decisions will be made when a concrete technical
requirement exists.

They must preserve the separation between Domain, Capture
and technical Artifact components defined by this ADR.

---

# Status

This decision defines the fundamental architecture
boundary between the domain Recording within the
NC-PoRe Core and technical audio capture.

It also defines the domain and technical separation
between:

```text
Recording
    |
    | domain
    v
Capture
    |
    | technical
    v
Recording Artifact
    |
    +---- Artifact Registry
    |
    +---- Persistence
    |
    +---- Artifact Processing
```

The concrete technical recording implementation,
Artifact management, persistence, processing, recovery
and synchronization are defined through separate
architecture decisions and implementations.
