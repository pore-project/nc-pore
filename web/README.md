# NC-PoRe browser integration

The production Nextcloud Talk connector lives in `js/pore-talk-audio-connector.js`.

The connector attaches to Talk's current audio `TrackEnabler` output, clones the
current source track and exposes the clone through the `pore:talk-audio-track`
event. Talk retains ownership of its original track and media lifecycle.

`pore-talk-capture-init.test.js` contains the browser-side connector boundary tests.
The test suite deliberately exercises the TrackEnabler integration point rather
than the older global `getUserMedia()` proof-of-concept hook.

The generic browser recording lifecycle is implemented separately in
`js/pore-recording-controller.js` and knows nothing about Talk.
