# ADR-071: Recording Capture, Preservation and Transport Formats

**Status:** Accepted

## Context

NC-PoRe is intended to remain platform- and host-open. A browser client must therefore not make Nextcloud Talk, or any other host platform, part of the NC-PoRe recording domain model. Platform-specific capabilities and constraints are supplied through connectors.

The capture environment may provide different audio capabilities. A browser, a native desktop client, and another host platform may expose different codecs, sample rates, channel layouts, sample formats, or container formats. At the same time, transport requirements may be imposed by the host or connector and do not necessarily need to match the best available local capture format.

The existing NC-PoRe model already represents recordings as tracks and chunks and records explicit recording configuration. Real CPAL capture, local persistence, synchronization and remote Nextcloud verification have been demonstrated successfully.

The next V1 step is a real browser client. Before implementing it, the boundary between capture, local preservation and transport must be explicit.

## Decision

NC-PoRe separates audio handling into three distinct format boundaries:

```text
CAPTURE
platform provides the best reliably available recording format
        ↓
PRESERVATION
NC-PoRe retains the highest-quality locally useful representation
        ↓
TRANSPORT
host/connector-specific representation used for synchronization
```

### 1. Capture format

The capture layer should use the **best recording format reliably available from the current capture environment**. NC-PoRe does not require every platform to produce the same capture format.

Capture capabilities are platform-specific and are exposed to the NC-PoRe recording boundary by the appropriate client/connector integration.

### 2. Preservation

The local recording should preserve the highest-quality representation that is practical and reliable for the capture environment. Preservation must not be degraded merely to satisfy a host's transport requirements.

The exact preservation representation remains an implementation-level decision where the capture platform imposes constraints. Lossless preservation is preferred whenever practical and supported.

### 3. Transport

Transport format is a separate concern from capture and preservation.

A host or connector may explicitly configure a transport representation, for example:

- MP3 at 64 kbit/s
- another explicitly supported compressed representation

When the host provides no transport-format requirement, **FLAC lossless compression is the NC-PoRe default transport representation**.

Conversion to the transport representation occurs at the transport boundary. A lossy transport conversion must not replace or degrade the preserved local recording.

### 4. Connector responsibility

A connector supplies platform-specific capabilities and constraints to the NC-PoRe client/application boundary. It may provide host configuration, including transport-format requirements, without making those requirements part of the domain Core.

Nextcloud/Talk is therefore an integration target, not the definition of the NC-PoRe recording format.

### 5. No audio "normalization" requirement

NC-PoRe does **not** require an early conversion of every capture into one canonical audio format merely to make the inputs uniform. In particular, "normalization" in this ADR does not mean loudness normalization and is not a required processing step.

If a later architectural boundary requires a specific technical representation for a concrete operation, that conversion must be explicit and justified at that boundary.

## Consequences

### Positive

- Browser and native capture clients can use the capabilities of their actual platform.
- Host transport requirements do not dictate local recording quality.
- Nextcloud/Talk integration remains connector-specific.
- Standalone and future host integrations can reuse the same NC-PoRe client/application contract.
- Lossless FLAC provides a sensible host-independent default transport representation.

### Costs

- The system must represent capture, preservation and transport as distinct concerns.
- Connectors must expose host configuration where transport requirements exist.
- Some transports require an explicit conversion step and associated CPU/storage cost.
- The exact browser preservation path still requires implementation decisions based on actual browser capture APIs.

## Open implementation questions

The following are deliberately deferred until the browser/client contract is designed:

- Which browser capture APIs and formats are supported in V1?
- Whether browser capture is preserved as raw PCM, a lossless encoded representation, or another representation before artifact creation.
- Which transport formats are supported in V1 beyond FLAC and explicitly configured examples such as MP3.
- How transport-format configuration is represented in the host/connector contract.
- Whether transport conversion is performed client-side, by the application boundary, or by a connector.
- How multi-track and per-track format constraints are represented.

These questions must not be resolved by making the browser client dependent on Nextcloud Talk.
