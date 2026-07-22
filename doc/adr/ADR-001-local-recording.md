# ADR-001: Local Recording as Fundamental Architecture Principle

## Status

Accepted

## Date

2026-07-22

---

# Context

Podcast- und Gesprächsaufnahmen werden häufig über zentrale
Online-Dienste realisiert.

Bei dieser Vorgehensweise entstehen mehrere Probleme:

- Die Audioqualität hängt von der Netzwerkverbindung ab.
- Verbindungsabbrüche können zu Audioverlust führen.
- Upload- und Streaminglast konkurrieren um dieselbe Internetverbindung.
- Die Datenkontrolle liegt teilweise bei externen Dienstanbietern.

NC-PoRe verfolgt das Prinzip:

> Meine Daten gehören mir.

Daher soll die Aufnahme unabhängig von externen Servern
und unabhängig von der aktuellen Netzwerkqualität erfolgen.

---

# Decision

NC-PoRe zeichnet Audiodaten grundsätzlich lokal auf dem
Endgerät jedes Teilnehmers auf.

Während der laufenden Aufnahme werden keine Audiodaten
an den Server übertragen.

Nach Abschluss der Aufnahme werden die erzeugten Audiodateien
kontrolliert zum selbstgehosteten NC-PoRe-Server übertragen.

---

# Consequences

## Positive Auswirkungen

- Unabhängigkeit von Internetqualität während der Aufnahme.
- Keine zusätzliche Netzlast während des Gesprächs.
- Höhere Audioqualität möglich.
- Teilnehmer behalten Kontrolle über ihre Audiodaten.
- Aufnahme kann auch bei temporären Netzwerkproblemen erfolgen.

---

## Negative Auswirkungen

- Lokale Recorder-Software ist erforderlich.
- Mehrere Audiospuren müssen später synchronisiert werden.
- Speicherplatz auf Teilnehmergeräten wird benötigt.
- Upload erfolgt zeitversetzt nach der Aufnahme.

---

# Alternatives considered

## Serverseitige Stream-Aufnahme

Verworfen.

Gründe:

- Netzwerkabhängigkeit.
- Qualitätsverlust bei Verbindungsproblemen.
- zusätzliche Belastung der Verbindung.

---

## Cloud-basierte Aufnahmedienste

Verworfen.

Gründe:

- Abhängigkeit von externen Anbietern.
- eingeschränkte Datenhoheit.
- nicht vereinbar mit der Grundphilosophie von NC-PoRe.

---

# Notes

Diese Entscheidung ist eine zentrale Designentscheidung.

Alle zukünftigen Komponenten müssen dieses Prinzip respektieren.