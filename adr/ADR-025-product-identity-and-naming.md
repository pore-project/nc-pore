# ADR-025: Product Identity and Naming

* Status: Accepted
* Date: 2026-07-23
* Decision Type: Product Definition

---

# Deutsch (English version below)

## Kontext

NC-PoRe wurde ursprünglich als Werkzeug zur Unterstützung einer eigenen Podcast-Produktion entwickelt.

Der erste konkrete Anwendungsfall ist die Produktion eines Podcasts mit mehreren Beteiligten, verteilt über verschiedene Geräte und Plattformen.

Während der Entwicklung wurde deutlich, dass der Anwendungsfall über eine reine Audio-Aufnahme hinausgeht.

Podcast-Produktion umfasst nicht nur das Aufzeichnen von Audiodaten, sondern auch:

* Zusammenarbeit zwischen Teilnehmern
* Verwaltung von Sessions
* Organisation von Medien
* Austausch und Speicherung von Produktionsdaten
* zukünftige Erweiterungen wie Video und weitere Medienformate

Der ursprüngliche Name "NC-PoRe" soll erhalten bleiben, benötigt jedoch eine klare und langfristig tragfähige Bedeutung.

---

## Entscheidung

NC-PoRe steht für:

**Nextcloud Podcast Production Environment**

Die Bedeutung der Bestandteile:

* **NC** → Nextcloud als ursprüngliche Integrationsplattform
* **Po** → Podcast als erster konkreter Anwendungsbereich
* **R** → pRoduction als Fokus auf den gesamten Produktionsprozess, nicht nur die Aufnahme
* **E** → Environment als umfassende Arbeitsumgebung

---

## Bedeutung des Namens

NC-PoRe ist keine reine Recorder-Anwendung.

Die Software wird als Umgebung zur kollaborativen Podcast-Produktion entwickelt.

Der Fokus liegt auf:

* Sessions statt einzelner Dateien
* Zusammenarbeit statt Einzelaufnahme
* Produktionsabläufen statt nur Hardwarezugriff
* Nutzererfahrung statt technischer Komplexität

---

## Verhältnis zu Nextcloud

Nextcloud ist die ursprüngliche technische Heimat von NC-PoRe.

Die erste Version wird bewusst Nextcloud-orientiert entwickelt.

Dies bedeutet jedoch nicht, dass NC-PoRe dauerhaft ausschließlich auf Nextcloud beschränkt bleibt.

Die Architektur bleibt offen für zukünftige Erweiterungen und weitere Provider.

Die Beziehung ist:

```text
NC-PoRe

    |
    |
Nextcloud Integration (V1)

    |
    |
zukünftige Provider möglich
```

---

## Grundprinzipien

### 1. Der Name beschreibt den Ursprung, nicht die Grenze

NC-PoRe erinnert an die Herkunft des Projekts.

Der Name soll jedoch keine zukünftigen Erweiterungen verhindern.

---

### 2. Der Nutzer steht im Mittelpunkt

NC-PoRe wird nicht entwickelt, um technisch möglichst komplex zu sein.

Es soll eine einfache und verständliche Lösung für reale Produktionsanforderungen bieten.

Technische Komplexität bleibt im Hintergrund.

---

### 3. Produktion statt nur Aufnahme

Eine Podcast-Produktion besteht aus mehr als einer Audiodatei.

NC-PoRe betrachtet eine Produktion als zusammenhängenden Prozess:

```text
Production Session

├── Participants
├── Media Streams
├── Metadata
├── Events
├── Assets
└── Exports
```

---

## Konsequenzen

### Vorteile

* klare Produktidentität
* verständliche Zielrichtung
* Erweiterbarkeit über Audio hinaus
* Verbindung zwischen heutiger Nutzung und zukünftiger Vision
* ehrliche Kommunikation über die Herkunft des Projekts

### Nachteile

* der Begriff "Environment" ist weniger konkret als "Recorder"
* der größere Anspruch erfordert langfristige Architekturdisziplin

Diese Nachteile werden bewusst akzeptiert.

---

## Nicht-Ziele

Diese Entscheidung bedeutet nicht:

* dass NC-PoRe sofort eine vollständige Medienproduktionsplattform sein muss
* dass alle zukünftigen Funktionen bereits in Version 1 enthalten sein müssen
* dass Nextcloud durch andere Provider ersetzt werden soll

Die Entwicklung erfolgt schrittweise.

---

## Leitgedanke

NC-PoRe begann als Werkzeug für einen konkreten Podcast.

Die Architektur ermöglicht daraus eine offene Umgebung für kollaborative Podcast-Produktion zu entwickeln.

Der Name erinnert an die Herkunft.

Die Architektur ermöglicht die Zukunft.

---

# English (Deutsche Version oben)

## Context

NC-PoRe was originally created as a tool to support a real podcast production workflow.

The first concrete use case is producing a podcast with multiple participants distributed across different devices and platforms.

During development it became clear that the use case goes beyond simple audio recording.

Podcast production includes more than capturing audio data:

* collaboration between participants
* session management
* media organization
* exchange and storage of production data
* future extensions such as video and additional media formats

The original name "NC-PoRe" should be preserved, but requires a clear and sustainable meaning.

---

## Decision

NC-PoRe stands for:

**Nextcloud Podcast Production Environment**

The meaning of the individual parts:

* **NC** → Nextcloud as the original integration platform
* **Po** → Podcast as the first concrete use case
* **R** → pRoduction, focusing on the complete production process rather than recording only
* **E** → Environment as a complete working environment

---

## Meaning of the Name

NC-PoRe is not a simple recorder application.

The software is developed as an environment for collaborative podcast production.

The focus is on:

* sessions instead of individual files
* collaboration instead of isolated recording
* production workflows instead of only hardware access
* user experience instead of technical complexity

---

## Relationship to Nextcloud

Nextcloud is the original technical home of NC-PoRe.

The first version is intentionally developed with a strong Nextcloud integration.

However, this does not mean NC-PoRe will permanently depend exclusively on Nextcloud.

The architecture remains open for future extensions and additional providers.

Relationship:

```text
NC-PoRe

    |
    |
Nextcloud Integration (V1)

    |
    |
future providers possible
```

---

## Principles

### 1. The name describes the origin, not the limitation

NC-PoRe reflects where the project started.

The name must not prevent future extensions.

---

### 2. Users come first

NC-PoRe is not developed to maximize technical complexity.

It should provide a simple and understandable solution for real production needs.

Technical complexity remains hidden from users.

---

### 3. Production instead of recording only

Podcast production is more than an audio file.

NC-PoRe treats production as a connected process:

```text
Production Session

├── Participants
├── Media Streams
├── Metadata
├── Events
├── Assets
└── Exports
```

---

## Consequences

### Benefits

* clear product identity
* understandable direction
* extensibility beyond audio
* connection between current use and future vision
* honest communication about project origin

### Costs

* the term "Environment" is less concrete than "Recorder"
* the broader vision requires long-term architectural discipline

These costs are consciously accepted.

---

## Non-Goals

This decision does not mean:

* NC-PoRe must immediately become a complete media production platform
* all future features must exist in version 1
* Nextcloud should be replaced by other providers

Development will happen step by step.

---

## Guiding Principle

NC-PoRe started as a tool for a specific podcast.

Its architecture enables the evolution into an open environment for collaborative podcast production.

The name remembers the origin.

The architecture enables the future.
