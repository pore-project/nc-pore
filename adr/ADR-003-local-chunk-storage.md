# Deutsch ([English version below](#english-version))

# ADR-003: Chunk-basierte lokale Audiospeicherung

## Status

Angenommen

## Datum

2026-07-22

---

# Kontext

Podcastaufnahmen können mehrere Stunden dauern und erzeugen
große Audiodateien.

Eine direkte Speicherung in eine einzelne finale Datei
erzeugt mehrere Risiken:

- Bei einem Absturz kann die gesamte Aufnahme verloren gehen.
- Fehler beim Schreiben können die Datei beschädigen.
- Lange Dateien sind schwieriger zu verwalten.
- Eine Wiederaufnahme nach Unterbrechungen ist schwierig.

NC-PoRe soll auch bei realen Alltagssituationen zuverlässig
funktionieren.

---

# Entscheidung

NC-PoRe speichert laufende Aufnahmen lokal in mehreren
aufeinanderfolgenden Chunks.

Während der Aufnahme wird nicht direkt eine finale
Masterdatei erzeugt.

Beispiel:
Session_2026_07_22/

audio/
chunk_0001.wav
chunk_0002.wav
chunk_0003.wav
chunk_0004.wav

Nach Abschluss der Aufnahme werden die Chunks zu einer
finalen Audiospur zusammengeführt.

Beispiel:
Host.wav
Gast.wav

---

# Eigenschaften eines Chunks

Ein Chunk:

- besitzt eine eindeutige Nummer
- enthält eine definierte Zeitspanne
- wird nach erfolgreichem Schreiben abgeschlossen
- wird niemals überschrieben

Beispiel:
chunk_0001.wav
chunk_0002.wav
chunk_0003.wav

---

# Standardgröße

Die genaue Chunk-Größe ist konfigurierbar.

Richtwert:
5 Minuten

Begründung:

- ausreichend kleine Wiederherstellungseinheiten
- überschaubare Dateigrößen
- geringer Verwaltungsaufwand

---

# Wiederherstellungsverhalten

Bei einer Unterbrechung:

1. Bereits gespeicherte Chunks bleiben erhalten.
2. Nicht abgeschlossene Chunks werden erkannt.
3. Die Session kann wiederhergestellt oder sauber beendet werden.

---

# Konsequenzen

## Positive Auswirkungen

- hohe Ausfallsicherheit
- geringe Gefahr von Datenverlust
- Wiederherstellung möglich
- geeignet für lange Aufnahmen
- bessere Fehleranalyse

---

## Negative Auswirkungen

- zusätzliche Verwaltungslogik erforderlich
- Zusammenführen der Chunks notwendig
- mehr Dateien im lokalen Speicher

---

# Betrachtete Alternativen

## Direkte Speicherung einer großen WAV-Datei

Verworfen.

Gründe:

- höheres Verlustrisiko
- schwierigere Fehlerbehandlung
- keine einfache Wiederherstellung

---

## Upload während der Aufnahme

Nicht als Standard vorgesehen.

Gründe:

- zusätzliche Netzlast während des Gesprächs
- Konkurrenz zur Audio-Kommunikation
- Abhängigkeit von Internetqualität

---

# Hinweise

Chunk-basierte Speicherung unterstützt das zentrale
NC-PoRe-Prinzip:

> Die Aufnahmequalität darf nicht von der Netzwerkverbindung abhängen.

Die Netzwerkverbindung wird erst nach Abschluss der
Aufnahme relevant.

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-003: Local Chunk-Based Audio Storage

## Status

Accepted

## Date

2026-07-22

---

# Context

Podcast recordings can last several hours and produce
large audio files.

Directly storing a recording in a single final file
creates several risks:

- A crash can result in the loss of the entire recording.
- Write errors can corrupt the file.
- Long files are more difficult to manage.
- Resuming after interruptions is difficult.

NC-PoRe is intended to operate reliably even in real-world
everyday situations.

---

# Decision

NC-PoRe stores ongoing recordings locally in multiple
consecutive chunks.

During recording, no final master file is created directly.

Example:
Session_2026_07_22/

audio/
chunk_0001.wav
chunk_0002.wav
chunk_0003.wav
chunk_0004.wav

After the recording is completed, the chunks are combined
into a final audio track.

Example:
Host.wav
Guest.wav

---

# Chunk Properties

A chunk:

- has a unique number
- contains a defined time span
- is finalized after successful writing
- is never overwritten

Example:
chunk_0001.wav
chunk_0002.wav
chunk_0003.wav

---

# Standard Size

The exact chunk size is configurable.

Reference value:
5 minutes

Rationale:

- sufficiently small recovery units
- manageable file sizes
- low administrative overhead

---

# Recovery Behavior

In the event of an interruption:

1. Already stored chunks remain available.
2. Incomplete chunks are detected.
3. The session can be recovered or cleanly terminated.

---

# Consequences

## Positive Impacts

- high resilience against failures
- low risk of data loss
- recovery is possible
- suitable for long recordings
- improved error analysis

---

## Negative Impacts

- additional management logic is required
- chunks must be merged
- more files are stored locally

---

# Alternatives Considered

## Direct Storage of a Large WAV File

Rejected.

Reasons:

- higher risk of data loss
- more difficult error handling
- no simple recovery mechanism

---

## Upload During Recording

Not intended as the standard approach.

Reasons:

- additional network load during the conversation
- competes with audio communication
- dependency on Internet quality

---

# Notes

Chunk-based storage supports the central
NC-PoRe principle:

> Recording quality must not depend on the network connection.

The network connection only becomes relevant after the
recording has been completed.
