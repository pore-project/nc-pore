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

1. Kontext / Context
2. Problemstellung / Problem Statement
3. Entscheidung / Decision
4. Begründung / Rationale
5. Konsequenzen / Consequences

Additional sections such as Alternatives Considered, Scope, Non-Goals, Dependencies, or Status Notes may be added where useful, but they do not replace the required sections.

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
