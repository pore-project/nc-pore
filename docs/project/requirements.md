# NC-PoRe Requirements

## Ziel

NC-PoRe ist eine selbstgehostete Podcast-Produktionsplattform
mit lokaler Audioaufnahme und anschließender Verarbeitung
auf einem eigenen Server.

---

# Funktionale Anforderungen

## Aufnahme

NC-PoRe muss:

- Audio lokal auf dem Teilnehmergerät aufnehmen.
- Die Aufnahme unabhängig von der Netzwerkqualität ermöglichen.
- Hochwertige Audiodateien erzeugen.
- Mehrere Teilnehmer getrennt aufnehmen können.

---

## Speicherung

NC-PoRe muss:

- lokale Zwischenspeicherung ermöglichen.
- Aufnahmen in Chunks speichern.
- abgeschlossene Aufnahmen zum Server übertragen.
- Originalaufnahmen unverändert erhalten.

---

## Teilnehmer

NC-PoRe muss unterstützen:

- interne Benutzer
- externe Gäste
- Rollen und Berechtigungen

Geplante Rollen:

- Administrator
- Moderator
- Benutzer
- Editor
- Gast

---

## Datenschutz

NC-PoRe muss:

- transparent über laufende Aufnahmen informieren.
- Zustimmung der Teilnehmer dokumentieren.
- ohne externe Cloud-Dienste funktionieren.

---

## Export

NC-PoRe soll ermöglichen:

- Weiterverarbeitung in Audiosoftware.
- strukturierte Ablage der Audiospuren.
- spätere Integration von DAW-Projektexporten.

---

# Nicht-Ziele Version 0.1

Nicht Bestandteil der ersten Version:

- Videoaufnahme
- Live-Mixing
- Streaming
- automatische Veröffentlichung
- Cloud-KI-Dienste