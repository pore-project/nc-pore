# ADR-012: Export Architecture

## Status

Accepted

## Date

2026-07-22

---

# Context

NC-PoRe erzeugt hochwertige Mehrspuraufnahmen.

Die Aufnahme ist jedoch nicht das Ende des
Produktionsprozesses.

Professionelle Podcaster verwenden häufig spezialisierte
Werkzeuge für:

- Schnitt
- Mischung
- Klangbearbeitung
- Mastering
- Veröffentlichung

Beispiele:

- Audacity
- Ardour
- andere DAWs

NC-PoRe soll diese Werkzeuge unterstützen und keine
proprietäre Produktionsumgebung erzwingen.

---

# Decision

NC-PoRe trennt Aufnahme und Produktion.

Die Plattform erzeugt offene Produktionsdaten,
die außerhalb von NC-PoRe weiterverarbeitet werden können.

---

# Export Principles

Ein Export enthält:

- Audiodaten
- Metadaten
- Synchronisationsinformationen
- Sessioninformationen

Beispiel:

```
Episode_042_Export/

audio/

    host.wav
    guest.wav
    cohost.wav

metadata.json

session.json
```

---

# Supported Export Types

## Raw Multitrack Export

Basisexport.

Enthält:

- einzelne Mono-WAV-Spuren
- Synchronisationsdaten
- Metadaten

Eigenschaften:

- immer verfügbar
- unabhängig von proprietären Werkzeugen

---

## Audacity Export

NC-PoRe kann optional eine vorbereitete
Audacity-Projektstruktur erzeugen.

Beinhaltet:

- importierte Spuren
- richtige Positionierung
- Spurbenennung
- Metadaten

---

## Ardour Export

NC-PoRe kann optional eine Ardour-Session erzeugen.

Beinhaltet:

- Sessiondateien
- Spuren
- Verknüpfungen
- Grundkonfiguration

---

# Export Ownership

Exportierte Daten gehören vollständig dem Benutzer.

NC-PoRe verhindert keine Weiterverarbeitung
außerhalb der Plattform.

---

# Free Core vs Extended Features

Der freie Kern unterstützt:

- Rohdatenexport
- offene Audioformate
- vollständigen Zugriff auf eigene Daten

Erweiterte Funktionen können zusätzliche
Komfortfunktionen anbieten:

- automatische DAW-Projekte
- Workflow-Automatisierung
- Archivverwaltung
- Produktionsvorlagen

---

# Consequences

## Positive Auswirkungen

- keine Abhängigkeit von NC-PoRe
- Unterstützung der FOSS-Werkzeuglandschaft
- professionelle Workflows möglich
- langfristige Datenverfügbarkeit

---

## Negative Auswirkungen

- zusätzliche Exportlogik erforderlich
- Tests verschiedener Produktionswerkzeuge notwendig
- Pflege mehrerer Formate

---

# Alternatives considered

## Eigenes geschlossenes Projektformat

Verworfen.

Grund:

Widerspricht der Datenhoheit und FOSS-Philosophie.

---

## Nur fertige Audiodatei exportieren

Verworfen.

Grund:

Nicht ausreichend für professionelle Produktion.

---

# Notes

NC-PoRe produziert Rohmaterial und organisiert
Arbeitsabläufe.

Die kreative Entscheidung über Schnitt und Mischung
bleibt beim Menschen.
```