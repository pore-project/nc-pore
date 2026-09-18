# Nextcloud Runtime Adapter V1

## Status

Implemented as the thin Nextcloud host boundary for V1 recording artifact transport.

The finalized-artifact transport is governed by ADR-083.

## V1 boundary

```text
Browser completion job
    -> prepare (authenticated OCS control request)
    -> temporary Nextcloud upload authorization
    -> browser PUT to temporary public WebDAV authorization
    -> verify (authenticated OCS control request)
    -> close temporary authorization
    -> authoritative recording completion
```

The browser already owns a fully finalized and durably preserved transfer artifact. V1 therefore does not introduce a second server-side recording artifact lifecycle.

The payload itself is transferred directly into the final Nextcloud Files location through a temporary, password-protected public upload authorization. The PoRe PHP application handles control-plane preparation, verification and cleanup; it does not receive the audio payload as a multipart upload.

## Browser control requests

The browser uses authenticated OCS requests for three control operations:

```text
POST /ocs/v2.php/apps/pore/v1/recordings/finalized-artifact/prepare
POST /ocs/v2.php/apps/pore/v1/recordings/finalized-artifact/verify
POST /ocs/v2.php/apps/pore/v1/recordings/finalized-artifact/close
```

`prepare` receives only metadata and integrity information:

- `core.ProductionId`
- `core.RecordingId`
- `recorder.RecordingSessionId`
- browser technical capture identity
- recording start time
- production/participant labels
- finalized payload size
- finalized payload SHA-256

It does **not** receive the finalized audio payload.

## Nextcloud transport authorization

During `prepare`, the Nextcloud connector:

1. resolves the production owner;
2. resolves the configured Files-relative storage root;
3. constructs the final artifact destination using `NextcloudArtifactPath`;
4. creates the required destination folders;
5. creates a temporary password-protected public upload share for the destination folder;
6. returns an opaque transport handle together with the temporary upload authorization.

The authorization contains a bounded public WebDAV endpoint, share credentials and the final filename. The browser does not construct the authoritative storage path.

Nextcloud's public WebDAV file endpoint is the transport data plane. Non-GET requests use the required `X-Requested-With: XMLHttpRequest` header. citeturn5search0

The temporary share is not the storage object. It only authorizes the one bounded upload operation and is closed after verification.

## Storage ownership and path contract

The host chooses a **storage root inside the production owner's Nextcloud Files tree**. This is a logical Files path, not a server filesystem path. A value such as:

`Büro/interviews`

means that PoRe starts directly below that location. PoRe must never receive, construct or use a path such as:

`/var/www/nextcloud/data/max/files/Büro/interviews/`

The latter is an implementation detail of the Nextcloud installation and remains outside the app contract.

The user-facing Talk settings section shows `audio` as the default placeholder. If no custom root is configured, PoRe uses `audio` at the owner's Files root. If the host configures a root, that configured path is the **complete PoRe root**; PoRe does not append `audio` to it.

PoRe then applies its standardized structure directly below the effective root:

```text
<effective PoRe root>/YYYY/MM/DD - HH:MIN <production label> - <core.ProductionId>/<participant>.wav
```

If no participant label is available, the technical capture identity remains the filename fallback.

## Connector responsibility

`NextcloudArtifactConnector` owns all Nextcloud-specific mechanics:

- production-owner resolution
- target folder resolution
- temporary public share creation
- share token/password handling
- public WebDAV upload authorization
- final artifact lookup
- remote size verification
- remote SHA-256 verification
- idempotent temporary-share cleanup

The connector does not expose Nextcloud concepts to Core or the provider-neutral recording model.

`NextcloudArtifactStorage` is obsolete and is no longer part of the transport path.

## Authoritative completion

A successful upload response alone is **never** considered sufficient for PoRe completion.

After the browser upload, the authenticated `verify` operation reads the destination file back through Nextcloud's Files API and checks:

1. the remote file exists;
2. its exact size equals the finalized payload size;
3. SHA-256 of the remote bytes equals the finalized payload SHA-256.

Only after all three checks succeed may the transport proceed to `close`.

`close` removes the temporary upload share. Closing is idempotent: an already expired or already removed share is considered closed.

Only after successful verification **and** successful transport close does the browser completion job become `completed` and trigger authoritative recording completion.

The local preservation artifact remains available until that point.

## Durable recovery

The browser completion job persists transport state:

```text
prepared
  -> authorized
  -> remote_present
  -> verified
  -> transport_closed
  -> completed
```

A failure retains the local finalized capture and the durable completion-job state. Recovery retries the outstanding step rather than declaring success merely because a previous HTTP request returned successfully.

## Configuration

The storage-root setting is stored per authenticated Nextcloud user under the app configuration key `storage_root`. An empty stored value means that PoRe uses the default root `audio`.

The setting endpoint accepts only a relative Files path. Absolute server paths and traversal components are rejected. The user-facing setting therefore never exposes or requests the Nextcloud server's physical data-directory path.

## Explicit non-goals

- No browser-to-PoRe multipart payload upload
- No second authoritative persistence path
- No direct access to Nextcloud's private data directory
- No recording transport through Nextcloud Talk recording
- No identity aliasing
- No HTTP-success-as-completion shortcut
- No provider-specific Nextcloud concepts in Core

## Architectural consequence

Nextcloud is the sole V1 transport provider and remains authoritative for final file storage. PoRe owns capture finalization, durable browser preservation, artifact identity and the integrity proof required before completion.
