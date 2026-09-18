# ADR-083: Verified Nextcloud Artifact Transport

- Status: Accepted
- Date: 2026-09-14
- Supersedes: ADR-082 for the finalized-artifact transport mechanism

## Context

The browser already owns a finalized recording capture that has been durably preserved locally. The former browser transport uploaded the complete payload as `multipart/form-data` to a PoRe PHP controller, which then wrote the payload directly through the Nextcloud Files API.

That created a second storage path beside the intended transport abstraction and made an HTTP success response too close to the semantic meaning of completion.

The transport must instead have one authoritative Nextcloud path with an explicit lifecycle:

```text
local Recorder artifact
    -> securely persisted
    -> transport artifact
    -> temporary Nextcloud upload authorization
    -> browser uploads directly to that authorization
    -> PoRe verifies remote size and SHA-256
    -> PoRe closes the temporary authorization
    -> local preservation and completion state may be cleaned up
```

## Decision

Nextcloud is the concrete transport provider for V1.

The browser does not upload the payload to a PoRe multipart endpoint. The PoRe application boundary is used only for transport control-plane operations:

1. `prepare` resolves the production owner, constructs the final Files-relative destination, creates the destination folder and creates a temporary password-protected public upload share.
2. `transfer` uploads the finalized payload to the bounded public WebDAV authorization returned by `prepare`.
3. `verify` resolves the prepared destination server-side and verifies both exact byte count and SHA-256 of the file read back through Nextcloud Files.
4. `close` removes the temporary share. Closing is idempotent; an already expired or removed share is treated as closed.

The temporary share is an authorization mechanism, not an additional storage object or lifecycle. The uploaded file remains in the final authoritative Nextcloud Files location after the share is closed.

## Integrity rule

`HTTP 2xx`, `201`, `204`, or an equivalent WebDAV success response never means that a recording is complete.

Completion is allowed only after:

```text
remote artifact present
    AND
remote size == expected size
    AND
remote SHA-256 == expected SHA-256
    AND
temporary transport authorization closed
```

If verification fails, local preservation remains available for retry/recovery. If closing fails after successful verification, the recording remains non-completed and the close operation may be retried. No verified transport artifact is deleted as part of transport close.

## Durable browser lifecycle

The browser completion job persists transport state and can resume from the last durable state:

```text
prepared
  -> authorized
  -> remote_present
  -> verified
  -> transport_closed
  -> completed
```

A failure does not erase the preservation artifact. Recovery retries the appropriate outstanding transport step.

## Relationship to fachlicher Recording and Production completion

`completed` in the browser transport lifecycle means **verified transport completion of one Artifact**. It does not by itself mean `core.Recording.Completed` and never means `core.Production.Completed`.

The verified transport completion contributes to Recording completion only when every expected participant artifact of that Recording has been confirmed as defined by ADR-084.

If a Production has already been closed through normal completion, timeout, or Host Force-Close, an outstanding Artifact remains valid and may still complete later. A late client may call `prepare` again and receive fresh temporary upload authorization. Expiration of an earlier share, token, URL, or other transport handle does not invalidate the stable Artifact identity.

Late Artifact Completion must not reopen the Production.

## Responsibility boundary

The provider-neutral PoRe layers know only that an artifact is being transferred and that a remote receipt must be verified.

Nextcloud-specific concepts remain in the Nextcloud connector:

- production-owner resolution
- Nextcloud Files path construction
- public share creation
- share token and password
- public WebDAV URL
- Nextcloud file lookup
- remote size/hash verification
- share deletion

Core and provider-neutral recording code must not acquire knowledge of these concepts.

## Identity and naming

The existing identities remain distinct:

```text
core.ProductionId
core.RecordingId
recorder.RecordingSessionId
artifact.ArtifactId
```

A transport handle is not an additional recording identity. It is an opaque provider authorization/continuation value. Architecturally relevant names follow the qualification rules of internal naming ADR `082i`.

## Path and storage contract

The existing `NextcloudArtifactPath` contract remains authoritative for the final Files-relative destination. The connector may create the destination folder during preparation, but it must not access Nextcloud's private server filesystem.

The browser receives only the bounded upload authorization and the final filename. It does not construct or own the authoritative Nextcloud storage path.

The PoRE application authorizes prepare against the authenticated Nextcloud user and the authoritative recording state. Only a recording host or participant may prepare transport. The transport handle is additionally bound to the authenticated user that prepared it; verify and close reject use by another user. Provider mechanics remain inside the Nextcloud connector.

## Collision and concurrency rule

Preparation treats the path and filename as a content-addressed target candidate:

1. If the intended filename already exists and its exact byte size and SHA-256 match the prepared artifact, the existing file is reused and no upload authorization is created.
2. If the intended filename exists with different content, the connector selects the first free numeric suffix: `name (2).ext`, then `name (3).ext`, and so on. Existing files are never overwritten by PoRE transport.
3. The temporary public share grants create permission only. The browser also sends `If-None-Match: *` on the final PUT. A race that makes the selected filename unavailable is treated as a collision: the temporary authorization is closed, preparation is repeated, and the next free target is selected.
4. Reused and uploaded artifacts both pass through the same server-side size + SHA-256 verification and transport-close lifecycle before completion.

## Consequences

- There is one artifact transport path to Nextcloud.
- The obsolete browser -> PoRe PHP multipart -> Nextcloud Files payload path is removed.
- Local durable preservation remains the recovery source until verified transport completion.
- Nextcloud-specific mechanics are isolated in one connector.
- A future transport provider can implement the same provider-neutral lifecycle without introducing Nextcloud concepts into Core.
- Transport completion remains distinct from fachliche Recording and Production completion.
- Production timeout/Force-Close is not an Artifact retention or expiration policy.
- The historical ADR-082 remains valuable as architectural history, but its prohibition of browser-to-WebDAV transport no longer describes the accepted V1 mechanism and is therefore superseded for this finalized-artifact transport.
