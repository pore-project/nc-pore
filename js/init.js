/*
 * NC-PoRe — early browser microphone capture hook.
 *
 * This init script is intentionally independent of Talk internals. Nextcloud
 * loads it early, before normal app scripts, so the browser capture boundary
 * can be observed before Talk calls getUserMedia().
 */

(() => {
	'use strict';

	const mediaDevices = navigator.mediaDevices;
	if (!mediaDevices || typeof mediaDevices.getUserMedia !== 'function') {
		return;
	}

	if (mediaDevices.getUserMedia.__poreHookInstalled) {
		return;
	}

	const originalGetUserMedia = mediaDevices.getUserMedia.bind(mediaDevices);

	const poreGetUserMedia = async function (constraints) {
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

	poreGetUserMedia.__poreHookInstalled = true;
	mediaDevices.getUserMedia = poreGetUserMedia;
})();
