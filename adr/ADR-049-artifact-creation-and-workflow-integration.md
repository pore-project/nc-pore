# ADR-049: Artifact Creation and Workflow Integration

* Status: Accepted
* Date: 2026-08-02
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

---

# Kontext

NC-PoRe trennt verschiedene technische Verantwortlichkeiten:

* CaptureProvider stellt Audioaufnahme-Funktionalität bereit.
* RecorderWorkflow koordiniert den Aufnahmeablauf.
* RecordingArtifact repräsentiert das technische Ergebnis einer Aufnahme.
* LocalArtifactRegistry verwaltet bekannte lokale Artifact-Referenzen.
* PersistenceProvider definiert die Persistenzgrenze.

Mit ADR-047 wurde eine eigene Local Artifact Registry eingeführt.

Mit ADR-048 wurde die Koordination zwischen Registry und Persistence definiert.

Damit entsteht die nächste Architekturfrage:

> Wo wird die Erzeugung eines Recording Artifacts aus einer abgeschlossenen Aufnahme koordiniert?

Die Audio-Capture-Schicht ist dafür nicht geeignet.

Sie kennt:

* technische Aufnahmeoperationen
* Start und Stop der Aufnahme

Sie kennt nicht:

* Recording-Lifecycle
* Artifact-Lifecycle
* Registry-Verwaltung
* Persistence

---

# Entscheidung

Die Erzeugung von Recording Artifacts wird durch die Workflow-Schicht koordiniert.

Der RecorderWorkflow erkennt den Abschluss eines Aufnahmeablaufs und übergibt die weitere Verwaltung an eine dedizierte Artifact-Koordinationsgrenze.

Der CaptureProvider erzeugt keine Artifacts.

---

# Verantwortlichkeiten

## RecorderWorkflow

Verantwortlich für:

* Koordination des Recording-Ablaufs
* Verbindung zwischen Session und technischen Komponenten
* Auslösen der Artifact-Erzeugung nach Abschluss einer Aufnahme

Nicht verantwortlich für:

* konkrete Speicherimplementierungen
* Audio-Backend-Details
* Registry-Verwaltung

---

## Artifact Coordination

Verantwortlich für:

* Erzeugung technischer Artifact-Referenzen
* Aktualisierung der Local Artifact Registry
* Koordination mit Persistence

Nicht verantwortlich für:

* Audioaufnahme
* Produktionslogik
* Storage-Implementierungen

---

## CaptureProvider

Verantwortlich für:

* Starten der Aufnahme
* Stoppen der Aufnahme

Nicht verantwortlich für:

* Artifact-Erzeugung
* Persistence
* Recovery-Prozesse

---

# Architekturgrenze

Die technische Struktur wird:

```text
RecorderWorkflow

↓

Artifact Coordination

↓

Local Artifact Registry

↓

Persistence Provider

↓

Storage
```

Die Workflow-Schicht koordiniert den Ablauf.

Die Artifact-Koordinationsschicht verwaltet die Übergabe zwischen Recording-Ergebnis, Registry und Persistence.

---

# Konsequenzen

## Positive Konsequenzen

* Audioaufnahme bleibt unabhängig von Artifact-Verwaltung.
* Artifact-Lifecycle bleibt außerhalb der Capture-Schicht.
* Registry und Persistence bleiben austauschbar.
* Verantwortlichkeiten bleiben nachvollziehbar.

---

## Negative Konsequenzen

* Eine zusätzliche Koordinationsgrenze entsteht.
* Mehrere Komponenten müssen zusammenarbeiten.
* Die Übergabe nach Recording-Ende muss definiert werden.

---

# Nicht Bestandteil dieser Entscheidung

Diese ADR entscheidet nicht über:

* konkrete Audio-Dateiformate
* Chunking-Strategien
* Synchronisation
* Exportformate
* Speichertechnologien

Diese Entscheidungen werden separat getroffen.

---

# Beziehung zu bestehenden ADRs

Diese Entscheidung erweitert:

* ADR-039 Recording Architecture and Capture Boundary
* ADR-040 Recorder Workflow and Capture Lifecycle Coordination
* ADR-042 Recording Artifact Model and Lifecycle Boundary
* ADR-044 Persistence Provider Interface
* ADR-047 Local Artifact Registry and Discovery Strategy
* ADR-048 Artifact Registry and Persistence Coordination

---

# Zusammenfassung

NC-PoRe erzeugt Recording Artifacts nicht innerhalb der Audioaufnahme.

Die Workflow-Schicht koordiniert die Erstellung und übergibt die technische Verwaltung an eine dedizierte Artifact-Koordinationsgrenze.

Dadurch bleiben Aufnahme, Artifact-Verwaltung und Speicherung sauber getrennt.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# Context

NC-PoRe separates several technical responsibilities:

* CaptureProvider provides audio capture capabilities.
* RecorderWorkflow coordinates recording execution.
* RecordingArtifact represents the technical result of a recording.
* LocalArtifactRegistry manages known local artifact references.
* PersistenceProvider defines the persistence boundary.

ADR-047 introduced a dedicated Local Artifact Registry.

ADR-048 defined coordination between registry and persistence.

The next architectural question is:

> Where is the creation of a Recording Artifact from a completed recording coordinated?

The audio capture layer is not suitable for this responsibility.

It knows:

* technical capture operations
* start and stop of capture

It does not know:

* recording lifecycle
* artifact lifecycle
* registry management
* persistence

---

# Decision

Recording Artifact creation is coordinated by the workflow layer.

The RecorderWorkflow detects completion of a recording process and delegates artifact management to a dedicated artifact coordination boundary.

The CaptureProvider does not create artifacts.

---

# Summary

NC-PoRe does not create Recording Artifacts inside the audio capture layer.

The workflow layer coordinates artifact creation and delegates technical management to the artifact coordination boundary.

This keeps recording, artifact management and persistence clearly separated.
