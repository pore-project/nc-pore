# ADR-076: ADR Governance and Information Boundary

* Status: Accepted
* Date: 2026-09-11
* Decision Type: Architecture / Governance

---

# Deutsch ([English version below](#english-version))

## Kontext

NC-PoRE verwendet Architecture Decision Records, um wesentliche Architekturentscheidungen nachvollziehbar festzuhalten. Die bisher entstandenen ADRs decken die frühe Architekturgrundlage, die erweiterte Architektur und die aktuelle Implementierungsarchitektur ab.

Mit der Weiterentwicklung des Projekts hat sich jedoch die Funktion der ADRs erweitert. Neben stabilen Architekturentscheidungen wurden teilweise auch zukünftige Produktmöglichkeiten und mögliche Erweiterungen beschrieben.

NC-PoRE wird als Open-Source-Projekt entwickelt. Gleichzeitig ist vorgesehen, einen Teil zukünftiger Erweiterungen als kostenpflichtige Add-ons bzw. kommerzielle Funktionen anzubieten.

Eine vollständige Veröffentlichung der zukünftigen Produktoberfläche ist dafür weder technisch notwendig noch strategisch sinnvoll.

## Problemstellung

Es muss zwischen notwendiger architektonischer Transparenz und unnötiger Offenlegung zukünftiger Produkt- und Geschäftspläne unterschieden werden.

Insbesondere darf aus einer öffentlichen ADR-Sammlung nicht ohne Not eine vollständige Roadmap für zukünftige NC-PoRE-Funktionen, Add-ons oder kommerzielle Leistungsmerkmale entstehen.

Gleichzeitig darf die gewünschte Zurückhaltung nicht dazu führen, dass wichtige Architekturentscheidungen, Schnittstellen oder grundlegende Open-Source-Prinzipien undokumentiert bleiben.

Es wird daher eine verbindliche Grenze zwischen öffentlich dokumentierter Architektur und intern dokumentierter Zukunfts-, Produkt- und Geschäftsarchitektur benötigt.

## Entscheidung

NC-PoRE unterscheidet zwischen **öffentlichen ADRs** und **internen ADRs**.

Beide Klassen verwenden dieselben formalen Qualitätsanforderungen:

- vollständige Zweisprachigkeit Deutsch/Englisch
- feste Einsprungmarken `#deutsch` und `#english-version`
- Kontext
- Problemstellung
- Entscheidung
- Begründung
- Konsequenzen

Öffentliche ADRs dokumentieren die für das Open-Source-Projekt relevanten und zur Veröffentlichung bestimmten Architekturentscheidungen.

Interne ADRs dokumentieren Entscheidungen, deren vollständige Veröffentlichung nicht erforderlich oder nicht strategisch sinnvoll ist. Dazu können insbesondere gehören:

- noch nicht veröffentlichte technische Fähigkeiten
- zukünftige Produktfunktionen
- Add-on-Grenzen
- Entitlement- und Lizenzierungsarchitektur
- kommerzielle Feature-Pakete
- Produkt-Roadmaps
- interne Architekturvarianten

Eine öffentliche ADR darf Erweiterbarkeit als architektonisches Prinzip beschreiben. Sie soll jedoch keine unnötig detaillierte Aufzählung zukünftiger oder kommerziell geplanter Funktionen enthalten.

Die Entscheidung über die öffentliche oder interne Einstufung erfolgt **vor der Veröffentlichung** einer ADR. Eine technische Architekturentscheidung wird nicht allein deshalb öffentlich dokumentiert, weil ihre Existenz für die interne Entwicklung bekannt ist.

Die interne Einstufung ist keine Geheimhaltungs- oder Sicherheitsklassifikation im rechtlichen Sinn. Sie bezeichnet die gewünschte Dokumentationsgrenze innerhalb des Projekts.

## Begründung

Die Architektur von NC-PoRE soll nachvollziehbar und offen bleiben. Daraus folgt jedoch nicht, dass jede zukünftige Produktidee öffentlich angekündigt werden muss.

Eine klare Trennung verhindert zwei gegensätzliche Fehler:

1. Zu wenig Dokumentation: wichtige Architekturentscheidungen bleiben implizit und gehen verloren.
2. Zu viel Offenlegung: zukünftige kommerzielle Funktionen und Produktpläne werden unnötig vorab veröffentlicht.

Die Trennung erlaubt es, den offenen Kern transparent zu dokumentieren und gleichzeitig die konkrete zukünftige Produktoberfläche zurückhaltend zu behandeln.

Sie unterstützt außerdem die bereits bestehende Trennung zwischen Architekturentscheidung, Implementierung und Produktplanung.

## Konsequenzen

### Positive Konsequenzen

- Öffentliche ADRs bleiben nachvollziehbar und aussagekräftig.
- Interne Architekturentscheidungen können vollständig dokumentiert werden, ohne sie automatisch öffentlich zu machen.
- Zukünftige kommerzielle Funktionen müssen nicht vor ihrer Markteinführung öffentlich angekündigt werden.
- Die Open-Source-Architektur bleibt transparent, wo Transparenz für ihre Nutzung und Weiterentwicklung erforderlich ist.
- Die ADR-Qualität bleibt unabhängig davon erhalten, ob eine Entscheidung öffentlich oder intern dokumentiert wird.
- Künftige ADRs erhalten eine einheitliche formale Struktur.

### Negative Konsequenzen

- Es entsteht eine zusätzliche Entscheidung darüber, ob eine ADR öffentlich oder intern geführt wird.
- Interne Dokumentation benötigt einen kontrollierten Ablageort und einen definierten Zugriff.
- Beim Überarbeiten älterer ADRs muss geprüft werden, ob darin unbeabsichtigt zukünftige Produktinformationen veröffentlicht werden.
- Eine interne ADR kann nicht als Begründung dafür dienen, architektonisch notwendige öffentliche Dokumentation wegzulassen.

## Abgrenzung

Diese ADR entscheidet nicht:

- welche konkreten Funktionen künftig kostenpflichtig werden
- welche Add-ons angeboten werden
- welche Preise oder Lizenzmodelle gelten
- welche konkrete interne Ablage- oder Zugriffsinfrastruktur verwendet wird
- welche zukünftigen Produktversionen veröffentlicht werden

Diese Entscheidungen werden in dafür vorgesehenen internen Dokumenten getroffen.

---

# English Version ([Deutsche Version oben](#deutsch))

## Context

NC-PoRE uses Architecture Decision Records to document significant architectural decisions in a traceable manner. Existing ADRs cover the early architectural foundation, extended architecture, and current implementation architecture.

As the project evolved, the role of ADRs also expanded. In some cases, future product capabilities and possible extensions were described alongside stable architectural decisions.

NC-PoRE is developed as an open-source project. At the same time, some future extensions are intended to be offered as paid add-ons or commercial features.

Publishing the complete future product surface is neither technically necessary nor strategically useful for that purpose.

## Problem Statement

A distinction must be made between necessary architectural transparency and unnecessary disclosure of future product and business plans.

In particular, a public ADR collection should not unnecessarily become a complete roadmap of future NC-PoRE features, add-ons, or commercial capabilities.

At the same time, the desired restraint must not result in important architectural decisions, interfaces, or fundamental open-source principles remaining undocumented.

A binding boundary is therefore required between publicly documented architecture and internally documented future, product, and business architecture.

## Decision

NC-PoRE distinguishes between **public ADRs** and **internal ADRs**.

Both classes use the same formal quality requirements:

- complete German/English bilingual documentation
- fixed entry anchors `#deutsch` and `#english-version`
- Context
- Problem Statement
- Decision
- Rationale
- Consequences

Public ADRs document architectural decisions relevant to the open-source project and intended for publication.

Internal ADRs document decisions whose complete publication is not required or not strategically useful. This may include, in particular:

- unreleased technical capabilities
- future product features
- add-on boundaries
- entitlement and licensing architecture
- commercial feature packages
- product roadmaps
- internal architectural variants

A public ADR may describe extensibility as an architectural principle. It should not, however, unnecessarily enumerate future or commercially planned capabilities in detail.

The public or internal classification is decided **before publication** of an ADR. An architectural decision is not made public merely because its existence is known to the internal development team.

The internal classification is not a legal confidentiality or security classification. It denotes the intended documentation boundary within the project.

## Rationale

NC-PoRE's architecture should remain understandable and open. This does not mean that every future product idea must be announced publicly.

A clear separation prevents two opposite failures:

1. Insufficient documentation: important architectural decisions remain implicit and may be lost.
2. Excessive disclosure: future commercial features and product plans are unnecessarily published in advance.

The separation allows the open core to remain transparent while treating the concrete future product surface with appropriate restraint.

It also supports the existing separation between architectural decisions, implementation, and product planning.

## Consequences

### Positive Consequences

- Public ADRs remain understandable and meaningful.
- Internal architectural decisions can be documented completely without automatically becoming public.
- Future commercial features do not have to be announced before their market release.
- The open-source architecture remains transparent where transparency is required for use and further development.
- ADR quality remains independent of whether a decision is documented publicly or internally.
- Future ADRs receive a consistent formal structure.

### Negative Consequences

- An additional decision is required to classify an ADR as public or internal.
- Internal documentation requires a controlled location and defined access.
- Older ADRs must be reviewed for unintended disclosure of future product information when they are revised.
- An internal ADR must not be used as a reason to omit architecturally necessary public documentation.

## Scope

This ADR does not decide:

- which specific features will become paid in the future
- which add-ons will be offered
- which prices or licensing models will apply
- which concrete internal storage or access infrastructure will be used
- which future product versions will be released

These decisions will be made in dedicated internal documents.
