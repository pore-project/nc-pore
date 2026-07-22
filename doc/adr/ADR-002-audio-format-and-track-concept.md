# ADR-002: Audio Format and Track Concept

## Status

Accepted

## Date

2026-07-22

---

# Context

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

# Decision

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

# Consequences

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

# Alternatives considered

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

# Notes

Die Trennung von Aufnahme und Produktion ist ein
Grundprinzip von NC-PoRe.

NC-PoRe erzeugt Rohmaterial.

Die kreative Bearbeitung erfolgt in spezialisierten
Produktionswerkzeugen.