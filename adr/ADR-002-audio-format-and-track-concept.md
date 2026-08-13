# Deutsch ([English version below](#english-version))

# ADR-002: Audioformat und Spurkonzept

## Status

Angenommen

## Datum

2026-07-22

---

# Kontext

NC-PoRe ist nicht nur ein Aufnahmewerkzeug, sondern eine
Podcast-Produktionsplattform.

Für professionelle Nachbearbeitung ist es notwendig,
die einzelnen Teilnehmer getrennt bearbeiten zu können.

Eine während der Aufnahme erzeugte Mischung würde diese
Möglichkeiten einschränken:

- individuelle Lautstärkeanpassung wäre schwieriger
- Störgeräusche könnten nicht gezielt entfernt werden
- unterschiedliche Bearbeitung einzelner Stimmen wäre nicht möglich
- spätere Produktionsschritte wären eingeschränkt

---

# Entscheidung

NC-PoRe speichert Aufnahmen grundsätzlich als getrennte
Monospuren pro Teilnehmer.

Jeder Teilnehmer erzeugt eine eigene Audiodatei.

Beispiel:
Episode_042/

audio/
host.wav
gast.wav
cohost.wav
metadata.json

Die Aufnahme wird nicht zu einer gemeinsamen Audiodatei gemischt.

---

# Audioformat

Das bevorzugte Masterformat ist:
WAV
PCM
48 kHz
24 Bit
Mono

Begründung:

- verlustfreie Speicherung
- professionelle Weiterverarbeitung
- Unterstützung durch praktisch alle DAWs
- ausreichende Qualität für Sprache und Musikanteile

---

# Alternative Aufnahmeformate

Komprimierte Formate wie Opus können optional unterstützt werden.

Beispiel:
Opus
48 kHz
128 kbit/s oder höher
Mono

Sie sind geeignet für:

- geringe Speicheranforderungen
- mobile Szenarien
- schnelle Übertragung

Sie ersetzen jedoch nicht das hochwertige Masterformat.

---

# Konsequenzen

## Positive Auswirkungen

- maximale Flexibilität in der Nachbearbeitung
- einfache Bearbeitung in DAWs
- Sprecher können unabhängig behandelt werden
- bessere Archivqualität
- keine Qualitätsverluste durch Vorabmischung

---

## Negative Auswirkungen

- größerer Speicherbedarf
- mehr Dateien pro Episode
- Synchronisation mehrerer Spuren notwendig

---

# Betrachtete Alternativen

## Gemeinsamer Stereo-Mix während der Aufnahme

Verworfen.

Gründe:

- keine individuelle Bearbeitung möglich
- Fehler sind dauerhaft eingebrannt
- entspricht nicht professionellen Produktionsabläufen

---

## Mehrkanal-WAV mit allen Teilnehmern

Nicht als primäres Format gewählt.

Begründung:

Mehrkanal-WAV kann technisch mehrere Spuren enthalten,
aber einzelne Monodateien bieten:

- bessere Kompatibilität
- einfachere Archivierung
- bessere Zusammenarbeit zwischen verschiedenen DAWs

---

# Hinweise

Die Trennung von Aufnahme und Produktion ist ein
Grundprinzip von NC-PoRe.

NC-PoRe erzeugt Rohmaterial.

Die kreative Bearbeitung erfolgt in spezialisierten
Produktionswerkzeugen.

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-002: Audio Format and Track Concept

## Status

Accepted

## Date

2026-07-22

---

# Context

NC-PoRe is not only a recording tool but a podcast production platform.

For professional post-production, it is necessary to be able to process each participant separately.

A mix created during recording would limit these possibilities:

- individual volume adjustment would be more difficult
- unwanted noise could not be removed selectively
- different processing of individual voices would not be possible
- later production steps would be constrained

---

# Decision

NC-PoRe stores recordings as separate mono tracks per participant by default.

Each participant produces a separate audio file.

Example:
Episode_042/

audio/
host.wav
gast.wav
cohost.wav
metadata.json

The recording is not mixed into a single audio file.

---

# Audio Format

The preferred master format is:
WAV
PCM
48 kHz
24 bit
Mono

Rationale:

- lossless storage
- professional post-production
- support by practically all DAWs
- sufficient quality for speech and musical content

---

# Alternative Recording Formats

Compressed formats such as Opus may optionally be supported.

Example:
Opus
48 kHz
128 kbit/s or higher
Mono

They are suitable for:

- low storage requirements
- mobile scenarios
- fast transfer

However, they do not replace the high-quality master format.

---

# Consequences

## Positive Effects

- maximum flexibility in post-production
- easy editing in DAWs
- speakers can be processed independently
- better archival quality
- no quality loss caused by pre-mixing

---

## Negative Effects

- higher storage requirements
- more files per episode
- synchronization of multiple tracks is required

---

# Alternatives Considered

## Common Stereo Mix During Recording

Rejected.

Reasons:

- no individual processing possible
- errors are permanently baked in
- does not correspond to professional production workflows

---

## Multichannel WAV with All Participants

Not selected as the primary format.

Rationale:

Multichannel WAV can technically contain multiple tracks,
but individual mono files provide:

- better compatibility
- simpler archiving
- better collaboration between different DAWs

---

# Notes

The separation of recording and production is a fundamental principle of NC-PoRe.

NC-PoRe produces raw material.

Creative editing takes place in specialized production tools.
