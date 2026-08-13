# Deutsch ([English version below](#english-version))

# ADR-012: Export Architecture

## Status

Accepted

## Date

2026-07-22

---

# Kontext

NC-PoRe erzeugt hochwertige Mehrspuraufnahmen.

Die Aufnahme ist jedoch nicht das Ende des Produktionsprozesses.

Professionelle Podcaster verwenden häufig spezialisierte Werkzeuge für:

- Schnitt
- Mischung
- Klangbearbeitung
- Mastering
- Veröffentlichung

Beispiele:

- Audacity
- Ardour
- andere DAWs

NC-PoRe soll diese Werkzeuge unterstützen und keine proprietäre Produktionsumgebung erzwingen.

---

# Entscheidung

NC-PoRe trennt Aufnahme und Produktion.

Die Plattform erzeugt offene Produktionsdaten, die außerhalb von NC-PoRe weiterverarbeitet werden können.

---

# Exportprinzipien

Ein Export enthält:

- Audiodaten
- Metadaten
- Synchronisationsinformationen
- Sessioninformationen

Beispiel:

```text
Episode_042_Export/

audio/

    host.wav
    guest.wav
    cohost.wav

metadata.json

session.json
```

---

# Unterstützte Exporttypen

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

NC-PoRe kann optional eine vorbereitete Audacity-Projektstruktur erzeugen.

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

# Exportverantwortung

Exportierte Daten gehören vollständig dem Benutzer.

NC-PoRe verhindert keine Weiterverarbeitung außerhalb der Plattform.

---

# Freier Kern vs. erweiterte Funktionen

Der freie Kern unterstützt:

- Rohdatenexport
- offene Audioformate
- vollständigen Zugriff auf eigene Daten

Erweiterte Funktionen können zusätzliche Komfortfunktionen anbieten:

- automatische DAW-Projekte
- Workflow-Automatisierung
- Archivverwaltung
- Produktionsvorlagen

---

# Konsequenzen

## Positive Auswirkungen

- keine Abhängigkeit von NC-PoRe
- Unterstützung der FOSS-Werkzeuglandschaft
- professionelle Workflows möglich
- langfristige Datenverfügbarkeit

## Negative Auswirkungen

- zusätzliche Exportlogik erforderlich
- Tests verschiedener Produktionswerkzeuge notwendig
- Pflege mehrerer Formate

---

# Betrachtete Alternativen

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

# Hinweise

NC-PoRe produziert Rohmaterial und organisiert Arbeitsabläufe.

Die kreative Entscheidung über Schnitt und Mischung bleibt beim Menschen.

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-012: Export Architecture

## Status

Accepted

## Date

2026-07-22

---

# Context

NC-PoRe produces high-quality multitrack recordings.

Recording, however, is not the end of the production process.

Professional podcasters often use specialized tools for:

- editing
- mixing
- audio processing
- mastering
- publishing

Examples:

- Audacity
- Ardour
- other DAWs

NC-PoRe should support these tools and must not impose a proprietary production environment.

---

# Decision

NC-PoRe separates recording from production.

The platform produces open production data that can be processed outside NC-PoRe.

---

# Export Principles

An export contains:

- audio data
- metadata
- synchronization information
- session information

Example:

```text
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

Basic export.

Contains:

- individual mono WAV tracks
- synchronization data
- metadata

Properties:

- always available
- independent of proprietary tools

---

## Audacity Export

NC-PoRe may optionally generate a prepared Audacity project structure.

Includes:

- imported tracks
- correct positioning
- track naming
- metadata

---

## Ardour Export

NC-PoRe may optionally generate an Ardour session.

Includes:

- session files
- tracks
- links
- basic configuration

---

# Export Ownership

Exported data belongs entirely to the user.

NC-PoRe does not prevent further processing outside the platform.

---

# Free Core vs. Extended Features

The free core supports:

- raw data export
- open audio formats
- full access to the user's own data

Extended features may provide additional convenience functions:

- automatic DAW projects
- workflow automation
- archive management
- production templates

---

# Consequences

## Positive Effects

- no dependency on NC-PoRe
- support for the FOSS tool ecosystem
- professional workflows possible
- long-term data availability

## Negative Effects

- additional export logic required
- testing of different production tools necessary
- maintenance of multiple formats

---

# Alternatives Considered

## Proprietary Closed Project Format

Rejected.

Reason:

Contradicts data ownership and the FOSS philosophy.

---

## Export Only the Final Audio File

Rejected.

Reason:

Not sufficient for professional production.

---

# Notes

NC-PoRe produces raw material and organizes workflows.

The creative decisions about editing and mixing remain with the human.
