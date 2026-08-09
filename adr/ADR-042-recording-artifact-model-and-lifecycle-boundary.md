# ADR-042 Recording Artifact Model and Lifecycle Boundary

* Status: Accepted
* Date: 2026-08-01
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

---

# Kontext

Mit ADR-039 wurde die Grenze zwischen fachlichem Recording-Modell und technischer Audioaufnahme definiert.

ADR-040 beschreibt die Koordination des lokalen Recorder-Workflows.

ADR-041 definiert die Trennung zwischen lokalen Aufnahmedaten und deren Speicherung.

Zwischen Capture und Storage fehlt jedoch noch ein zentrales technisches Objekt:

Das Ergebnis einer lokalen Aufnahme.

Dieses Ergebnis soll unabhängig sein von:

- der eigentlichen Audioaufnahme
- der späteren Speicherung
- der Synchronisation
- konkreten Dateiformaten

Es wird daher ein eigenständiges Architekturkonzept benötigt.

---

# Entscheidung

NC-PoRe führt das **Recording Artifact** als eigenständiges technisches Modell ein.

Ein Recording Artifact beschreibt das Ergebnis einer lokalen Aufnahme.

Es ist kein Domänenobjekt.

Es gehört ausschließlich zur Recorder- und Infrastrukturarchitektur.

---

# Architectural Principle

Das Recording Artifact bildet die technische Übergabeeinheit zwischen:

```text
Capture

↓

Recording Artifact

↓

Storage

↓

Synchronization

↓

Export
```

Es repräsentiert das erzeugte Aufnahmeergebnis unabhängig von dessen späterer Verwendung.

---

# Verantwortlichkeiten

Ein Recording Artifact beschreibt:

- welche Aufnahme erzeugt wurde
- welche technischen Bestandteile dazugehören
- welche technischen Metadaten vorhanden sind
- welchen technischen Zustand das Artefakt besitzt

Es beschreibt nicht:

- fachliche Produktionsregeln
- Rollen
- Teilnehmer
- Session-Lebenszyklen
- Geschäftslogik

Diese verbleiben ausschließlich im Core.

---

# Artifact Contents

Ein Recording Artifact kann beispielsweise enthalten:

- Artifact Identifier
- Recording Session Identifier
- Track-Informationen
- technische Metadaten
- Zeitstempel
- Integritätsinformationen

Es enthält bewusst keine fachlichen Produktionsentscheidungen.

---

# Lifecycle

Ein Recording Artifact besitzt einen eigenen technischen Lebenszyklus.

Beispiel:

```text
Created

↓

Capturing

↓

Available

↓

Stored

↓

Synchronized

↓

Archived
```

Zusätzlich können Fehlerzustände auftreten.

Der Lifecycle ist unabhängig vom fachlichen Recording-Lifecycle.

---

# Trennung von Domain und Artifact

Der Core kennt Recording Artifacts nicht.

Der Core kennt ausschließlich:

- Recording
- Production Session
- fachliche Beziehungen

Recorder und Infrastruktur arbeiten dagegen mit Recording Artifacts.

---

# Storage Independence

Ein Recording Artifact beschreibt Daten.

Es entscheidet nicht:

- wo Daten gespeichert werden
- wie Daten gespeichert werden
- welches Dateiformat verwendet wird
- welche Speichertechnologie eingesetzt wird

Diese Verantwortung liegt beim Storage Provider.

---

# Synchronization Independence

Ein Recording Artifact kann synchronisiert werden.

Die Synchronisationslogik gehört jedoch nicht zum Artifact.

Das Artifact beschreibt lediglich den zu synchronisierenden Inhalt.

---

# Export Independence

Exports arbeiten mit Recording Artifacts.

Das Artifact entscheidet jedoch nicht:

- welches Exportformat erzeugt wird
- welche Konvertierung erfolgt
- welche Zielplattform verwendet wird

Exports bleiben eigenständige Komponenten.

---

# Technology Independence

Diese Entscheidung legt keine technische Repräsentation fest.

Nicht Bestandteil dieser ADR sind beispielsweise:

- WAV
- FLAC
- MP3
- Containerformate
- Chunk-Formate
- Datenbanken

Diese werden durch spätere Entscheidungen beschrieben.

---

# Konsequenzen

## Positive Konsequenzen

- klare Trennung zwischen Domain und Infrastruktur
- Capture bleibt unabhängig vom Storage
- Storage bleibt unabhängig vom Export
- Synchronisation arbeitet mit einer stabilen technischen Einheit
- zukünftige Erweiterungen bleiben einfacher

---

## Negative Konsequenzen

- zusätzliches Architekturmodell
- weiterer technischer Lebenszyklus
- zusätzliche Schnittstellen

Diese Nachteile werden bewusst akzeptiert.

---

# Betrachtete Alternativen

## Direct Storage After Capture

Nicht gewählt.

Begründung:

Capture würde direkt von Storage abhängen.

Dies würde die Architekturkopplung erhöhen.

---

## Storage-specific Recording Objects

Nicht gewählt.

Begründung:

Das Recording-Ergebnis würde von einer konkreten Speichertechnologie abhängen.

Dies widerspricht den Architekturprinzipien von NC-PoRe.

---

# Beziehung zu bestehender Architektur

Diese Entscheidung erweitert:

- ADR-039 Recording Architecture and Capture Boundary
- ADR-040 Recorder Workflow and Capture Lifecycle Coordination
- ADR-041 Local Recording Artifact and Storage Boundary

Sie definiert die technische Einheit, die zwischen Capture und Storage übertragen wird.

---

# Zukünftige Entscheidungen

Spätere Entscheidungen werden unter anderem behandeln:

- Artifact-Datenstruktur
- Track-Beschreibung
- Integritätsprüfung
- Hashing
- Chunk-Zuordnung
- Persistenzmodell

Diese Entscheidungen erfolgen unabhängig von dieser Architekturentscheidung.

---

# Status

Diese Entscheidung definiert das Recording Artifact als zentrale technische Übergabeeinheit zwischen Aufnahme, Speicherung, Synchronisation und Export.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# Context

ADR-039 defined the boundary between the domain Recording model and technical audio capture.

ADR-040 describes coordination of the local recorder workflow.

ADR-041 defines the separation between local recording data and storage.

A central technical object is still missing:

The result of a local recording.

This result should remain independent from:

- the actual audio capture
- later storage
- synchronization
- concrete file formats

Therefore an independent architectural concept is required.

---

# Decision

NC-PoRe introduces the **Recording Artifact** as an independent technical model.

A Recording Artifact describes the result of a local recording.

It is not a domain object.

It belongs exclusively to the recorder and infrastructure architecture.

---

# Architectural Principle

The Recording Artifact represents the technical handover unit between:

```text
Capture

↓

Recording Artifact

↓

Storage

↓

Synchronization

↓

Export
```

It represents the generated recording result independently from its later usage.

---

# Responsibilities

A Recording Artifact describes:

- which recording was created
- which technical components belong to it
- which technical metadata exists
- which technical state the artifact has

It does not describe:

- production rules
- roles
- participants
- session lifecycles
- business logic

These remain exclusively inside the Core.

---

# Artifact Contents

A Recording Artifact may contain:

- Artifact Identifier
- Recording Session Identifier
- track information
- technical metadata
- timestamps
- integrity information

It intentionally contains no production decisions.

---

# Lifecycle

A Recording Artifact has its own technical lifecycle.

Example:

```text
Created

↓

Capturing

↓

Available

↓

Stored

↓

Synchronized

↓

Archived
```

Additional error states may exist.

The lifecycle is independent from the domain Recording lifecycle.

---

# Separation from Domain

The Core does not know Recording Artifacts.

The Core only knows:

- Recording
- Production Session
- domain relationships

Recorder and infrastructure components work with Recording Artifacts.

---

# Storage Independence

A Recording Artifact describes data.

It does not decide:

- where data is stored
- how data is stored
- which file format is used
- which storage technology is used

This responsibility belongs to the Storage Provider.

---

# Synchronization Independence

A Recording Artifact can be synchronized.

Synchronization logic is not part of the Artifact.

The Artifact only describes the content that needs to be synchronized.

---

# Export Independence

Exports operate on Recording Artifacts.

The Artifact does not decide:

- which export format is generated
- which conversion is performed
- which target platform is used

Exports remain independent components.

---

# Technology Independence

This decision does not define a technical representation.

The following are explicitly not part of this ADR:

- WAV
- FLAC
- MP3
- container formats
- chunk formats
- databases

These will be defined through later decisions.

---

# Consequences

## Positive Consequences

- clear separation between domain and infrastructure
- capture remains independent from storage
- storage remains independent from export
- synchronization works with a stable technical unit
- future extensions remain easier

---

## Negative Consequences

- additional architectural model
- additional technical lifecycle
- additional interfaces

These disadvantages are consciously accepted.

---

# Alternatives Considered

## Direct Storage After Capture

Rejected.

Reason:

Capture would directly depend on storage.

This would increase architectural coupling.

---

## Storage-specific Recording Objects

Rejected.

Reason:

The recording result would depend on a specific storage technology.

This contradicts NC-PoRe architecture principles.

---

# Relationship to Existing Architecture

This decision extends:

- ADR-039 Recording Architecture and Capture Boundary
- ADR-040 Recorder Workflow and Capture Lifecycle Coordination
- ADR-041 Local Recording Artifact and Storage Boundary

It defines the technical unit transferred between Capture and Storage.

---

# Future Considerations

Future decisions will address topics including:

- Artifact data structure
- track description
- integrity validation
- hashing
- chunk assignment
- persistence model

These decisions will be made independently from this architecture decision.

---

# Status

This decision defines the Recording Artifact as the central technical handover unit between capture, storage, synchronization and export.
