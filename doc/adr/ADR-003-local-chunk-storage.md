# ADR-003: Local Chunk-Based Audio Storage

## Status

Accepted

## Date

2026-07-22

---

# Context

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

# Decision

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

# Chunk-Eigenschaften

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

# Recovery-Verhalten

Bei einer Unterbrechung:

1. Bereits gespeicherte Chunks bleiben erhalten.
2. Nicht abgeschlossene Chunks werden erkannt.
3. Die Session kann wiederhergestellt oder sauber beendet werden.

---

# Consequences

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

# Alternatives considered

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

# Notes

Chunk-basierte Speicherung unterstützt das zentrale
NC-PoRe-Prinzip:

> Die Aufnahmequalität darf nicht von der Netzwerkverbindung abhängen.

Die Netzwerkverbindung wird erst nach Abschluss der
Aufnahme relevant.