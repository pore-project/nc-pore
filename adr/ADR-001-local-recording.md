# ADR-001: Local Recording as Fundamental Architecture Principle

## Status

Accepted

## Date

2026-07-22

---

# Deutsch ([English version below](#english-version))

# Kontext

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

# Entscheidung

NC-PoRe zeichnet Audiodaten grundsätzlich lokal auf dem
Endgerät jedes Teilnehmers auf.

Während der laufenden Aufnahme werden keine Audiodaten
an den Server übertragen.

Nach Abschluss der Aufnahme werden die erzeugten Audiodateien
kontrolliert zum selbstgehosteten NC-PoRe-Server übertragen.

---

# Konsequenzen

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

# Betrachtete Alternativen

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

# Hinweise

Diese Entscheidung ist eine zentrale Designentscheidung.

Alle zukünftigen Komponenten müssen dieses Prinzip respektieren.

---

# English Version ([Deutsche Version oben](#deutsch))

# Context

Podcast and conversation recordings are often implemented using centralized
online services.

This approach creates several problems:

- Audio quality depends on the network connection.
- Connection failures can result in audio loss.
- Upload and streaming traffic compete for the same Internet connection.
- Control over the data is partly held by external service providers.

NC-PoRe follows the principle:

> My data belongs to me.

Therefore, recording shall operate independently of external servers
and independently of the current network quality.

---

# Decision

NC-PoRe records audio data locally on the end device of each participant
as a fundamental principle.

No audio data is transmitted to the server while recording is in progress.

After recording has been completed, the generated audio files are
transferred in a controlled manner to the self-hosted NC-PoRe server.

---

# Consequences

## Positive effects

- Independence from Internet quality during recording.
- No additional network load during the conversation.
- Higher audio quality is possible.
- Participants retain control over their audio data.
- Recording can continue even during temporary network problems.

---

## Negative effects

- Local recorder software is required.
- Multiple audio tracks must be synchronized later.
- Storage space is required on participant devices.
- Upload takes place after the recording with a time delay.

---

# Alternatives considered

## Server-side stream recording

Rejected.

Reasons:

- Dependence on the network.
- Quality loss in case of connection problems.
- Additional load on the connection.

---

## Cloud-based recording services

Rejected.

Reasons:

- Dependence on external providers.
- Limited data sovereignty.
- Not compatible with the fundamental philosophy of NC-PoRe.

---

# Notes

This is a central design decision.

All future components must respect this principle.
