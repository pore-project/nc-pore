# NC-PoRe browser integration

The Nextcloud Talk integration is a device-selection observer. PoRE does **not**
record Talk's communication track: that path is allowed to apply communication
processing and encoding that are unsuitable as the PoRE recording source.

The connector observes Talk's selected microphone only to identify the device.
PoRE then opens its own `getUserMedia()` capture for that device with communication
processing disabled where the browser exposes those constraints. The resulting
track is owned by PoRE and is the recording source.

When Talk changes its selected microphone, the connector opens a new independent
PoRE capture for the new device and retires the previous PoRE capture. The Talk
track itself is never stopped or recorded by PoRE.

`pore-talk-capture-init.test.js` contains the browser-side connector boundary tests.
The generic browser recording lifecycle is implemented separately in
`js/pore-recording-controller.js` and knows nothing about Talk.
