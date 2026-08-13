# Deutsch ([English version below](#english-version))

# ADR-004: Upload Only After Recording Completion

## Status

Accepted

## Date

2026-07-22

---

# Kontext

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

# Entscheidung

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

# Konsequenzen

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

# Betrachtete Alternativen

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

# Hinweise

NC-PoRe behandelt Netzwerkverbindungen als Transportweg
nach Abschluss einer Aufnahme, nicht als Bestandteil des
Aufnahmeprozesses.

Die Aufnahme selbst bleibt ein lokaler Vorgang.

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-004: Upload Only After Recording Completion

## Status

Accepted

## Date

2026-07-22

---

# Context

Many modern remote recording systems transfer audio data
to a server while the recording is still in progress.

This approach creates dependencies:

- The Internet connection affects recording quality.
- Upload and conversation share the same network connection.
- Weak or unstable connections can cause additional problems.
- Participants with limited bandwidth are disadvantaged.

NC-PoRe follows the principle:

> Recording quality must not depend on network quality.

---

# Decision

NC-PoRe transfers audio data only after the recording has been completed.

During an active session:

- no audio is transferred to the server.
- audio is stored locally only.
- no additional network bandwidth is required.

After the recording has ended:

1. local files are finalized.
2. metadata is generated.
3. upload to the NC-PoRe server is started.
4. the server confirms successful transfer.

---

# Upload Behavior

The upload must:

- be resumable.
- be able to detect errors.
- avoid incomplete files.
- be able to verify integrity.

Example:
Local recording

↓

Finalization

↓

Upload

↓

Verification

↓

Archiving

---

# Consequences

## Positive Effects

- maximum audio quality independent of the Internet
- no additional network load during the conversation
- suitable for poor Internet connections
- better production planning
- clear separation between recording and storage

---

## Negative Effects

- files must be temporarily stored locally.
- upload takes place asynchronously after recording.
- participants need sufficient storage space.
- the server receives audio data only after the session has ended.

---

# Alternatives Considered

## Live Upload During Recording

Rejected.

Reasons:

- additional load on the network connection
- possible impact on the conversation
- dependency on external factors

---

## Server-Side Recording of an Audio Stream

Rejected.

Reasons:

- network quality affects the result
- failures cannot be corrected retrospectively
- does not comply with the principle of data sovereignty

---

# Notes

NC-PoRe treats network connections as a transport path
after a recording has been completed, not as part of the
recording process.

The recording itself remains a local process.
