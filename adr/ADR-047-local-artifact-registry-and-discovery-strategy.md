# ADR-047: Local Artifact Registry and Discovery Strategy

* Status: Accepted
* Date: 2026-08-02
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

---

# Kontext

NC-PoRe verwaltet Recording Artifacts als eigenständige technische Objekte.

Mit ADR-041 wurde die Trennung zwischen Recording Artifacts und Storage definiert.

Mit ADR-042 wurde ein eigener Lifecycle für Recording Artifacts eingeführt.

Mit ADR-046 wurde die Strategie für Recovery und Konsistenzprüfung lokaler Artifacts definiert.

Damit entsteht die nächste notwendige Frage:

> Wie erkennt und verwaltet NC-PoRe die lokal vorhandenen Artifacts?

Eine reine Storage-Abfrage reicht dafür nicht aus.

Storage stellt Daten bereit, besitzt aber keine eigene Übersicht über technische Zustände, offene Recovery-Prozesse oder lokale Artifact-Referenzen.

---

# Entscheidung

NC-PoRe führt eine eigene Local Artifact Registry ein.

Die Registry verwaltet die technische Übersicht lokaler Recording Artifacts.

Die Registry ist:

* getrennt vom Storage
* getrennt vom Recording Artifact selbst
* unabhängig von konkreten Speichertechnologien
* über eine technische Schnittstelle abstrahiert

---

# Verantwortlichkeiten

Die Local Artifact Registry ist verantwortlich für:

* Erfassen vorhandener Artifacts
* Verwaltung technischer Metadaten
* Auffinden von Artifacts für Recovery-Prozesse
* Unterstützung von Konsistenzprüfungen
* Verwaltung technischer Artifact-Referenzen

Die Registry ist nicht verantwortlich für:

* Speicherung der eigentlichen Mediendaten
* Audioverarbeitung
* fachliche Recording-Logik
* Netzwerk-Synchronisation

---

# Architekturgrenzen

Die technische Struktur wird:

```text
Recording Workflow

↓

Recording Artifact

↓

Local Artifact Registry

↓

Persistence Provider

↓

Storage
```

Die Registry kennt Existenz und technischen Zustand von Artifacts.

Der Persistence Provider verwaltet deren Speicherung.

---

# Konsequenzen

## Positive Konsequenzen

* Recovery-Prozesse können gezielt nach Artifacts suchen.
* Inkonsistenzen können erkannt werden.
* Storage-Technologien bleiben austauschbar.
* Artifact-Verwaltung bleibt unabhängig von Medienformaten.

---

## Negative Konsequenzen

* Eine zusätzliche technische Komponente entsteht.
* Registry-Zustände müssen selbst konsistent gehalten werden.
* Die Beziehung zwischen Registry und Persistence muss definiert werden.

---

# Nicht Bestandteil dieser Entscheidung

Diese ADR entscheidet nicht über:

* konkrete Dateiformate
* Datenbanktechnologien
* Synchronisationsprotokolle
* Cloud- oder Server-Speicher
* Audioaufnahmeimplementierungen

Diese Entscheidungen werden separat getroffen.

---

# Beziehung zu bestehenden ADRs

Diese Entscheidung erweitert:

* ADR-041 Local Recording Artifact and Storage Boundary
* ADR-042 Recording Artifact Model and Lifecycle Boundary
* ADR-043 Local Recording Persistence Boundary
* ADR-044 Persistence Provider Interface
* ADR-046 Local Artifact Recovery and Consistency Strategy

---

# Zusammenfassung

NC-PoRe verwendet eine eigene Local Artifact Registry als technische Verwaltungsschicht zwischen Recording Artifacts, Persistence und Storage.

Die Registry ermöglicht Auffindbarkeit, Recovery und Konsistenzprüfung, ohne Storage oder Domainlogik zu vermischen.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# Context

NC-PoRe manages Recording Artifacts as independent technical objects.

ADR-041 defined the separation between Recording Artifacts and Storage.

ADR-042 introduced a dedicated lifecycle for Recording Artifacts.

ADR-046 defined the recovery and consistency strategy for local artifacts.

This creates the next required question:

> How does NC-PoRe identify and manage locally existing artifacts?

A pure storage query is insufficient.

Storage provides data, but it does not maintain technical lifecycle awareness, recovery state or local artifact references.

---

# Decision

NC-PoRe introduces a dedicated Local Artifact Registry.

The registry maintains the technical overview of local Recording Artifacts.

The registry is:

* separated from storage
* separated from the Recording Artifact itself
* independent from concrete storage technologies
* abstracted through a technical interface

---

# Responsibilities

The Local Artifact Registry is responsible for:

* tracking existing artifacts
* maintaining technical metadata
* locating artifacts for recovery processes
* supporting consistency checks
* managing technical artifact references

The registry is not responsible for:

* storing media data itself
* audio processing
* domain recording logic
* network synchronization

---

# Architecture Boundaries

The technical structure becomes:

```text
Recording Workflow

↓

Recording Artifact

↓

Local Artifact Registry

↓

Persistence Provider

↓

Storage
```

The registry knows the existence and technical state of artifacts.

The Persistence Provider manages their storage.

---

# Consequences

## Positive Consequences

* Recovery processes can locate artifacts intentionally.
* Inconsistencies can be detected.
* Storage technologies remain replaceable.
* Artifact management remains independent from media formats.

---

## Negative Consequences

* An additional technical component is introduced.
* Registry state must remain consistent.
* The relationship between registry and persistence must be defined.

---

# Not Part of This Decision

This ADR does not decide:

* concrete file formats
* database technologies
* synchronization protocols
* cloud or server storage
* audio capture implementations

These decisions will be handled separately.

---

# Relationship to Existing ADRs

This decision extends:

* ADR-041 Local Recording Artifact and Storage Boundary
* ADR-042 Recording Artifact Model and Lifecycle Boundary
* ADR-043 Local Recording Persistence Boundary
* ADR-044 Persistence Provider Interface
* ADR-046 Local Artifact Recovery and Consistency Strategy

---

# Summary

NC-PoRe uses a dedicated Local Artifact Registry as a technical management layer between Recording Artifacts, Persistence and Storage.

The registry enables discovery, recovery and consistency validation without mixing storage concerns with domain logic.
