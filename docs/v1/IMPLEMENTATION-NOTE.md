# PoRE V1 implementation note

> Development note for the first Talk integration milestone.

## Verified runtime path

Nextcloud Talk exposes the live microphone as the `audio` output track of its `MediaDevicesSource`. The verified runtime showed the RODECaster track as a live `MediaStreamTrack` and confirmed that a `MediaStream` built from that track is usable by `MediaRecorder` with `audio/webm;codecs=opus`.

The Talk media pipeline connects `MediaDevicesSource` to the audio `TrackEnabler`. The PoRE connector therefore attaches at the TrackEnabler output boundary rather than reaching into browser device selection or replacing Talk's capture implementation.

## Recording lifecycle

The recording lifecycle is independent of the Talk room lifecycle. The host action **Stop recording** is the recording stop event. Leaving/ending the Talk room is not the normal recording stop trigger; room teardown is only a later cleanup/failure boundary.

V1 therefore follows:

`host start recording -> capture -> host stop recording -> MediaRecorder.stop() -> final chunk -> finalize session`

## Runtime evidence

- `MediaDevicesSource.getOutputTrack('audio')` returned a live audio track.
- The track was also present in the corresponding active `MediaStream`.
- `MediaRecorder` accepted that stream with `audio/webm;codecs=opus`.
- A short test produced multiple `dataavailable` chunks and a final Blob.
- Track identity remained stable across the tested path.

These observations are implementation evidence, not an API guarantee of future Talk versions. They must be revalidated when the Talk/Spreed runtime changes.

## V1 boundary

V1 deliberately does not make room termination the primary recording control. It also does not require modifying Talk's WebRTC implementation itself. The connector consumes the exposed media pipeline boundary and owns the PoRE recording lifecycle.
