# PoRE Runtime Boundary — V1

## Purpose

This document defines the smallest runtime boundary required to connect a host adapter to the existing PoRE Application path.

## Invariant

The runtime is host-neutral. It must not import, name, or depend on Nextcloud, Nextcloud Talk, or another host platform.

The host-specific side is responsible for authentication, request handling, provider-specific mapping, and transport into this boundary.

## V1 operation

The only V1 operation is:

`recording.submit_finalized_artifact`

It accepts:

- protocol version
- request identity
- browser capture identity
- recorder technical session identity
- PoRE production identity
- PoRE recording identity
- technical track identity
- audio configuration
- finalized payload length
- finalized payload bytes

The runtime returns:

- protocol version
- request identity
- processing status
- resulting artifact identity when stored
- a stable error code when rejected or failed

## Framing

The reference runtime uses a simple length-prefixed stdin/stdout framing:

```text
request:
  u32-be header length
  JSON header
  raw payload

response:
  u32-be JSON length
  JSON response
```

The framing is deliberately not an HTTP API. It is a local process boundary and can be replaced later without changing the host-neutral application contract.

## Processing path

```text
host adapter
    |
    | recording.submit_finalized_artifact
    v
PoRE Runtime
    |
    v
BrowserRecordingArtifact
    |
    v
RecordingArtifactProcessor
    |
    v
PersistenceProvider
```

The runtime does not create a second persistence path or a second synchronization lifecycle.

## Explicit non-goals

V1 does not define:

- a PoRE HTTP server
- a Nextcloud-specific runtime API
- a Talk-specific runtime API
- a long-running daemon requirement
- a particular process supervisor
- a particular deployment model
- future host protocols
- future transport protocols

Those are separate decisions if a concrete need arises.
