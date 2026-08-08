# NC-PoRe Documentation Format

## Deutsche Version ([English version below](#english-version))

---

# Zweck

Diese Datei definiert verbindliche Formatregeln für die technische Dokumentation von NC-PoRe.

Die Regeln gelten insbesondere für:

* Architecture Decision Records (ADRs)
* project-status.md
* implementation-plan.md
* technische Projektbeschreibungen
* vergleichbare technische Dokumente

Diese Datei ist Bestandteil der Dokumentationsgrundlage von NC-PoRe.

---

# Sprachversionen

Dieses Dokument besteht aus zwei gleichwertigen Sprachversionen:

1. Deutsche Version
2. English Version

Die deutsche Version steht vor der englischen Version.

Beide Versionen sind Bestandteile **desselben Dokuments**. Die englische Version ist die englische Entsprechung der deutschen Version und keine eigenständige Dokumentationsvariante.

Für beide Sprachversionen gilt:

* Inhaltliche Aussagen müssen übereinstimmen.
* Technische Entscheidungen müssen identisch wiedergegeben werden.
* Technische Identifier bleiben sprachübergreifend unverändert.
* Formatierungsregeln gelten identisch.
* Änderungen an der deutschen Version müssen in der englischen Version nachvollzogen werden.
* Änderungen an der englischen Version dürfen keine davon abweichende technische Aussage erzeugen.

Die Sprachtrennung dient ausschließlich der Zugänglichkeit für unterschiedliche Lesergruppen.

Sie stellt keine fachliche oder normative Trennung dar.

---

# Grundsatz

Technische Dokumentation soll präzise, nachvollziehbar und formal konsistent sein.

Die Formatierung dient der Lesbarkeit und Strukturierung. Sie soll technische Begriffe nicht durch unnötige Markdown-Auszeichnung verändern.

---

# Technische Identifier

Technische Identifier werden grundsätzlich **ohne Inline-Backticks** geschrieben.

Beispiele:

ArtifactRecoveryService

PersistenceProvider

LocalArtifactRegistry

ArtifactCoordinator

InMemoryPersistenceProvider

FilesystemPersistenceProvider

Das gilt sowohl für die deutsche als auch für die englische Version eines Dokuments.

Ein technischer Identifier wird nicht allein deshalb mit Backticks ausgezeichnet, weil es sich um einen Namen einer Klasse, Schnittstelle, Komponente oder eines anderen technischen Elements handelt.

---

# Markdown-Codeformatierung

Backticks werden nur verwendet, wenn tatsächlich Code oder eine technische Darstellung als Code formatiert werden soll.

Dazu gehören insbesondere:

* Shell-Befehle
* Quellcode
* Konfigurationsdateien
* echte Codefragmente
* technische Darstellungen, die ausdrücklich als Codeblock formatiert werden

Ein technischer Identifier allein ist **kein Grund für Inline-Backticks**.

---

# Technische Diagramme

Textbasierte technische Diagramme dürfen als Markdown-Codeblock dargestellt werden.

Beispiel:

```text
PersistenceProvider
        |
        v
ArtifactRecoveryService
        |
        v
LocalArtifactRegistry
```

Die Codeblock-Formatierung dient hier der Darstellung der Struktur.

Sie bedeutet nicht, dass die einzelnen technischen Identifier als Inline-Code formatiert werden sollen.

---

# Verankerung

Diese Dokumentationsregel ist Bestandteil der technischen Dokumentationsgrundlage von NC-PoRe.

Andere technische Dokumente sollen diese Datei nicht durch abweichende lokale Formatregeln außer Kraft setzen.

Bei der Erstellung oder Änderung technischer Dokumentation ist diese Datei als verbindliche Formatvorgabe zu berücksichtigen.

Die Regel zu den Sprachversionen ist Bestandteil dieser Formatvorgabe. Deutsche und englische Versionen eines Dokuments sind daher gemeinsam zu pflegen und dürfen nicht unabhängig voneinander weiterentwickelt werden.

---

# Ziel

NC-PoRe verwendet technische Dokumentation als nachvollziehbare Grundlage für Architektur, Entscheidungen und Implementierung.

Deshalb sollen Inhalt und Form konsistent bleiben.

Die Formatierung soll die technische Aussage unterstützen und nicht selbst zu einer Quelle von Inkonsistenzen werden.

---

# English Version ([Deutsche Version oben](#deutsche-version))

---

# Purpose

This file defines binding formatting rules for the technical documentation of NC-PoRe.

The rules apply in particular to:

* Architecture Decision Records (ADRs)
* project-status.md
* implementation-plan.md
* technical project descriptions
* comparable technical documents

This file is part of the documentation foundation of NC-PoRe.

---

# Language Versions

This document consists of two equivalent language versions:

1. German Version
2. English Version

The German version precedes the English version.

Both versions are parts of **the same document**. The English version is the English equivalent of the German version and is not an independent documentation variant.

The following rules apply to both language versions:

* Statements must be consistent.
* Technical decisions must be represented identically.
* Technical identifiers remain unchanged across languages.
* Formatting rules apply identically.
* Changes to the German version must be reflected in the English version.
* Changes to the English version must not introduce a conflicting technical statement.

The separation into language versions exists solely to make the documentation accessible to different reader groups.

It does not constitute a technical or normative separation.

---

# Principle

Technical documentation should be precise, traceable, and formally consistent.

Formatting is used for readability and structure. It must not alter the presentation of technical terminology through unnecessary Markdown markup.

---

# Technical Identifiers

Technical identifiers are generally written **without inline backticks**.

Examples:

ArtifactRecoveryService

PersistenceProvider

LocalArtifactRegistry

ArtifactCoordinator

InMemoryPersistenceProvider

FilesystemPersistenceProvider

This applies equally to the German and English versions of a document.

A technical identifier is not enclosed in backticks merely because it is the name of a class, interface, component, or other technical element.

---

# Markdown Code Formatting

Backticks are used only when actual code or a technical representation is intentionally formatted as code.

This includes in particular:

* shell commands
* source code
* configuration files
* actual code fragments
* technical representations that are explicitly formatted as code blocks

A technical identifier alone is **not a reason to use inline backticks**.

---

# Technical Diagrams

Text-based technical diagrams may be represented as Markdown code blocks.

Example:

```text
PersistenceProvider
        |
        v
ArtifactRecoveryService
        |
        v
LocalArtifactRegistry
```

The code-block formatting is used here to preserve the structure of the diagram.

It does not mean that the individual technical identifiers should be formatted as inline code.

---

# Anchoring

This documentation rule is part of the technical documentation foundation of NC-PoRe.

Other technical documents must not override this rule through conflicting local formatting conventions.

When creating or modifying technical documentation, this file is to be treated as a binding formatting reference.

The rule concerning language versions is part of this formatting specification. German and English versions of a document must therefore be maintained together and must not be developed independently.

---

# Goal

NC-PoRe uses technical documentation as a traceable foundation for architecture, decisions, and implementation.

Content and form should therefore remain consistent.

Formatting should support the technical statement rather than become a source of inconsistencies itself.
