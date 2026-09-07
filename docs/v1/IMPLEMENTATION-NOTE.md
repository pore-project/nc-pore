# PoRE V1 implementation note

> Development note for the first Talk integration milestone.

## Capture boundary

PoRE does **not** use the audio track that Talk sends through its communication pipeline as the recording source. Talk is a host integration point, not the PoRE recorder.

The connector observes Talk's current local audio source only to identify the selected microphone. PoRE then opens its own browser capture for that device. This keeps the PoRE capture path independent from Talk's communication processing, including noise suppression, echo cancellation, automatic gain control and communication codecs.

The PoRE capture request disables the browser communication-processing controls where they are exposed:

- `echoCancellation: false`
- `noiseSuppression: false`
- `autoGainControl: false`

No artificial sample rate, channel count or bit depth is requested. The actual track settings delivered by the browser are authoritative for the available source quality.

The resulting PoRE-owned `MediaStreamTrack` is the only track handed to the generic browser recording controller. Talk retains ownership of its own communication track.

## Talk integration boundary

The connector attaches to Talk's audio `TrackEnabler` only as a notification boundary for the currently selected microphone and for source replacement. It does not consume or clone the Talk communication track for recording.

When Talk replaces its active microphone track, the connector uses the new track only to identify the new device and opens a new independent PoRE capture. A stale asynchronous capture result is discarded if a newer source has already superseded it.

If the independent PoRE capture cannot be opened, the connector does not silently fall back to recording Talk's processed communication track. The V1 UI reports the failure and does not present the failed source as ready for recording.

## Recording lifecycle

The recording lifecycle is independent of the Talk room lifecycle. The host action **Aufnahme beenden** is the recording stop event. Leaving/ending the Talk room is not the normal recording stop trigger; room teardown is only a later cleanup/failure boundary.

V1 therefore follows:

`host start recording -> independent local capture -> host stop recording -> recorder finalization`

The browser recording controller remains generic and knows nothing about Talk.

## Recording representation

The browser controller currently uses the browser's `MediaRecorder` capability for the V1 participation path. Its output is a browser recording `Blob`, not a persisted PoRE `RecordingArtifact`.

The browser path must therefore cross an explicit application/artifact boundary before it can enter the authoritative PoRE persistence and synchronization lifecycle. No UI or connector code may label a browser `Blob` as already persisted.

## Runtime evidence

The first Talk reality check established that Talk obtains a live microphone `MediaStreamTrack` through its `MediaDevicesSource` and that the track can be observed at the Talk media-pipeline boundary. That evidence is used for device/source identification only.

It does not establish that the Talk communication track is suitable as the PoRE master recording source. The independent-capture boundary is the architectural requirement.

These observations are implementation evidence, not an API guarantee of future Talk versions. They must be revalidated when the Talk/Spreed runtime changes.

## V1 UI boundary

Host controls are injected only when the Talk host integration is available. The controls are explicitly **Aufnahme starten** and **Aufnahme beenden**. During an active recording the state is shown as **Lokale Aufnahme läuft**.

The UI identifies the microphone used by the independent PoRE capture and, when the browser exposes them, its actual sample rate, sample size and channel count. These are informational values from the capture track, not requested promises.

`Aufnahme beenden` stops the PoRE recording controller only. It does not call Talk's `webrtc.stop()` and does not end the Talk room.

If Talk replaces the active microphone while a recording is running, V1 treats the source replacement as a recording boundary rather than silently continuing with an unverified source.

## V1 boundary

V1 deliberately does not make room termination the primary recording control and does not make Talk's processed communication audio the PoRE master source. The connector is responsible only for the Talk-specific source-selection boundary; the generic recording controller remains separate from Talk.

The next integration boundary is the existing PoRE recording/artifact path. A browser-produced recording must cross that boundary explicitly; a browser `Blob` must not be presented as an already-persisted `RecordingArtifact`.
