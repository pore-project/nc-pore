# NC-PoRe browser integration

## Early microphone capture hook

`pore-talk-capture-init.js` is deliberately limited to the browser capture boundary.

When an audio-capable `getUserMedia()` call succeeds, it creates a clone of the
captured microphone `MediaStreamTrack` and publishes that clone through the
`pore:microphone-clone` browser event. The original stream is returned unchanged
so Nextcloud Talk retains its normal media pipeline.

This is an integration spike for ADR-072. It is not yet the production loader,
recording implementation, or signalling implementation.
