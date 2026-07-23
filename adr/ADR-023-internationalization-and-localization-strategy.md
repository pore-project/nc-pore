# ADR-023: Internationalization and Localization Strategy

* Status: Accepted
* Date: 2026-07-23
* Decision Type: Architecture

---

# Deutsch (English version below)

## Kontext

NC-PoRe wird in Deutschland entwickelt und ist Teil einer internationalen Welt.

Das Projekt soll von Anfang an so gestaltet werden, dass Menschen unabhängig von Sprache und Herkunft die Software nutzen und erweitern können.

Mehrsprachigkeit darf deshalb keine nachträglich hinzugefügte Funktion sein, sondern muss Bestandteil der Architektur werden.

Eine spätere Übersetzung einer ursprünglich nur einsprachig entwickelten Software verursacht häufig:

* hohe technische Kosten
* uneinheitliche Benutzeroberflächen
* schwer wartbare Texte
* Einschränkungen bei zukünftigen Erweiterungen

---

## Entscheidung

NC-PoRe wird von Anfang an internationalisierungsfähig entwickelt.

Die Software trennt technische Logik und sprachabhängige Inhalte.

Sichtbare Texte werden nicht direkt im Quellcode gespeichert, sondern über ein Localization-System bereitgestellt.

Beispiel:

```text
Code

    |
    |
    v

Translation Key

    |
    |
    v

Language Resource

├── Deutsch
├── English
└── weitere Sprachen
```

---

## Grundprinzipien

### 1. Keine fest eingebauten Benutzertexte

Benutzer sichtbare Texte dürfen nicht direkt in Programmcode geschrieben werden.

Statt:

```text
"Recording started"
```

wird ein Übersetzungsschlüssel verwendet:

```text
recording.started
```

Die tatsächliche Darstellung erfolgt über Sprachdateien.

---

### 2. Deutsch und Englisch als gleichwertige Projektsprache

NC-PoRe wird zweisprachig entwickelt.

Dokumentation:

* Deutsch (English version below)
* English (Deutsche Version oben)

Beide Sprachen werden gleichwertig behandelt.

Der Quellcode und technische Bezeichner verwenden Englisch als gemeinsame technische Sprache.

---

### 3. Erweiterbarkeit für weitere Sprachen

Die Architektur muss neue Sprachen ermöglichen, ohne Änderungen an der Programmlogik zu benötigen.

Neue Übersetzungen sollen ergänzt werden können durch:

* zusätzliche Sprachdateien
* Übersetzungsplattformen
* Community-Beiträge

---

### 4. Übersetzungen als Community-Beitrag

NC-PoRe soll Übersetzungen durch die Community ermöglichen.

Dafür sollen externe Übersetzungssysteme unterstützt werden.

Mögliche Werkzeuge können sein:

* Weblate
* Transifex
* Crowdin
* vergleichbare Systeme

Die konkrete Auswahl eines Werkzeugs ist keine Architekturentscheidung und kann später erfolgen.

---

## Konsequenzen

### Vorteile

* internationale Nutzbarkeit von Anfang an
* bessere Wartbarkeit
* einfachere Community-Beiträge
* klare Trennung von Code und Sprache
* Vorbereitung für zukünftige Märkte und Nutzergruppen

### Nachteile

* höherer initialer Entwicklungsaufwand
* zusätzliche Dateien und Strukturen
* Übersetzungsverwaltung muss organisiert werden

Diese Nachteile werden bewusst akzeptiert.

Die langfristige Flexibilität ist wichtiger als kurzfristige Einfachheit.

---

## Nicht-Ziele

Diese Entscheidung bedeutet nicht:

* dass jede Sprache sofort vollständig unterstützt wird
* dass jede Übersetzung perfekt sein muss, bevor Software veröffentlicht wird
* dass technische Begriffe unnötig übersetzt werden

Die Architektur schafft Möglichkeiten, ohne unnötige Komplexität einzuführen.

---

## Leitgedanke

NC-PoRe soll eine Software sein, die Menschen weltweit nutzen können.

Sprache darf kein technisches Hindernis sein.

Die Software passt sich an Menschen an – nicht Menschen an die Software.

---

# English (Deutsche Version oben)

## Context

NC-PoRe is developed in Germany and is part of an international world.

The project should be designed from the beginning so that people can use and extend the software regardless of language or origin.

Internationalization must not be added later as an afterthought, but must be part of the architecture.

Adding translations to software that was originally designed for only one language often causes:

* high technical costs
* inconsistent user interfaces
* difficult maintenance
* limitations for future extensions

---

## Decision

NC-PoRe will be developed with internationalization support from the beginning.

The software separates technical logic from language-dependent content.

User-visible texts are not stored directly in source code, but provided through a localization system.

Example:

```text
Code

    |
    |
    v

Translation Key

    |
    |
    v

Language Resource

├── Deutsch
├── English
└── additional languages
```

---

## Principles

### 1. No embedded user-facing texts

User-visible texts must not be written directly into application code.

Instead of:

```text
"Recording started"
```

a translation key is used:

```text
recording.started
```

The actual text is provided through language resources.

---

### 2. German and English as equal project languages

NC-PoRe is developed bilingually.

Documentation:

* Deutsch (English version below)
* English (Deutsche Version oben)

Both languages are treated equally.

Source code and technical identifiers use English as the common technical language.

---

### 3. Extensibility for additional languages

The architecture must allow additional languages without changes to application logic.

New translations should be possible through:

* additional language files
* translation platforms
* community contributions

---

### 4. Translations as community contribution

NC-PoRe should allow community-driven translations.

External translation systems should be supported.

Possible tools include:

* Weblate
* Transifex
* Crowdin
* comparable systems

The selection of a specific tool is not an architectural decision and can be made later.

---

## Consequences

### Benefits

* international usability from the beginning
* improved maintainability
* easier community contributions
* clear separation of code and language
* preparation for future markets and users

### Costs

* higher initial development effort
* additional files and structures
* translation management requirements

These costs are consciously accepted.

Long-term flexibility is more important than short-term simplicity.

---

## Non-Goals

This decision does not mean:

* supporting every language immediately
* requiring perfect translations before releases
* translating technical terms unnecessarily

The architecture creates possibilities without introducing unnecessary complexity.

---

## Guiding Principle

NC-PoRe should be software that people around the world can use.

Language must not become a technical barrier.

Software adapts to people – not people to software.
