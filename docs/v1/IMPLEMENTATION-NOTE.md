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

## V1 connector boundary

The production connector clones the current `TrackEnabler` output and exposes that PoRE-owned clone through a neutral browser event. Talk retains ownership of its original track.

Connector boundary tests cover:

- initial TrackEnabler attachment and cloning,
- repeated attachment without duplicate cloning,
- TrackEnabler output replacement and release of the previous PoRE clone,
- source-track termination,
- clean connector detachment,
- disposal of the PoRE-owned clone.

## V1 UI boundary

Host controls are injected only when Talk exposes the host-level action for ending the meeting for everyone. The controls are explicitly **Aufnahme starten** and **Aufnahme beenden**; during an active recording the state is shown as **Aufnahme läuft**.

`Aufnahme beenden` stops the PoRE recording controller only. It does not call Talk's `webrtc.stop()` and does not end the Talk room.

If Talk replaces the active source track while a recording is running, V1 finalizes the current recording with the explicit reason `talk-track-replaced` rather than continuing silently from a stale source.

## V1 boundary

V1 deliberately does not make room termination the primary recording control. It also does not require modifying Talk's WebRTC implementation itself. The connector consumes the exposed media pipeline boundary and owns only the host-integration side; the generic recording controller remains separate from Talk-specific logic.
