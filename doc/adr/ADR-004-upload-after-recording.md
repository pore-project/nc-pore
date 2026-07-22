# ADR-004: Upload Only After Recording Completion

## Status

Accepted

## Date

2026-07-22

---

# Context

Viele moderne Remote-Aufnahmesysteme übertragen Audiodaten
bereits während der laufenden Aufnahme an einen Server.

Dieses Verfahren erzeugt Abhängigkeiten:

- Die Internetverbindung beeinflusst die Aufnahmequalität.
- Upload und Gespräch teilen sich dieselbe Netzwerkverbindung.
- Schwache oder instabile Verbindungen können zusätzliche
  Probleme verursachen.
- Teilnehmer mit begrenzter Bandbreite werden benachteiligt.

NC-PoRe verfolgt den Grundsatz:

> Die Aufnahmequalität darf nicht von der Netzwerkqualität abhängen.

---

# Decision

NC-PoRe überträgt Audiodaten grundsätzlich erst nach
Abschluss der Aufnahme.

Während einer laufenden Session:

- erfolgt keine Audioübertragung zum Server.
- wird ausschließlich lokal gespeichert.
- wird keine zusätzliche Netzwerkbandbreite benötigt.

Nach Beendigung der Aufnahme:

1. lokale Dateien werden finalisiert.
2. Metadaten werden erzeugt.
3. Upload zum NC-PoRe-Server wird gestartet.
4. Server bestätigt die erfolgreiche Übertragung.

---

# Upload-Verhalten

Der Upload muss:

- wiederaufnehmbar sein.
- Fehler erkennen können.
- unvollständige Dateien vermeiden.
- Integrität prüfen können.

Beispiel:
Lokale Aufnahme

↓

Finalisierung

↓

Upload

↓

Prüfung

↓

Archivierung

---

# Consequences

## Positive Auswirkungen

- maximale Audioqualität unabhängig vom Internet
- keine zusätzliche Netzlast während des Gesprächs
- geeignet für schlechte Internetverbindungen
- bessere Planbarkeit der Produktion
- klare Trennung zwischen Aufnahme und Speicherung

---

## Negative Auswirkungen

- Dateien müssen lokal zwischengespeichert werden.
- Upload erfolgt zeitversetzt.
- Teilnehmer benötigen ausreichend Speicherplatz.
- Server erhält Audiodaten erst nach Ende der Session.

---

# Alternatives considered

## Live-Upload während der Aufnahme

Verworfen.

Gründe:

- zusätzliche Belastung der Netzwerkverbindung
- mögliche Beeinträchtigung des Gesprächs
- Abhängigkeit von externen Faktoren

---

## Serverseitige Aufnahme eines Audiostreams

Verworfen.

Gründe:

- Netzwerkqualität beeinflusst Ergebnis
- Ausfälle können nicht nachträglich korrigiert werden
- entspricht nicht dem Prinzip der Datenhoheit

---

# Notes

NC-PoRe behandelt Netzwerkverbindungen als Transportweg
nach Abschluss einer Aufnahme, nicht als Bestandteil des
Aufnahmeprozesses.

Die Aufnahme selbst bleibt ein lokaler Vorgang.