# ADR Policy

[Deutsche Version](#deutsch) · [English Version](#english-version)

<a id="deutsch"></a>

# ADR-Richtlinie

NC-PoRe Architecture Decision Records folgen einem gemeinsamen Dokumentationsvertrag.

## Sprache und Navigation

Jede ADR ist strikt zweisprachig.

Die deutsche Fassung ist die erste vollständige Fassung, die englische Fassung die zweite vollständige Fassung. Beide Fassungen müssen denselben Entscheidungsinhalt enthalten; keine Sprachfassung ist eine Zusammenfassung der anderen.

Jede Sprachfassung verwendet einen festen Einstiegsanker, damit Leser direkt zur entsprechenden Sprachfassung springen können:

- Deutscher Einstiegsanker: `#deutsch`
- Englischer Einstiegsanker: `#english-version`

Der Dokumentkopf verlinkt von Deutsch nach Englisch; die englische Fassung verlinkt zurück nach Deutsch.

## Verbindliche ADR-Struktur

Jede ADR muss ausdrücklich enthalten:

1. Kontext
2. Problemstellung / Fragestellung
3. Entscheidung
4. Begründung
5. Konsequenzen

Zusätzliche Abschnitte wie „Betrachtete Alternativen“, „Geltungsbereich“, „Nicht-Ziele“, „Abhängigkeiten“ oder „Statushinweise“ können bei Bedarf ergänzt werden. Sie ersetzen jedoch keinen der fünf verbindlichen Abschnitte.

## Öffentliche und interne Entscheidungen

Eine ADR kann öffentlich oder intern sein.

Öffentliche ADRs dokumentieren Architekturprinzipien und Entscheidungen, die NC-PoRE als Teil seiner Open-Source-Projektdokumentation offenlegen kann und will.

Interne ADRs dürfen Implementierungsdetails, unveröffentlichte Fähigkeiten, Produktstrategie, kommerzielle Funktionen, Entitlement-Architektur, zukünftige Produktrichtungen oder andere Informationen dokumentieren, deren Veröffentlichung die geplante Produktoberfläche von NC-PoRE unnötig offenlegen würde.

Der Status „intern“ darf niemals dazu führen, dass die ADR selbst weniger sorgfältig dokumentiert wird. Interne ADRs folgen derselben zweisprachigen Struktur und denselben verbindlichen Abschnitten wie öffentliche ADRs.

## Informationsminimierung

Architektonische Offenheit erfordert nicht die Veröffentlichung der vollständigen zukünftigen Produktplanung.

Öffentliche ADRs sollen das dokumentieren, was erforderlich ist, um die veröffentlichte Architektur zu verstehen und ihre Entscheidungen nachvollziehen zu können. Sie sollen keine unveröffentlichten oder kommerziell geplanten Fähigkeiten allein deshalb detailliert aufzählen, weil die Architektur diese grundsätzlich ermöglichen könnte.

Zukünftige Erweiterbarkeit kann als architektonische Eigenschaft beschrieben werden, ohne eine detaillierte Liste zukünftiger Produkte oder kostenpflichtiger Funktionen zu veröffentlichen.

## Kommerziell sensible Architektur

Wenn eine Architekturentscheidung eine zukünftige kommerzielle Fähigkeit unterstützt, soll die öffentliche ADR nur das stabile Architekturprinzip dokumentieren, dessen Veröffentlichung beabsichtigt ist. Produktspezifische Funktionsdetails, Entitlement-Mechanismen, Feature-Pakete, kommerzielle Stufen und unveröffentlichte Roadmap-Details gehören in interne ADRs, sofern nicht bewusst eine Veröffentlichung beschlossen wurde.

## Verhältnis zu Implementierungsdokumenten

ADR-Dateien beantworten:

> Warum wurde eine Entscheidung getroffen?

Implementierungsdokumente beantworten:

> Wie wird die Entscheidung umgesetzt?

Produkt- und Projektdokumente beantworten:

> Was bauen wir, wann und in welcher Version?

Eine zukünftige Fähigkeit soll nicht allein deshalb in einer ADR veröffentlicht werden, weil sie in einem Implementierungsplan oder einer Architekturdiskussion auftaucht.

---

<a id="english-version"></a>

# ADR Policy

NC-PoRe Architecture Decision Records follow a common documentation contract.

## Language and navigation

Every ADR is strictly bilingual.

The German version is the first complete version and the English version is the second complete version. The two versions must contain the same decision content; neither language is a summary of the other.

Each language section uses a fixed entry anchor so readers can jump directly to the corresponding language version:

- German entry anchor: `#deutsch`
- English entry anchor: `#english-version`

The document header links from German to English, and the English section links back to German.

## Required ADR structure

Every ADR must explicitly contain:

1. Context
2. Problem Statement / Question
3. Decision
4. Rationale
5. Consequences

Additional sections such as Alternatives Considered, Scope, Non-Goals, Dependencies, or Status Notes may be added where useful, but they do not replace any of the five required sections.

## Public versus internal decisions

An ADR may be public or internal.

Public ADRs document architectural principles and decisions that NC-PoRE is prepared to expose as part of its open-source project documentation.

Internal ADRs may document implementation details, unreleased capabilities, product strategy, commercial features, entitlement architecture, future product directions, or other information whose publication would unnecessarily disclose NC-PoRE's planned product surface.

Internal status must never be used to weaken the quality of the ADR itself. Internal ADRs follow the same bilingual structure and required sections as public ADRs.

## Information minimisation

Architectural openness does not require publication of the complete future product roadmap.

Public ADRs should describe what is necessary to understand and preserve the published architecture. They should avoid enumerating unreleased or commercially planned capabilities merely because the architecture could support them.

Future extensibility may be stated as an architectural property without publishing a detailed list of future products or paid features.

## Commercially sensitive architecture

If an architectural decision supports a future commercial capability, the public ADR should document only the stable architectural principle that is appropriate for public disclosure. Product-specific capability details, entitlement mechanisms, feature packaging, commercial tiers, and unreleased roadmap details belong in internal ADRs unless there is a deliberate decision to publish them.

## Relationship to implementation documents

ADR files answer:

> Why was a decision made?

Implementation documents answer:

> How is the decision implemented?

Product and project planning documents answer:

> What are we building, when, and in which release?

A future capability should not be published in an ADR merely because it appears in an implementation plan or architectural discussion.
