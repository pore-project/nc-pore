# Nextcloud Runtime Adapter V1

## Status

Implemented as the thin normal Nextcloud app boundary for V1 recording artifact delivery.

## V1 boundary

```text
Browser completion
    -> authenticated Nextcloud OCS endpoint
    -> RecordingTransportController
    -> Nextcloud Files API
    -> authoritative stored artifact
```

The browser already owns a fully finalized and durably persisted transfer artifact. V1 therefore does not introduce a second PoRE synchronization queue, sync worker, or remote-transfer lifecycle. Nextcloud is the host and owns the authoritative file storage.

The host-neutral PoRE Runtime remains a separate boundary for PoRE-owned processing and future host variants; it is not required to duplicate the authoritative Nextcloud file in V1.

## Browser request

The browser sends a `multipart/form-data` POST to the normal authenticated Nextcloud OCS endpoint:

`/ocs/v2.php/apps/pore/v1/recordings/finalized-artifact`

The request contains:

- `metadata`: JSON metadata with authoritative and technical identities, recording start time, production label and audio format facts
- `payload`: the finalized WAV blob

The browser sets `OCS-APIRequest: true` and uses the existing Nextcloud session. No second authentication mechanism is introduced.

## Storage ownership and path contract

The host chooses a **storage root inside the current user's Nextcloud Files tree**. This is a logical Files path, not a server filesystem path. A value such as:

`Büro/interviews`

means that PoRE starts directly below that location through Nextcloud's Files API. PoRE must never receive, construct or use a path such as:

`/var/www/nextcloud/data/max/files/Büro/interviews/`

The latter is an implementation detail of the Nextcloud installation and is deliberately outside the app contract. In particular, the app must not use direct filesystem access to bypass Nextcloud's permission model.

The user-facing Talk settings section shows `audio` as the default placeholder. If no custom root is configured, PoRE uses `audio` at the user's Files root. If the host configures a root, that configured path is the **complete PoRE root**; PoRE does not append `audio` to it.

PoRE then applies its standardized structure directly below the effective root:

```text
<effective PoRe root>/YYYY/MM/DD - HH:MIN <production label> - <core.ProductionId>/<captureId>.wav
```

Therefore an explicitly configured root produces, for example:

```text
Büro/interviews/2026/09/05 - 15:42 Interview mit Max Muster - <core.ProductionId>/<captureId>.wav
```

and the default produces:

```text
audio/2026/09/05 - 15:42 Interview mit Max Muster - <core.ProductionId>/<captureId>.wav
```

The responsibility split is intentional:

1. **Host:** chooses the base location in its own Nextcloud Files namespace.
2. **PoRE:** constructs the standardized organization below that base.
3. **Nextcloud:** creates/writes the file and remains authoritative for storage and permissions.

The complete returned `path` is a **user-Files-relative Nextcloud path**, never a server data-directory path.

## Talk settings integration

PoRE registers a custom `NC-PoRE` section in the existing Nextcloud Talk settings dialog through Talk's `OCA.Talk.Settings` extension point. This keeps PoRE configuration where Talk users already expect Talk-related settings to live and avoids introducing a separate PoRE settings page.

V1 exposes the storage root as a plain relative path field. The field is empty when no custom root has been saved, so the visible `audio` value is the default placeholder rather than a value that gets concatenated to configured paths. Entering `Büro/interviews` therefore makes `Büro/interviews` the complete PoRE root.

No folder picker is required for V1. This keeps the UI aligned with the existing settings convention and avoids coupling PoRE's storage contract to a second path-selection abstraction.

## Nextcloud storage handoff

`RecordingTransportController` validates the request and obtains the uploaded temporary file through `IRequest::getUploadedFile()`.

`NextcloudArtifactStorage` obtains the authenticated user's Files root through Nextcloud's public `IRootFolder` / `IUserSession` APIs. It resolves the configured logical root below that user root, creates the required folders, and streams the temporary upload into the destination in bounded chunks. No direct access to Nextcloud's underlying data directory is used.

Path components are validated so neither artifact identity nor the host-configured root can escape the user's Files namespace.

## Authoritative completion

A successful HTTP/OCS response alone is not considered sufficient for PoRE completion.

Before storage, the server hashes the uploaded temporary payload and checks it against the browser's SHA-256. After writing, the adapter verifies both:

1. the stored Nextcloud file size equals the finalized payload size;
2. SHA-256 of the file read back through the Nextcloud Files API equals SHA-256 of the finalized transfer payload.

Only after both checks succeed does the controller return `status: stored` together with the Nextcloud file id, user-relative path, size and SHA-256. The browser then marks the durable completion job as completed.

This gives PoRE the concrete host-side acknowledgement it needs: the artifact exists in Nextcloud Files and is bit-for-bit identical to the finalized transfer artifact.

## Configuration

The storage-root setting is stored per authenticated Nextcloud user under the app configuration key `storage_root`. An empty stored value means that PoRE uses the default root `audio`.

The setting endpoint accepts only a relative Files path. Absolute server paths and traversal components are rejected. The user-facing setting therefore never exposes or requests the Nextcloud server's physical data-directory path.

## Explicit non-goals

- No AppAPI dependency
- No ExApp
- No standalone PoRe HTTP server for V1
- No direct browser WebDAV dependency
- No PoRE synchronization work queue for V1
- No V1 sync worker
- No second authoritative persistence path
- No identity aliasing
- No direct access to Nextcloud's private data directory

## Architectural consequence

Nextcloud owns the transport and storage responsibility that it already provides. PoRE owns the capture, durable browser preservation, artifact identity and integrity proof up to the host boundary. V1 completion is the successful, size-checked and hash-checked handoff into Nextcloud Files.
