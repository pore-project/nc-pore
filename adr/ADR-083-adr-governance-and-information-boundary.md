# ADR-083: ADR Governance and Information Boundary

* Status: Accepted
* Date: 2026-09-11
* Decision Type: Governance

---

# Deutsch ([English version below](#english-version))

## Kontext

NC-PoRe verwendet Architecture Decision Records (ADRs), um wesentliche Architektur- und Entwicklungsentscheidungen nachvollziehbar zu dokumentieren.

Nicht jede interne Entwicklungsinformation gehört automatisch in die öffentliche Projektdokumentation. Entscheidungen können zunächst intern dokumentiert werden, ohne dass daraus eine öffentliche Verpflichtung oder eine Veröffentlichungspflicht entsteht.

Gleichzeitig soll die Trennung zwischen öffentlicher und interner Dokumentation klar, nachvollziehbar und technisch belastbar sein.

---

# Entscheidung

NC-PoRe unterscheidet zwischen **öffentlichen ADRs** und **internen ADRs**.

Öffentliche ADRs dokumentieren Entscheidungen, die Bestandteil der öffentlich nachvollziehbaren Projektarchitektur sind.

Interne ADRs dokumentieren Entscheidungen und Informationen, deren vollständige Veröffentlichung nicht erforderlich oder (noch) nicht beabsichtigt ist.

Die interne Dokumentation ist kein automatischer Bestandteil der öffentlichen Projektdokumentation.

---

# Öffentliche ADRs

Öffentliche ADRs:

* liegen im zentralen öffentlichen ADR-Bereich
* sind Bestandteil der öffentlichen Projektdokumentation
* können von Community und externen Beitragenden eingesehen werden
* beschreiben die für das öffentliche Projekt relevanten Entscheidungen

Die Dateinamen und die Repository-Struktur dienen als Navigation; ein separater öffentlicher ADR-Index ist nicht erforderlich.

---

# Interne ADRs

Interne ADRs dürfen Informationen dokumentieren, deren Veröffentlichung nicht beabsichtigt ist.

Sie können beispielsweise Entscheidungen und Informationen enthalten, die zunächst nur für die interne Entwicklung erforderlich sind.

Interne ADRs:

* sind technisch von der öffentlichen Dokumentation getrennt
* werden nicht automatisch in öffentliche Übersichten aufgenommen
* werden nicht automatisch Teil öffentlicher Changelogs oder Releases
* werden nicht automatisch durch öffentliche Dokumentationsprozesse veröffentlicht

Die bloße Verwendung einer internen ADR-Nummer macht den Inhalt nicht öffentlich.

---

# Informationsgrenze

Die öffentliche Dokumentation beschreibt die öffentlich relevanten Architekturentscheidungen und tatsächlichen Projektgrundlagen.

Sie soll keine Fähigkeiten allein deshalb detailliert aufzählen, weil die Architektur diese grundsätzlich ermöglichen könnte.

Interne ADRs dürfen dagegen Informationen dokumentieren, deren vollständige Veröffentlichung nicht erforderlich oder (noch) nicht beabsichtigt ist, einschließlich noch nicht veröffentlichter Entwicklungsentscheidungen oder anderer bewusst zurückgehaltener Informationen.

---

# Veröffentlichung interner ADRs

Eine interne ADR kann später bewusst veröffentlicht werden, wenn dies als sinnvoll und angemessen entschieden wird.

Eine solche Veröffentlichung ist eine eigenständige Entscheidung. Sie erfolgt nicht automatisch durch Code-Merges, Releases oder andere öffentliche Änderungen.

---

# Verhältnis zu Code und Releases

Das Zusammenführen von Code in einen öffentlichen Branch macht eine zugehörige interne ADR nicht automatisch öffentlich.

Ebenso bedeutet eine Veröffentlichung von Code nicht, dass alle internen Entscheidungen oder Entwicklungsinformationen veröffentlicht werden müssen.

---

# Konsequenzen

## Positive Auswirkungen

* klare Trennung zwischen öffentlicher und interner Dokumentation
* nachvollziehbare öffentliche Architektur
* interne Entwicklungsarbeit kann dokumentiert werden, ohne automatisch veröffentlicht zu werden
* bewusste spätere Veröffentlichung bleibt möglich
* Community-Beiträge und öffentliche Erweiterbarkeit werden nicht verhindert

## Negative Auswirkungen

* interne und öffentliche Dokumentation müssen organisatorisch und technisch getrennt verwaltet werden
* Veröffentlichungsentscheidungen müssen bewusst getroffen werden
* Informationen dürfen nicht versehentlich aus internen Dokumenten in öffentliche Prozesse gelangen

Diese Nachteile werden bewusst akzeptiert.

---

# Leitgedanke

Nicht jede interne Entscheidung muss öffentlich sein.

Öffentliche Architektur soll nachvollziehbar bleiben, ohne interne Entwicklung unnötig vorwegzunehmen.

**Öffentlich, was öffentlich sein soll. Intern, was zunächst intern bleiben soll. Bewusst veröffentlichen, wenn es sinnvoll ist.**

---

# English Version ([Deutsche Version oben](#deutsch))

## Context

NC-PoRe uses Architecture Decision Records (ADRs) to document significant architecture and development decisions in a traceable way.

Not every internal development detail automatically belongs in the public project documentation. Decisions may initially be documented internally without creating a public commitment or publication requirement.

At the same time, the separation between public and internal documentation must be clear, traceable and technically robust.

---

# Decision

NC-PoRe distinguishes between **public ADRs** and **internal ADRs**.

Public ADRs document decisions that form part of the publicly traceable project architecture.

Internal ADRs document decisions and information whose complete publication is not required or (yet) not intended.

Internal documentation is not an automatic part of the public project documentation.

---

# Public ADRs

Public ADRs:

* reside in the central public ADR area
* are part of the public project documentation
* can be reviewed by the community and external contributors
* describe decisions relevant to the public project

File names and repository structure provide navigation; a separate public ADR index is not required.

---

# Internal ADRs

Internal ADRs may document information whose publication is not intended.

They may contain decisions and information that are initially required only for internal development.

Internal ADRs:

* are technically separated from public documentation
* are not automatically included in public overviews
* are not automatically part of public changelogs or releases
* are not automatically published by public documentation processes

Using an internal ADR number alone does not make its content public.

---

# Information Boundary

Public documentation describes publicly relevant architecture decisions and actual project foundations.

It should not enumerate capabilities in detail merely because the architecture could theoretically support them.

Internal ADRs may document information whose complete publication is not required or (yet) not intended, including unreleased development decisions or other deliberately withheld information.

---

# Publication of Internal ADRs

An internal ADR may later be deliberately published when this is considered useful and appropriate.

Such publication is a separate decision. It does not happen automatically through code merges, releases or other public changes.

---

# Relationship to Code and Releases

Merging code into a public branch does not automatically make an associated internal ADR public.

Likewise, publishing code does not mean that all internal decisions or development information must be published.

---

# Consequences

## Positive Effects

* clear separation between public and internal documentation
* traceable public architecture
* internal development work can be documented without automatic publication
* deliberate later publication remains possible
* community contributions and public extensibility are not prevented

## Negative Effects

* internal and public documentation must be managed separately, both organizationally and technically
* publication decisions must be deliberate
* information must not accidentally move from internal documentation into public processes

These disadvantages are consciously accepted.

---

# Guiding Principle

Not every internal decision needs to be public.

Public architecture should remain traceable without unnecessarily exposing internal development too early.

**Public what should be public. Internal what should initially remain internal. Publish deliberately when appropriate.**
