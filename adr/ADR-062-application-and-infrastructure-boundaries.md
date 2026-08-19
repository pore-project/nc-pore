# ADR-062: Application and Infrastructure Boundaries

**Status:** Accepted  
**Date:** 2026-08-19

## Context

The production-management implementation had accumulated two architectural problems:

1. Production-session use-case orchestration was exposed from `core::api`, even though repository-backed orchestration belongs to the application layer.
2. The concrete filesystem implementation of `ProductionSessionRepository` lived in the application crate, even though filesystem persistence is infrastructure.

This made the dependency direction less explicit and allowed application code to acquire concrete storage concerns.

## Decision

NC-PoRe separates the layers as follows:

```text
Core
  ├── Domain model and invariants
  └── Repository contracts / reconstitution boundaries

Application
  ├── Production-management use cases
  └── Workflow orchestration across Core and technical boundaries

Infrastructure
  └── Concrete storage implementations
```

The production-management API is therefore part of `nc-pore-application`, not Core.

`ProductionSessionRepository` remains a Core-owned contract. `FileProductionSessionRepository` is implemented by `nc-pore-infrastructure`.

Application code must not contain concrete filesystem persistence implementations or direct filesystem serialization for ProductionSession state.

## Consequences

### Positive

- Core remains independent of application orchestration and storage technology.
- Application becomes the single public home for repository-backed production-management use cases.
- Infrastructure can evolve independently from application workflows.
- The dependency direction is explicit and testable.
- The architecture checks can enforce the application persistence boundary.

### Negative

- The workspace gains an infrastructure crate.
- Callers that previously imported production-management operations from `core::api` must use `nc-pore-application` instead.
- Some integration wiring becomes more explicit because concrete persistence types now enter at the infrastructure boundary.

## Alternatives Rejected

### Keep `core::api`

Rejected because repository-backed orchestration is application behavior, not domain behavior.

### Keep filesystem persistence in Application

Rejected because it violates the Persistence Boundary and makes the application layer responsible for a concrete storage technology.

### Put ProductionSession persistence into Recorder

Rejected because ProductionSession persistence is not a recorder responsibility. Recording artifact persistence and production-session persistence are distinct technical concerns.

## Validation

The architecture-check suite now includes an explicit Application boundary check preventing concrete filesystem persistence dependencies from reappearing in `application/src`.
