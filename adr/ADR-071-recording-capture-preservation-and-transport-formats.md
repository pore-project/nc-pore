# ADR-071 Recording Capture, Preservation and Transport Formats

* Status: Proposed
* Date: 2026-08-24
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

---

# Kontext

NC-PoRe soll plattform- und hostoffen bleiben. Ein Browserclient darf daher weder Nextcloud Talk noch eine andere Hostplattform zu einem Bestandteil des NC-PoRe-Aufzeichnungsmodells machen. Plattformspezifische Fähigkeiten und Einschränkungen werden stattdessen durch den jeweiligen Connector bereitgestellt.

Die Aufnahmeumgebung kann unterschiedliche Audiofähigkeiten zur Verfügung stellen. Ein Browser, ein nativer Desktopclient oder eine andere Hostplattform können unterschiedliche Codecs, Sample-Raten, Kanalaufteilungen, Sample-Formate oder Containerformate anbieten.

Gleichzeitig können für den Transport Anforderungen des Hosts oder Connectors gelten. Diese müssen nicht dem bestmöglichen lokalen Aufnahmeformat entsprechen.

Das bestehende NC-PoRe-Modell bildet Aufnahmen bereits als Tracks und Chunks ab und speichert die explizite Recording Configuration. Eine reale CPAL-Aufnahme sowie lokale Persistenz, Synchronisation und die anschließende Verifikation auf Nextcloud wurden erfolgreich durchlaufen.

Der nächste V1-Schritt ist ein echter Browserclient. Vor dessen Implementierung muss die Grenze zwischen Aufnahme, lokaler Bewahrung und Transport eindeutig definiert sein.

---

# Entscheidung

NC-PoRe trennt die Audioverarbeitung in drei klar voneinander getrennte Formatgrenzen:

```text
CAPTURE
Plattform liefert das bestmögliche zuverlässig verfügbare Aufnahmeformat

        |
        v

PRESERVATION
NC-PoRe bewahrt die höchstmögliche lokal sinnvolle Qualität

        |
        v

TRANSPORT
Host-/Connector-spezifische Darstellung für die Synchronisation
```

---

# Capture Format

Die Capture-Schicht verwendet das **beste Aufnahmeformat, das die jeweilige Aufnahmeumgebung zuverlässig bereitstellen kann**.

NC-PoRe verlangt ausdrücklich nicht, dass jede Plattform dasselbe Capture-Format liefert.

Die jeweiligen Capture-Fähigkeiten sind plattformspezifisch und werden über die entsprechende Client-/Connector-Integration an die NC-PoRe-Recording-Grenze übergeben.

---

# Preservation

Die lokale Aufnahme soll die **höchstmögliche praktisch sinnvolle Qualität** bewahren, die die jeweilige Aufnahmeumgebung zuverlässig liefern kann.

Die lokale Aufnahmequalität darf nicht allein deshalb reduziert werden, weil ein Host ein bestimmtes Transportformat verlangt.

Die konkrete Form der Preservation bleibt dort eine Implementierungsentscheidung, wo die jeweilige Capture-Plattform technische Einschränkungen vorgibt.

Lossless Preservation wird bevorzugt, sofern dies praktisch möglich und von der jeweiligen Plattform unterstützt wird.

---

# Transport

Das Transportformat ist eine eigenständige Angelegenheit und wird vom Capture- und Preservation-Format getrennt betrachtet.

Ein Host oder Connector kann ausdrücklich eine Transportdarstellung konfigurieren, beispielsweise:

* MP3 mit 64 kbit/s
* ein anderes explizit unterstütztes komprimiertes Format

Wenn der Host keine Transportanforderung vorgibt, verwendet NC-PoRe standardmäßig **FLAC, lossless komprimiert**.

Eine Konvertierung in das Transportformat findet an der Transportgrenze statt.

Eine verlustbehaftete Transportkonvertierung darf die lokal bewahrte Aufnahme nicht ersetzen oder verschlechtern.

---

# Verantwortung des Connectors

Ein Connector stellt die plattformabhängigen Fähigkeiten und Einschränkungen für die NC-PoRe-Client-/Application-Grenze bereit.

Dazu kann auch die vom Host vorgegebene Transportkonfiguration gehören.

Diese Anforderungen werden jedoch nicht Bestandteil des NC-PoRe-Core-Domainmodells.

Damit gilt insbesondere:

> **Nextcloud/Talk ist ein Integrationsziel — nicht die Definition des NC-PoRe-Aufnahmeformats.**

---

# Keine verpflichtende Audio-Normalisierung

NC-PoRe verlangt keine frühe Konvertierung jeder Aufnahme in ein einheitliches Audioformat, nur um unterschiedliche Eingaben technisch zu vereinheitlichen.

Insbesondere bedeutet „Normalisierung“ in dieser ADR nicht Lautstärkenormalisierung und ist kein vorgeschriebener Verarbeitungsschritt.

Falls eine spätere architektonische Grenze für eine konkrete Operation eine bestimmte technische Darstellung benötigt, muss die entsprechende Konvertierung explizit an dieser Grenze erfolgen und begründet sein.

---

# Konsequenzen

## Vorteile

* Browser- und native Aufnahmeclients können die tatsächlichen Fähigkeiten ihrer Plattform nutzen.
* Transportanforderungen eines Hosts bestimmen nicht die Qualität der lokalen Aufnahme.
* Die Nextcloud-/Talk-Integration bleibt auf den Connector beschränkt.
* Ein eigenständiger NC-PoRe-Client und zukünftige Hostintegrationen können denselben Client-/Application-Vertrag verwenden.
* FLAC bietet einen sinnvollen hostunabhängigen Default für den Transport.

## Kosten

* Capture, Preservation und Transport müssen im System als unterschiedliche Verantwortungsbereiche modelliert werden.
* Connectoren müssen Hostkonfigurationen bereitstellen, wenn Transportanforderungen existieren.
* Manche Transportformate benötigen eine explizite Konvertierung und verursachen dafür zusätzliche CPU-/Speicherkosten.
* Der konkrete Preservation-Weg im Browser muss noch anhand der tatsächlich verfügbaren Browser-Capture-APIs entschieden werden.

---

# Testing Consequences

Die Trennung ermöglicht getrennte Tests für:

* plattformspezifische Capture-Fähigkeiten
* Preservation ohne Hostabhängigkeit
* Transportkonvertierung und Transportkonfiguration
* Connector-spezifische Vorgaben
* End-to-End-Aufnahme und Synchronisation

---

# Relationship to Existing Architecture

Diese Entscheidung baut insbesondere auf folgenden Architekturentscheidungen auf:

* ADR-001 Local Recording
* ADR-002 Audio Format and Track Concept
* ADR-007 Open Formats and Interoperability
* ADR-008 Client Architecture
* ADR-039 Recording Architecture and Capture Boundary
* ADR-069 Nextcloud Remote Artifact Storage
* ADR-070 Recording Delivery Format

Die Entscheidung definiert die Grenze, auf der der V1-Browserclient aufsetzt. Nextcloud/Talk wird dabei als konkrete Connector-/Hostintegration behandelt und nicht als Grundlage des allgemeinen Clientvertrags.

---

# Future Considerations

Folgende Fragen werden bewusst erst beim Entwurf des Client-Vertrags und der Browserimplementierung entschieden:

* Welche Browser-Capture-APIs und Formate werden in V1 unterstützt?
* Wird Browser-Capture vor der Artifact-Erzeugung als Raw PCM, verlustfrei kodierte Darstellung oder in einer anderen Form bewahrt?
* Welche Transportformate werden in V1 neben FLAC unterstützt?
* Wie wird die Transportformat-Konfiguration im Host-/Connector-Vertrag dargestellt?
* Erfolgt die Transportkonvertierung clientseitig, an der Application-Grenze oder innerhalb des Connectors?
* Wie werden Multi-Track-Aufnahmen und gegebenenfalls unterschiedliche Formatvorgaben pro Track abgebildet?

Diese Fragen dürfen nicht dadurch beantwortet werden, dass der Browserclient von Nextcloud Talk abhängig gemacht wird.

---

# Status

Diese Entscheidung definiert die Formatgrenzen zwischen plattformspezifischer Audioaufnahme, lokaler Preservation und hostabhängigem Transport.

Die konkrete Browser- und Connector-Implementierung erfolgt innerhalb dieser Architekturgrenzen.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# Context

NC-PoRe is intended to remain platform- and host-open. A browser client must therefore not make Nextcloud Talk, or any other host platform, part of the NC-PoRe recording domain model. Platform-specific capabilities and constraints are supplied through connectors.

The capture environment may provide different audio capabilities. A browser, a native desktop client, and another host platform may expose different codecs, sample rates, channel layouts, sample formats, or container formats.

At the same time, transport requirements may be imposed by the host or connector. These do not necessarily need to match the best available local capture format.

The existing NC-PoRe model already represents recordings as tracks and chunks and records explicit recording configuration. Real CPAL capture, local persistence, synchronization and subsequent Nextcloud verification have been demonstrated successfully.

The next V1 step is a real browser client. Before implementing it, the boundary between capture, local preservation and transport must be explicit.

---

# Decision

NC-PoRe separates audio handling into three distinct format boundaries:

```text
CAPTURE
platform provides the best reliably available recording format

        |
        v

PRESERVATION
NC-PoRe retains the highest-quality locally useful representation

        |
        v

TRANSPORT
host/connector-specific representation used for synchronization
```

---

# Capture Format

The capture layer should use the **best recording format reliably available from the current capture environment**.

NC-PoRe does not require every platform to produce the same capture format.

Capture capabilities are platform-specific and are exposed to the NC-PoRe recording boundary by the appropriate client/connector integration.

---

# Preservation

The local recording should preserve the **highest-quality representation that is practical and reliable for the capture environment**.

Preservation must not be degraded merely to satisfy a host's transport requirements.

The exact preservation representation remains an implementation-level decision where the capture platform imposes constraints.

Lossless preservation is preferred whenever practical and supported.

---

# Transport

Transport format is a separate concern from capture and preservation.

A host or connector may explicitly configure a transport representation, for example:

* MP3 at 64 kbit/s
* another explicitly supported compressed representation

When the host provides no transport-format requirement, **FLAC lossless compression is the NC-PoRe default transport representation**.

Conversion to the transport representation occurs at the transport boundary.

A lossy transport conversion must not replace or degrade the preserved local recording.

---

# Connector Responsibility

A connector supplies platform-specific capabilities and constraints to the NC-PoRe client/application boundary.

This may include host-provided transport configuration.

These requirements do not become part of the NC-PoRe Core domain model.

In particular:

> **Nextcloud/Talk is an integration target — not the definition of the NC-PoRe recording format.**

---

# No Mandatory Audio Normalization

NC-PoRe does not require an early conversion of every capture into one uniform audio format merely to make different inputs technically identical.

In this ADR, “normalization” does not mean loudness normalization and is not a mandatory processing step.

If a later architectural boundary requires a specific technical representation for a concrete operation, that conversion must be explicit, occur at that boundary, and be justified.

---

# Consequences

## Benefits

* Browser and native capture clients can use the actual capabilities of their platform.
* Host transport requirements do not dictate local recording quality.
* Nextcloud/Talk integration remains connector-specific.
* A standalone NC-PoRe client and future host integrations can reuse the same client/application contract.
* FLAC provides a sensible host-independent default transport representation.

## Costs

* Capture, preservation and transport must be modeled as distinct responsibilities.
* Connectors must expose host configuration where transport requirements exist.
* Some transport formats require explicit conversion and associated CPU/storage cost.
* The concrete browser preservation path still requires a decision based on the actual browser capture APIs.

---

# Testing Consequences

The separation enables independent testing of:

* platform-specific capture capabilities
* preservation without host dependency
* transport conversion and transport configuration
* connector-specific requirements
* end-to-end recording and synchronization

---

# Relationship to Existing Architecture

This decision builds in particular on:

* ADR-001 Local Recording
* ADR-002 Audio Format and Track Concept
* ADR-007 Open Formats and Interoperability
* ADR-008 Client Architecture
* ADR-039 Recording Architecture and Capture Boundary
* ADR-069 Nextcloud Remote Artifact Storage
* ADR-070 Recording Delivery Format

The decision defines the boundary on which the V1 browser client is built. Nextcloud/Talk is treated as a concrete connector/host integration rather than as the foundation of the general client contract.

---

# Future Considerations

The following questions are deliberately deferred until the client contract and browser implementation are designed:

* Which browser capture APIs and formats are supported in V1?
* Whether browser capture is preserved as raw PCM, a lossless encoded representation, or another representation before artifact creation.
* Which transport formats are supported in V1 besides FLAC?
* How transport-format configuration is represented in the host/connector contract.
* Whether transport conversion is performed client-side, at the application boundary, or by a connector.
* How multi-track recordings and potentially different per-track format constraints are represented.

These questions must not be resolved by making the browser client dependent on Nextcloud Talk.

---

# Status

This decision defines the format boundaries between platform-specific audio capture, local preservation and host-dependent transport.

Concrete browser and connector implementation follows these architectural boundaries.
