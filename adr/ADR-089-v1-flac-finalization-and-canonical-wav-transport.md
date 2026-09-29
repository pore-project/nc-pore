# ADR-089: V1 FLAC Finalization and Canonical WAV Transport

## Status

Accepted

## Context

ADR-070 defines FLAC as the V1 default delivery format, while ADR-071 separates local capture/preservation from transport. ADR-083 requires that transport success is established only after the remote artifact has been verified and the temporary transport authorization has been closed.

The V1 browser capture is already a durable PCM24 mono preservation master. FLAC therefore must not replace the live capture/preservation path. It is a second, post-capture representation used for delivery.

## Decision

NC-PoRe V1 finalizes a durable local PCM preservation artifact to FLAC only after the recording has been stopped and the complete PCM capture has been durably persisted.

The finalization boundary is staged and crash-safe:

1. The durable PCM chunks remain the source of truth while FLAC encoding is in progress.
2. Encoded FLAC chunks are written to a separate durable IndexedDB store.
3. The staged FLAC is accepted only after its chunk continuity, per-chunk integrity, total size and SHA-256 have been verified.
4. Only then is the capture manifest switched atomically from PCM storage to FLAC storage and the original PCM chunks removed in the same transaction.
5. If finalization is interrupted before that commit, recovery may discard the incomplete FLAC staging area and encode again from the still-preserved PCM capture.

The browser transport payload for the V1 default path is audio/flac. The existing verified transport lifecycle remains unchanged: prepare -> upload -> verify -> close.

The Nextcloud connector verifies the uploaded FLAC payload by size and SHA-256 and then decodes it through the host-neutral PoRE Runtime. The decoded result must be canonical V1 PCM WAV: mono, 24-bit integer PCM, 48 kHz or the explicitly supported 44.1 kHz fallback. The canonical WAV is the durable remote artifact stored in Nextcloud Files under the artifact identity defined by ADR-069.

The uploaded FLAC is a temporary transport representation. It remains present until the transport handle is closed successfully, so repeated verification remains meaningful and idempotent. Closing the FLAC transport handle may then remove only that temporary FLAC file; the canonical WAV is not removed.

A previously verified artifact may be reused through its stable Nextcloud File-ID without re-uploading the local FLAC payload. Such reuse verifies the canonical WAV remote artifact and does not alter the established Artifact identity.

V1 supports only the preservation profile already defined by ADR-087: mono, 24-bit integer PCM, 48 kHz preferred and 44.1 kHz fallback. F32 and other capture/transport profiles are outside this decision.

## Consequences

- Live recording remains independent of the FLAC encoder and is protected by the existing durable PCM persistence boundary.
- Browser FLAC finalization is recoverable without trusting an incomplete compressed payload.
- The server remains the authority for the final remote WAV representation and can verify a real FLAC bitstream rather than trusting client-declared metadata alone.
- Artifact identity remains based on the canonical verified artifact record rather than on provider filename, File-ID or FLAC byte representation.
- FLAC transport and canonical WAV storage can coexist without conflating delivery format with preservation format.

## Testing Requirements

The V1 implementation must include:

- unit coverage for packed signed PCM24 conversion and encoder lifecycle;
- actual libFLAC browser-encoder execution against a small real capture payload;
- persistence tests covering staged FLAC integrity, atomic commit and recovery-safe incomplete staging;
- runtime tests that decode a real FLAC fixture and verify the resulting canonical WAV metadata and PCM payload;
- Nextcloud connector tests covering FLAC collision handling, canonical WAV creation/reuse, invalid FLAC profile rejection and temporary FLAC cleanup on close;
- transport/controller tests proving that the existing ADR-083 recovery and idempotency semantics remain intact.

## References

- ADR-069: Nextcloud Remote Artifact Storage V1
- ADR-070: Audio Delivery Format and FLAC Default
- ADR-071: Recording Capture, Preservation and Transport Formats
- ADR-083: Verified Nextcloud Artifact Transport
- ADR-084: Recording Artifact Aggregation and Completion Semantics
- ADR-085: Production Completion, Exceptional Closure and Late Artifact Delivery
- ADR-087: V1 Browser Capture Quality Boundary
