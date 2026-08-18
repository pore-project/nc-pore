# Deutsch ([English version below](#english-version))

# ADR-039: Storage Provider and Retention Policy Boundaries

## Status

Proposed

## Date

2026-08-17

## Decision Type

Architecture

---

# Kontext

NC-PoRe ist self-hosted und User/Betreiber sollen die Kontrolle über ihre Produktionsdaten behalten.

Der technische Storage-Mechanismus muss dennoch austauschbar bleiben. Local Filesystem, Nextcloud-basierter Storage, S3-kompatibler Storage, WebDAV und zukünftige Provider sind plausible Implementierungen.

Der Vergleich mit Ennuicastr macht außerdem Retention als sinnvolles Konzept sichtbar. Die Retention Policy darf jedoch nicht zu einer impliziten Domain-Regel für ein Recording werden.

---

# Entscheidung

NC-PoRe hält den **Core unabhängig von konkreten Storage Providern**.

Core und Application Layer arbeiten mit Artifact- und Storage-Fähigkeiten statt mit provider-spezifischen APIs.

Konkrete Provider werden hinter einer Storage-Provider-Grenze implementiert.

Konzeptionell:

```text
                    NC-PoRe Core
                         |
                  Artifact Storage API
                         |
          +--------------+--------------+
          |              |              |
      Local FS        S3-compatible   WebDAV
          |              |              |
          +--------------+--------------+
                         |
                  Future providers
```

Nextcloud bleibt eine wichtige Integration bzw. ein wichtiger Provider im NC-PoRe-Ökosystem, aber das Domain-Modell darf nicht von Nextclouds interner Storage-Repräsentation abhängen.

---

# Architekturprinzip

> Der Core weiß, was gespeichert werden muss; der Provider weiß, wie und wo es gespeichert wird.

Storage Provider dürfen keine fachlichen Domain-Autoritäten werden.

---

# Data Ownership

Self-hosted Storage ist das Default-Produktmodell.

Optionale externe Storage- oder Processing-Services können später als ausdrücklich gewählte Provider-/Add-on-Fähigkeiten hinzukommen. Solche Services dürfen Produktionsdaten nicht stillschweigend außerhalb der vom Betreiber gewählten Storage-Grenze verschieben.

---

# Retention Policy

Retention wird als **Storage Policy** behandelt und nicht als intrinsische Recording-Lifecycle-Regel.

Ein Deployment kann später beispielsweise festlegen:

* Raw Capture für einen definierten Zeitraum behalten
* abgeleitete Production Artifacts dauerhaft behalten
* fehlgeschlagene/unvollständige Artifacts nach einem definierten Zeitraum entfernen
* alles unbegrenzt behalten

Das Domain-Modell darf keine universelle Ablaufdauer voraussetzen.

---

# Formatunabhängigkeit

Storage Provider speichern Artifacts als opaque Payloads zusammen mit den für die Artifact-Abstraktion erforderlichen Metadaten.

Ein Provider muss nicht verstehen, ob ein Artifact WAV, FLAC, Opus, WebM oder ein anderes Format enthält.

Dadurch bleiben Formatentscheidungen in Capture-, Reconstruction- und Processing-Schichten und werden nicht Teil der Storage-Implementierung.

---

# Konsequenzen

## Positive Auswirkungen

* Storage-Technologien bleiben austauschbar
* Self-hosting bleibt der Default
* externer Storage kann ergänzt werden, ohne das Domain-Modell zu verändern
* Retention Policy kann sich unabhängig von Domain-Lifecycles entwickeln
* Storage bleibt von Medienformaten entkoppelt

---

## Negative Auswirkungen

* Provider-Abstraktion führt zusätzliche Interfaces ein
* provider-spezifische Fähigkeiten können eine explizite Capability-Aushandlung erfordern
* Retention und Löschung benötigen sorgfältige Policy- und Audit-Behandlung

---

# Betrachtete Alternativen

## Direkter Provider-Zugriff aus dem Core

Verworfen. Dadurch würde Domain Logic an Infrastruktur-Technologie gekoppelt und alternative Provider würden schwerer implementierbar.

---

## Verbindliche universelle Retention-Dauer

Verworfen. Self-hosted Deployments haben unterschiedliche betriebliche, rechtliche und produktbezogene Anforderungen.

---

## Binäre Medien direkt in der Domain-Datenbank speichern

Als allgemeine Architekturregel verworfen. Binäre Artifacts sollen über Artifact Storage behandelt werden, während Metadaten Teil des strukturierten Application Models bleiben.

---

# Beziehung zu bestehender Architektur

Diese Entscheidung konkretisiert ADR-026, das die Storage Provider Layer und Provider-Unabhängigkeit bereits festlegt.

Sie ergänzt explizite Grenzen für Artifact Storage, Data Ownership und Retention Policy.

Sie ergänzt ADR-038, indem sie festlegt, wo Raw und Derived Artifacts gespeichert werden können, ohne das Domain-Modell an ein bestimmtes Backend zu koppeln.

---

# Zukünftige Betrachtungen

Eine spätere Storage-Implementierungs-ADR muss das konkrete Provider-Interface, Atomicity Guarantees, Integrity Verification, Resumable Writes, Deletion Semantics und das Provider-Capability-Modell definieren.

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-039: Storage Provider and Retention Policy Boundaries

## Status

Proposed

## Date

2026-08-17

## Decision Type

Architecture

---

# Context

NC-PoRe is self-hosted and users/operators should retain control over their production data.

The technical storage mechanism must nevertheless remain replaceable. Local filesystem storage, Nextcloud-backed storage, S3-compatible storage, WebDAV and future providers are plausible implementations.

The comparison with Ennuicastr also highlights retention as a useful concept. Retention policy must not, however, become an implicit domain rule for a Recording.

---

# Decision

NC-PoRe keeps the **Core independent of concrete storage providers**.

Core and application layers operate on artifact and storage capabilities rather than provider-specific APIs.

Concrete providers are implemented behind a storage-provider boundary.

Conceptually:

```text
                    NC-PoRe Core
                         |
                  Artifact Storage API
                         |
          +--------------+--------------+
          |              |              |
      Local FS        S3-compatible   WebDAV
          |              |              |
          +--------------+--------------+
                         |
                  Future providers
```

Nextcloud remains an important integration/provider in the NC-PoRE ecosystem, but the domain model must not depend on Nextcloud's internal storage representation.

---

# Architectural Principle

> The Core knows what must be stored; the provider knows how and where it is stored.

Storage providers must not become domain authorities.

---

# Data Ownership

Self-hosted storage is the default product model.

Optional external storage or processing services may later be added as explicitly selected provider/add-on capabilities. Such services must not silently move production data outside the storage boundary chosen by the operator.

---

# Retention Policy

Retention is treated as a **storage policy**, not as an intrinsic Recording lifecycle rule.

A deployment may later define policies such as:

* retain Raw Capture for a defined period
* retain derived Production Artifacts permanently
* remove failed/incomplete Artifacts after a defined period
* retain everything indefinitely

The domain model must not assume a universal expiry period.

---

# Format Independence

Storage providers store Artifacts as opaque payloads together with the metadata required by the artifact abstraction.

A provider does not need to understand whether an Artifact contains WAV, FLAC, Opus, WebM or another format.

This keeps format decisions in capture, reconstruction and processing layers rather than in storage implementations.

---

# Consequences

## Positive Effects

* storage technologies remain replaceable
* self-hosting remains the default
* external storage can be added without changing the domain model
* retention policy can evolve independently of domain lifecycles
* storage remains independent of media formats

---

## Negative Effects

* provider abstraction introduces additional interfaces
* provider-specific capabilities may require explicit capability negotiation
* retention and deletion require careful policy and audit handling

---

# Alternatives Considered

## Direct Provider Access from the Core

Rejected. This would couple domain logic to infrastructure technology and make alternative providers harder to implement.

---

## Mandatory Universal Retention Period

Rejected. Self-hosted deployments have different operational, legal and product requirements.

---

## Store Binary Media Directly in the Domain Database

Rejected as a general architectural rule. Binary Artifacts should be handled through artifact storage while metadata remains part of the structured application model.

---

# Relationship to Existing Architecture

This decision refines ADR-026, which already establishes the Storage Provider Layer and provider independence.

It adds explicit boundaries for artifact storage, data ownership and retention policy.

It complements ADR-038 by defining where Raw and Derived Artifacts may be stored without coupling the domain model to a particular backend.

---

# Future Considerations

A later storage implementation ADR must define the concrete provider interface, atomicity guarantees, integrity verification, resumable writes, deletion semantics and provider capability model.
