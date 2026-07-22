# NC-PoRe Project Status

## Last Update

2026-07-22

---

# Project Phase

Current phase:

## Foundation and Architecture Definition

NC-PoRe befindet sich aktuell in der Konzeptions- und
Architekturphase.

Der technische Kern wird noch nicht implementiert.

Der Fokus liegt auf:

- Architekturentscheidungen
- Anforderungen
- Datenmodell
- Projektstruktur
- FOSS-Grundlagen

---

# Project Vision

NC-PoRe ist eine selbstgehostete Open-Source-Plattform
für professionelle Podcast-Aufnahmen und Produktion.

Zentrales Prinzip:

> Meine Daten gehören mir.

Audioaufnahmen werden lokal erzeugt und erst nach Abschluss
der Aufnahme zum eigenen Server übertragen.

---

# Completed

## Project Setup

Completed:

- GitHub Repository erstellt
- AGPL-3.0 Lizenz gewählt
- Dokumentationsstruktur eingerichtet

---

## Vision and Requirements

Completed:

- Projektvision dokumentiert
- funktionale Anforderungen definiert

---

## Architecture Decisions

Completed:

### ADR-001
Local Recording

Grundentscheidung für lokale Aufnahme.

### ADR-002
Audio Format and Track Concept

Getrennte hochwertige Monospuren als Produktionsbasis.

### ADR-003
Local Chunk Storage

Chunk-basierte lokale Speicherung.

### ADR-004
Upload After Recording

Upload erst nach Abschluss der Aufnahme.

### ADR-005
Consent and Recording Transparency

Transparente Aufnahme und dokumentierte Zustimmung.

### ADR-006
Role-Based Access Control

Rollenmodell für unterschiedliche Nutzergruppen.

### ADR-007
Open Formats and Interoperability

Offene Formate und freie Werkzeugwahl.

### ADR-008
Client Architecture

Modulare Recorder-Architektur mit professionellen
und vereinfachten Clients.

---

# Current Architecture Principles

NC-PoRe folgt diesen Grundsätzen:

- lokale Aufnahme
- keine Audioabhängigkeit vom Netzwerk
- offene Formate
- getrennte Audiospuren
- transparente Zustimmung
- rollenbasierte Rechte
- selbsthostbare Infrastruktur
- Erweiterbarkeit

---

# Next Steps

Geplante nächste Schritte:

## Architecture Refinement

- detaillierte Systemarchitektur
- Datenmodell
- Session-Modell
- Upload-Protokoll
- Synchronisationskonzept

---

## Technical Decisions

Geplante ADRs:

- ADR-009: Track Synchronisation
- ADR-010: Data Model
- ADR-011: Security Model
- ADR-012: Export Architecture

---

## Prototype Phase

Nach Abschluss der Architekturphase:

- technischer Recorder-Prototyp
- lokale WAV-Aufnahme
- Chunk-Verarbeitung
- einfache Sessionverwaltung

---

# Current Status Summary

NC-PoRe verfügt über:

- definierte Vision
- dokumentierte Anforderungen
- grundlegende Architektur
- zentrale Designentscheidungen

Die Implementierungsphase wurde bewusst noch nicht begonnen.
