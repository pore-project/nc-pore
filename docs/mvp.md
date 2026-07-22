# NC-PoRe MVP Definition

## Version

0.1

## Date

2026-07-22

---

# Goal

Der erste NC-PoRe-Prototyp soll beweisen, dass eine
datenschutzfreundliche lokale Podcastaufnahme mit
anschließender Serverübertragung möglich ist.

---

# MVP Principle

Der MVP konzentriert sich auf den Kernnutzen:

> Hochwertige lokale Aufnahme mehrerer Teilnehmer ohne
> Abhängigkeit von der Netzwerkqualität.

---

# Included Features

## Session Creation

Eine Aufnahme-Session kann erstellt werden.

Eine Session enthält:

- Titel
- Teilnehmer
- Zeitpunkt
- Status

---

## Local Recording

Der Recorder kann:

- ein Mikrofon auswählen
- Audio lokal aufnehmen
- WAV-Dateien erzeugen
- Aufnahme stoppen und speichern

---

## Multi Participant Recording

Mehrere Teilnehmer können getrennte Spuren erzeugen.

Beispiel:

```
session/

host.wav
guest.wav
metadata.json
```

---

## Local Chunk Storage

Aufnahmen werden während der Session
in lokale Chunks geschrieben.

Ziel:

- Schutz vor Datenverlust
- Wiederherstellbarkeit

---

## Upload

Nach Abschluss der Aufnahme:

- Dateien werden übertragen
- Integrität wird geprüft
- Session wird aktualisiert

---

## Basic Server Integration

Der Server kann:

- Sessions verwalten
- Benutzer zuordnen
- Dateien speichern
- Metadaten anzeigen

---

# Explicitly Not Included

Folgende Funktionen gehören nicht zum MVP:

## Video

Nicht Bestandteil der ersten Version.

---

## Live Streaming

Nicht Bestandteil der ersten Version.

---

## Automatic Publishing

Keine direkte Veröffentlichung.

---

## AI Features

Keine automatische:

- Transkription
- Zusammenfassung
- Sprecheranalyse

---

## Professional Export

Keine automatischen:

- Audacity-Projekte
- Ardour-Sessions

Diese Funktionen gehören zu späteren Erweiterungen.

---

# Success Criteria

Der MVP gilt als erfolgreich, wenn:

1. Zwei Teilnehmer können aufnehmen.

2. Die Audiospuren bleiben getrennt.

3. Die Qualität ist ausreichend für professionelle
   Sprachaufnahme.

4. Ein Upload zum eigenen Server funktioniert.

5. Die Daten können anschließend weiterverarbeitet werden.

---

# Next Development Phase

Nach erfolgreichem MVP:

- bessere Benutzeroberfläche
- Gastworkflow
- Exportfunktionen
- Produktionsverwaltung
- professionelle Features
```