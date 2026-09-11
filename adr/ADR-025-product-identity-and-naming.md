<a id="deutsch"></a>

# Deutsch

# ADR-025: Product Identity and Naming

* Status: Accepted
* Date: 2026-07-23
* Decision Type: Product Definition

---

## Kontext

NC-PoRe wurde ursprünglich als Werkzeug zur Unterstützung einer eigenen Podcast-Produktion entwickelt. Der erste konkrete Anwendungsfall ist die Produktion eines Podcasts mit mehreren Beteiligten, verteilt über verschiedene Geräte und Plattformen.

Während der Entwicklung wurde deutlich, dass der Anwendungsfall über eine reine Audio-Aufnahme hinausgeht. Podcast-Produktion umfasst Zusammenarbeit, Session-Verwaltung, Organisation und Austausch von Produktionsdaten.

---

## Entscheidung

NC-PoRe steht für:

**Nextcloud Podcast Production Environment**

Die Bedeutung der Bestandteile:

* **NC** → Nextcloud als ursprüngliche Integrationsplattform
* **Po** → Podcast als erster konkreter Anwendungsbereich
* **R** → pRoduction als Fokus auf den Produktionsprozess, nicht nur die Aufnahme
* **E** → Environment als umfassende Arbeitsumgebung

---

## Bedeutung des Namens

NC-PoRe ist keine reine Recorder-Anwendung. Die Software wird als Umgebung zur kollaborativen Podcast-Produktion verstanden.

Der Fokus liegt auf:

* Sessions statt einzelner Dateien
* Zusammenarbeit statt Einzelaufnahme
* Produktionsabläufen statt nur Hardwarezugriff
* Nutzererfahrung statt technischer Komplexität

---

## Verhältnis zu Nextcloud

Nextcloud ist die ursprüngliche technische Heimat von NC-PoRe und die Integrationsplattform der ersten Version.

Die Architektur der Software bleibt dabei von der internen Struktur der Host-Anwendung getrennt. Die konkrete technische Integration wird in den jeweiligen Architekturentscheidungen beschrieben.

---

## Grundprinzipien

### 1. Der Name beschreibt den Ursprung

NC-PoRe erinnert an die Herkunft des Projekts und beschreibt den ursprünglichen Anwendungskontext.

### 2. Der Nutzer steht im Mittelpunkt

NC-PoRe soll eine einfache und verständliche Lösung für reale Produktionsanforderungen bieten. Technische Komplexität bleibt im Hintergrund.

### 3. Produktion statt nur Aufnahme

Eine Podcast-Produktion besteht aus mehr als einer Audiodatei. NC-PoRe betrachtet eine Produktion als zusammenhängenden Prozess:

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
* konsistente Sprache für das Produkt
* ehrliche Kommunikation über die Herkunft des Projekts

### Nachteile

* der Begriff "Environment" ist weniger konkret als "Recorder"
* der breitere fachliche Anspruch erfordert architektonische Disziplin

---

## Nicht-Ziele

Diese Entscheidung legt nicht fest, welche zusätzlichen Medien, Provider oder Funktionen zu einem späteren Zeitpunkt angeboten werden.

---

## Leitgedanke

NC-PoRe begann als Werkzeug für einen konkreten Podcast und wird als Umgebung für kollaborative Podcast-Produktion verstanden. Der Name beschreibt die Herkunft des Projekts; die konkrete Architektur wird durch die jeweiligen ADRs festgelegt.

---

<a id="english-version"></a>

# English Version

# ADR-025: Product Identity and Naming

* Status: Accepted
* Date: 2026-07-23
* Decision Type: Product Definition

---

## Context

NC-PoRe was originally created as a tool to support a real podcast production workflow. The first concrete use case is producing a podcast with multiple participants distributed across different devices and platforms.

During development it became clear that the use case goes beyond simple audio recording. Podcast production includes collaboration, session management, organization and exchange of production data.

---

## Decision

NC-PoRe stands for:

**Nextcloud Podcast Production Environment**

The meaning of the individual parts:

* **NC** → Nextcloud as the original integration platform
* **Po** → Podcast as the first concrete use case
* **R** → pRoduction, focusing on the production process rather than recording only
* **E** → Environment as a complete working environment

---

## Meaning of the Name

NC-PoRe is not a simple recorder application. The software is understood as an environment for collaborative podcast production.

The focus is on:

* sessions instead of individual files
* collaboration instead of isolated recording
* production workflows instead of only hardware access
* user experience instead of technical complexity

---

## Relationship to Nextcloud

Nextcloud is the original technical home of NC-PoRe and the integration platform of the first version.

The software architecture remains separated from the internal structure of the host application. The concrete technical integration is described by the respective architectural decisions.

---

## Principles

### 1. The name describes the origin

NC-PoRe reflects where the project started and describes its original application context.

### 2. Users come first

NC-PoRe should provide a simple and understandable solution for real production needs. Technical complexity remains hidden from users.

### 3. Production instead of recording only

Podcast production is more than an audio file. NC-PoRe treats production as a connected process:

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
* consistent product terminology
* honest communication about project origin

### Costs

* the term "Environment" is less concrete than "Recorder"
* the broader domain scope requires architectural discipline

---

## Non-Goals

This decision does not define which additional media, providers or functions may be offered at a later point.

---

## Guiding Principle

NC-PoRe started as a tool for a specific podcast and is understood as an environment for collaborative podcast production. The name describes the origin of the project; concrete architecture is defined by the respective ADRs.
