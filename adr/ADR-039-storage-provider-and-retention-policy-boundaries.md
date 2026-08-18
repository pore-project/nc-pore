# ADR-039: Storage Provider and Retention Policy Boundaries

- Status: Proposed
- Date: 2026-08-17
- Decision Type: Architecture

---

## Context

NC-PoRe is self-hosted and users/operators should retain control over their production data.

The technical storage mechanism must nevertheless remain replaceable. Local filesystem storage, Nextcloud-backed storage, S3-compatible storage, WebDAV and future providers are all plausible implementations.

The comparison with Ennuicastr also raises retention as a useful concept, but retention policy must not become an implicit domain rule for a Recording.

## Decision

NC-PoRe keeps the **Core independent of concrete storage providers**.

The Core and application layer operate on artifact and storage capabilities rather than provider-specific APIs.

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

Nextcloud remains an important integration/provider in the NC-PoRe ecosystem, but the domain model must not depend on Nextcloud's internal storage representation.

## Architectural Principle

> The Core knows what must be stored; the provider knows how and where it is stored.

Storage providers must not become domain authorities.

## Data Ownership

Self-hosted storage is the default product model.

Optional external storage or processing services may be added later as explicit provider/add-on capabilities. Such services must not silently move production data away from the operator's chosen storage boundary.

## Retention Policy

Retention is treated as a **storage policy**, not as an intrinsic Recording lifecycle rule.

A deployment may later define policies such as:

- retain raw capture for a defined period
- retain derived production artifacts permanently
- remove failed/incomplete artifacts after a defined period
- retain everything indefinitely

The domain model must not assume a universal expiry period.

## Format Independence

Storage providers store artifacts as opaque payloads plus the metadata required by the artifact abstraction.

A provider does not need to understand whether an artifact contains WAV, FLAC, Opus, WebM or another format.

This keeps format decisions in capture, reconstruction and processing layers rather than in storage implementations.

## Consequences

### Positive

- storage technologies remain replaceable
- self-hosting remains the default
- external storage can be added without changing the domain model
- retention policy can evolve independently of domain lifecycles
- storage does not become coupled to media formats

### Negative

- provider abstraction introduces additional interfaces
- provider-specific capabilities may require explicit capability negotiation
- retention and deletion require careful policy and audit handling

## Alternatives Considered

### Direct provider access from the Core

Rejected. This would couple domain logic to infrastructure technology and make alternative providers harder to implement.

### Mandatory universal retention period

Rejected. Self-hosted deployments have different operational, legal and product requirements.

### Store binary media inside the domain database

Rejected as a general architectural rule. Binary artifacts should be handled through artifact storage while metadata remains part of the structured application model.

## Relationship to Existing Architecture

This decision refines ADR-026, which already establishes the Storage Provider Layer and provider independence.

It adds explicit boundaries for artifact storage, data ownership and retention policy.

It complements ADR-038 by defining where raw and derived artifacts may be stored without coupling the domain model to a particular backend.

## Future Considerations

A later storage implementation ADR must define the concrete provider interface, atomicity guarantees, integrity verification, resumable writes, deletion semantics and provider capability model.

---

# English Version

## Context

NC-PoRe is self-hosted and users/operators should retain control over their production data.

Storage must nevertheless remain replaceable. Local filesystem storage, Nextcloud-backed storage, S3-compatible storage, WebDAV and future providers are plausible implementations.

Retention is useful as a concept but must not become an implicit domain rule.

## Decision

NC-PoRe keeps the **Core independent of concrete storage providers**.

Core and application code use artifact and storage capabilities rather than provider-specific APIs.

Concrete providers sit behind a storage-provider boundary.

## Principle

> The Core knows what must be stored; the provider knows how and where it is stored.

## Data Ownership

Self-hosted storage is the default product model. Optional external providers may be added later as explicit add-ons and must not silently move production data outside the chosen storage boundary.

## Retention

Retention is a storage policy, not an intrinsic Recording lifecycle rule. Deployments may choose their own retention behavior.

## Format Independence

Storage providers treat artifact payloads as opaque with respect to media format.

## Consequences

- replaceable storage technologies
- self-hosting remains the default
- external storage can be added independently
- retention evolves separately from domain lifecycle
- storage remains independent of media formats
- additional provider interfaces and capability handling are required

## Future Work

A later ADR must define provider interfaces, atomicity, integrity, resumable writes, deletion semantics and provider capabilities.
