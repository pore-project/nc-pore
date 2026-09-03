/*
 * NC-PoRe — early Talk microphone capture hook
 *
 * Intentionally limited to the browser capture boundary. It observes
 * audio-capable getUserMedia() calls, clones the captured microphone track,
 * and publishes the clone for PoRE. The original stream is returned unchanged
 * so Nextcloud Talk keeps its normal media pipeline.
 */

(() => {
	'use strict';

	const mediaDevices = navigator.mediaDevices;
	if (!mediaDevices || typeof mediaDevices.getUserMedia !== 'function') {
		return;
	}

	const originalGetUserMedia = mediaDevices.getUserMedia.bind(mediaDevices);

	mediaDevices.getUserMedia = async function (constraints) {
		const stream = await originalGetUserMedia(constraints);

		if (!constraints?.audio) {
			return stream;
		}

		const audioTrack = stream.getAudioTracks()[0];
		if (!audioTrack || typeof audioTrack.clone !== 'function') {
			return stream;
		}

		const poreTrack = audioTrack.clone();
		window.dispatchEvent(new CustomEvent('pore:microphone-clone', {
			detail: {
				track: poreTrack,
				stream,
			},
		}));

		return stream;
	};
})();
