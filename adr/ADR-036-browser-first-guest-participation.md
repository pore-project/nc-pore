# ADR-036: Browser-First Guest Participation

- Status: Proposed
- Date: 2026-08-17
- Decision Type: Architecture
- Supersedes: relevant parts of ADR-008

---

## Context

NC-PoRe must support reliable local recording while keeping participation by external guests as simple as possible.

Requiring a separate client for Windows, macOS, Linux, iOS and Android would create installation and maintenance barriers, especially for occasional participants.

The Ennuicastr architecture demonstrates that browser-based participation can cover a substantial part of the remote recording use case without requiring a platform-specific guest application.

At the same time, professional recording may require capabilities that browsers cannot reliably provide on every platform.

## Decision

NC-PoRe adopts a **browser-first participation model**.

External guests should be able to join and record through a supported modern browser without installing a dedicated NC-PoRe application.

A native or specialized recorder may exist for professional workflows where browser capabilities are insufficient, but it is not a prerequisite for ordinary guest participation.

The browser client is responsible for capture and session interaction. It does not define the canonical production or storage format.

## Architectural Principle

> Participation should require a browser, not a platform-specific NC-PoRe client, wherever technically feasible.

The server and domain model must remain independent of a specific browser implementation.

## Scope Boundary

Browser-first does not mean browser-only.

Professional clients may later provide additional capabilities such as advanced hardware control, specialist monitoring, offline workflows or professional audio interfaces.

Such clients are optional extensions and must not become a dependency for guest participation.

## Consequences

### Positive

- very low entry barrier for guests
- no multi-platform guest-client distribution requirement
- simpler onboarding and updates
- broad device coverage through established browser platforms
- clear separation between capture client and production model

### Negative

- browser behavior, permissions and scheduling must be treated as explicit technical constraints
- background execution, device changes, buffer handling and browser lifecycle require dedicated testing
- some professional recording capabilities may require a specialized client

## Alternatives Considered

### Platform-specific guest clients

Rejected as the default approach because installation and maintenance would unnecessarily burden occasional participants.

### Browser-only for all recording scenarios

Rejected as an absolute requirement because professional audio workflows may need capabilities that cannot be guaranteed in browsers.

## Relationship to Existing Architecture

This decision refines ADR-008. The existing distinction between professional recording and simple guest participation remains valid, but the guest path is now explicitly browser-first rather than merely a possible future option.

ADR-026 remains authoritative for the separation between the domain model and storage providers.

## Future Considerations

The supported browser capabilities and minimum requirements must be defined before production use.

The browser capture model must be evaluated together with the synchronization and reconstructable-capture decisions.

---

# English Version

## Context

NC-PoRe must provide reliable local recording while keeping external guest participation as simple as possible.

Requiring separate clients for Windows, macOS, Linux, iOS and Android would create installation and maintenance barriers, especially for occasional participants.

Ennuicastr demonstrates that browser-based participation can cover a substantial part of remote recording without requiring a platform-specific guest application.

Professional recording may nevertheless require capabilities that browsers cannot reliably provide on every platform.

## Decision

NC-PoRe adopts a **browser-first participation model**.

External guests should be able to join and record through a supported modern browser without installing a dedicated NC-PoRe application.

A native or specialized recorder may exist for professional workflows where browser capabilities are insufficient, but it is not required for ordinary guest participation.

The browser client handles capture and session interaction. It does not define the canonical production or storage format.

## Principle

> Participation should require a browser, not a platform-specific NC-PoRe client, wherever technically feasible.

Browser-first does not mean browser-only. Professional clients remain possible as optional extensions.

## Consequences

- lower entry barrier for guests
- no platform-specific guest-client distribution requirement
- simpler onboarding and updates
- explicit need to test browser lifecycle, permissions, buffering and device changes
- specialized clients may still be required for advanced professional workflows
