# Browser-to-Application Artifact Transport

The browser recorder is a capture client. After local finalization it produces a durable persistence handoff containing authoritative production/recording identity plus distinct technical capture and recorder-session identities.

The finalized payload is not uploaded to a PoRe PHP multipart endpoint.

## Required runtime chain

```text
Browser capture
 -> durable local preservation
 -> completion job
 -> authenticated Nextcloud transport prepare
 -> temporary Nextcloud upload authorization
 -> browser PUT to bounded public WebDAV authorization
 -> authenticated remote verification
 -> idempotent transport close
 -> authoritative recording completion
```

The PoRe application endpoints are control-plane operations. They receive metadata, transport handles and integrity information, but never the finalized audio payload.

## Responsibility boundary

The browser completion job owns durable transport state and recovery.

The Nextcloud connector owns:

- production-owner resolution
- final Files-relative destination construction
- temporary public upload authorization
- WebDAV endpoint and credentials
- remote file lookup
- exact size verification
- SHA-256 verification
- authorization cleanup

Core and provider-neutral recording code do not know these Nextcloud concepts.

## Completion criterion

A recording is not complete because a browser upload returns HTTP success.

Completion requires:

```text
remote artifact exists
AND remote size matches
AND remote SHA-256 matches
AND temporary authorization is closed
```

The local preservation artifact remains available until this condition is satisfied.

## Historical boundary

The former direct browser -> PoRe PHP multipart -> Nextcloud Files path is obsolete and has been removed. The historical ADR-082 prohibition of browser WebDAV transport is superseded for this concrete V1 transport by ADR-083; the provider boundary and separation of Core from Nextcloud mechanics remain mandatory.
