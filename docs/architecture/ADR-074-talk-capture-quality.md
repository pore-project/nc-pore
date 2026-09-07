# ADR-074: Talk Capture Quality Boundary

- Status: Accepted
- Date: 2026-08-30
- Scope: V1 Talk integration

## Context

NC-PoRe is intended for professional podcast production and for recording material that may be delivered to broadcasters or other downstream users. The recording path therefore aims to preserve the input signal as faithfully as technically possible, preferably lossless, rather than merely producing speech that is intelligible in a call.

Nextcloud Talk is a communication layer. Its WebRTC audio path may apply browser/WebRTC audio processing and transports the call audio using a lossy codec such as Opus. A Talk-provided `MediaStreamTrack` therefore cannot be treated as an equivalent substitute for a native PoRE capture path: codec conversion cannot restore information already discarded upstream, and the quality of the microphone and capture environment remains a limiting factor.

The V1 prototype demonstrated that PoRE can capture the Talk-provided audio track, producing a valid 48 kHz stereo Opus stream. The resulting file was decodable, but its quality characteristics are those of the Talk/WebRTC path rather than those of a lossless PoRE master recording.

## Decision

PoRE separates communication from recording.

- Talk remains responsible for the communication stream.
- PoRE must not use the Talk/WebRTC encoded output as its professional recording master when a direct local capture path is available.
- PoRE should capture the local input device independently, before Talk's WebRTC processing and codec path, using the best quality actually supported by the selected device.
- Native PoRE capture remains the reference/master recording path and should preserve the input signal as faithfully as technically possible, preferably losslessly.
- Talk capture may remain available as a compatibility/convenience path, but must not be represented as equivalent in quality to native PoRE capture.

## Device capabilities

Before recording, PoRE should determine the capabilities of the selected input device rather than assuming a fixed recording format. The capture configuration should use the highest appropriate quality that the device actually supports.

PoRE must not manufacture quality characteristics that the source device does not provide. For example, a mono source must not be represented as a genuinely stereo source merely by duplicating channels.

## Device changes during recording

Input-device changes are part of the recording lifecycle and must be handled explicitly.

A change before recording starts simply selects the new input device for the recording.

A change during an active recording must not silently replace the source while pretending that the complete recording came from one unchanged device. The change should create a technical capture boundary so that the resulting RecordingArtifact can represent the source transition and preserve provenance.

The existing RecordingArtifact model, with tracks/chunks and sample offsets, is the intended place to represent such capture continuity and boundaries.

## Consequences

This decision deliberately accepts that a remote Talk participant may not provide the same technical quality as a locally captured professional microphone. PoRE can preserve the best signal available from that participant's actual input device, but cannot reconstruct information lost by poor hardware, room acoustics, browser processing, or lossy network encoding.

The decision also avoids adding a post-hoc WebM/Opus-to-WAV conversion merely to make a lossy Talk stream look like a lossless master. Such conversion changes the container/representation, not the underlying information content.

## Non-goals

- Replacing Talk's communication codec.
- Treating WebRTC/Opus as a lossless recording format.
- Promising broadcast-grade quality regardless of the participant's microphone or recording environment.
