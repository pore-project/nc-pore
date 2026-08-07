# ADR-052: Local Filesystem Persistence Provider

* Status: Accepted
* Date: 2026-08-07

---

# Deutsch ([English version below](#english-version))

---

# Kontext

NC-PoRe besitzt mit der Persistence Boundary bereits eine definierte technische Grenze zwischen Recorder-Logik und konkreter Speicherung.

Die bisherige Implementierung `InMemoryPersistenceProvider` dient der Validierung der Architektur und der automatisierten Tests.

Für eine produktionsfähige Anwendung wird jedoch eine persistente lokale Speicherung benötigt.

Die nächste technische Erweiterung ist daher eine konkrete Implementierung der bestehenden Persistence-Schnittstelle.

Diese Entscheidung betrifft ausschließlich die technische Umsetzung der Speicherung.

Die Architekturgrenze selbst bleibt unverändert.

---

# Entscheidung

NC-PoRe führt einen `FilesystemPersistenceProvider` ein.

Der `FilesystemPersistenceProvider`:

* implementiert das bestehende `PersistenceProvider` Interface
* ersetzt die InMemory-Implementierung nicht
* bleibt eine austauschbare Implementierung hinter der Persistence Boundary
* wird als erste produktionsnahe lokale Speicherung verwendet

Der Recorder arbeitet weiterhin ausschließlich gegen die definierte Persistence-Schnittstelle.

Das Dateisystem ist eine Implementierungsentscheidung und kein Bestandteil der Recorder-Architektur.

---

# Verantwortlichkeiten

Der `FilesystemPersistenceProvider` ist verantwortlich für:

* Speichern von RecordingArtifacts
* Laden von RecordingArtifacts
* Auflisten vorhandener RecordingArtifacts
* Entfernen von RecordingArtifacts

Die Implementierung kapselt sämtliche notwendigen Dateisystemzugriffe.

---

# Nicht enthalten

Dieser ADR entscheidet bewusst nicht über:

* konkretes Verzeichnislayout
* Dateinamenkonventionen
* Dateiformate
* Metadatenstrukturen außerhalb des RecordingArtifacts
* Synchronisation
* Recovery-Mechanismen
* Konfliktbehandlung
* Verschlüsselung
* Kompression
* Cloud Storage
* Datenbankpersistenz

Diese Themen werden in separaten Entscheidungen behandelt, sobald ein konkreter Bedarf besteht.

---

# Begründung

Die Persistence Boundary wurde bewusst vor der konkreten Storage-Technologie eingeführt.

Dadurch bleiben folgende Eigenschaften erhalten:

* Recorder Workflow kennt keine Speichertechnologie.
* Artifact Processing kennt keine Speichertechnologie.
* Application Flow übernimmt keine Persistenzverantwortung.
* Tests können weiterhin mit InMemoryPersistenceProvider ausgeführt werden.
* Weitere Storage-Implementierungen können später ergänzt werden.

Das Dateisystem wird als austauschbare technische Umsetzung betrachtet.

---

# Konsequenzen

## Positive Konsequenzen

* NC-PoRe erhält eine erste echte persistente Speicherung.
* Die bestehende Architektur bleibt unverändert.
* Bestehende Tests können weiterhin ohne Dateisystemabhängigkeit laufen.
* Die Persistence-Implementierung bleibt austauschbar.
* Produktionsnahe Tests werden möglich.

## Negative Konsequenzen

* Zwei Persistence-Implementierungen müssen gepflegt werden.
* Dateisystemzugriffe benötigen eigene Tests.
* Fehlerfälle der lokalen Speicherung müssen später berücksichtigt werden.

---

# Betrachtete Alternativen

## Direkte Dateisystemzugriffe im Recorder

Verworfen.

Die Speicherung würde dadurch in Workflow- oder Application-Schichten gelangen und die bestehende Architekturgrenze verletzen.

---

## Speicherung direkt im Artifact Processing

Verworfen.

Artifact Processing ist für die technische Verarbeitung des Ergebnisses verantwortlich, nicht für Storage-Details.

---

## Datenbank als erste Persistenzimplementierung

Nicht gewählt.

Eine Datenbank kann später sinnvoll sein, ist aber für die erste lokale Persistenzschicht nicht erforderlich.

---

## Cloud Storage als erste Persistenzimplementierung

Nicht gewählt.

Lokale Aufnahme und lokale Verfügbarkeit bleiben zentrale Prinzipien von NC-PoRe.

---

# Beziehung zu anderen ADRs

Dieser ADR baut auf folgenden Entscheidungen auf:

* ADR-043 Local Recording Persistence Boundary
* ADR-044 Persistence Provider Interface
* ADR-048 Artifact Registry and Persistence Coordination Boundary
* ADR-051 Recording Artifact Processing Boundary

Die Entwicklung der Persistenzarchitektur:

ADR-043
Persistence Boundary

↓

ADR-044
Persistence Provider Interface

↓

ADR-048
Artifact Registry and Persistence Coordination

↓

ADR-051
Artifact Processing Boundary

↓

ADR-052
Filesystem Persistence Provider

---

# Ergebnis

Mit diesem ADR wird die erste konkrete lokale Persistenzimplementierung vorbereitet.

Die Architektur bleibt unabhängig von der konkreten Speichertechnologie.

Der nächste Implementierungsschritt besteht in der Einführung des `FilesystemPersistenceProvider` hinter der bestehenden Persistence Boundary.

---

# English Version ([German version above](#deutsch))

---

# Context

NC-PoRe already contains a defined technical boundary between Recorder logic and concrete storage through the Persistence Boundary.

The existing `InMemoryPersistenceProvider` is used to validate the architecture and support automated tests.

A production-capable application requires persistent local storage.

The next technical extension is therefore a concrete implementation of the existing Persistence interface.

This decision affects only the technical storage implementation.

The architecture boundary itself remains unchanged.

---

# Decision

NC-PoRe introduces a `FilesystemPersistenceProvider`.

The `FilesystemPersistenceProvider`:

* implements the existing `PersistenceProvider` interface
* does not replace the in-memory implementation
* remains an interchangeable implementation behind the Persistence Boundary
* is used as the first production-oriented local storage implementation

The Recorder continues to interact only with the defined Persistence interface.

The filesystem is an implementation decision and not part of the Recorder architecture.

---

# Responsibilities

The `FilesystemPersistenceProvider` is responsible for:

* storing RecordingArtifacts
* loading RecordingArtifacts
* listing available RecordingArtifacts
* removing RecordingArtifacts

The implementation encapsulates all required filesystem access.

---

# Not Included

This ADR intentionally does not decide on:

* concrete directory layout
* filename conventions
* file formats
* metadata structures outside the RecordingArtifact
* synchronization
* recovery mechanisms
* conflict handling
* encryption
* compression
* cloud storage
* database persistence

These topics will be addressed in separate decisions when concrete requirements exist.

---

# Rationale

The Persistence Boundary was intentionally introduced before choosing a concrete storage technology.

This preserves the following properties:

* Recorder Workflow does not know storage technology.
* Artifact Processing does not know storage technology.
* Application Flow does not take over persistence responsibilities.
* Tests can continue using the InMemoryPersistenceProvider.
* Additional storage implementations can be added later.

The filesystem is treated as an interchangeable technical implementation.

---

# Consequences

## Positive Consequences

* NC-PoRe receives its first real persistent storage implementation.
* The existing architecture remains unchanged.
* Existing tests can continue without filesystem dependency.
* The persistence implementation remains replaceable.
* Production-oriented tests become possible.

## Negative Consequences

* Two persistence implementations need to be maintained.
* Filesystem access requires dedicated tests.
* Local storage error handling must be considered later.

---

# Alternatives Considered

## Direct filesystem access inside the Recorder

Rejected.

Storage would leak into workflow or application layers and violate the existing architecture boundary.

---

## Storage directly inside Artifact Processing

Rejected.

Artifact Processing is responsible for technical processing of results, not storage details.

---

## Database as first persistence implementation

Not selected.

A database may become useful later but is not required for the first local persistence layer.

---

## Cloud Storage as first persistence implementation

Not selected.

Local recording and local availability remain central NC-PoRe principles.

---

# Relationship to Other ADRs

This ADR builds on the following decisions:

* ADR-043 Local Recording Persistence Boundary
* ADR-044 Persistence Provider Interface
* ADR-048 Artifact Registry and Persistence Coordination Boundary
* ADR-051 Recording Artifact Processing Boundary

The evolution of the persistence architecture:

ADR-043
Persistence Boundary

↓

ADR-044
Persistence Provider Interface

↓

ADR-048
Artifact Registry and Persistence Coordination

↓

ADR-051
Artifact Processing Boundary

↓

ADR-052
Filesystem Persistence Provider

---

# Result

This ADR prepares the first concrete local persistence implementation.

The architecture remains independent from the concrete storage technology.

The next implementation step is introducing the `FilesystemPersistenceProvider` behind the existing Persistence Boundary.
