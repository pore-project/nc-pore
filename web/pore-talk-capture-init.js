/*
 * NC-PoRe — early Talk microphone capture hook
 *
 * This file is intentionally tiny. It is loaded before Nextcloud Talk's
 * normal client bundle and observes microphone MediaStreams at the browser
 * capture boundary. PoRE receives a clone of the captured audio track while
 * Talk keeps the original stream and its normal processing pipeline.
 *
 * No PoRE recording, signalling, or UI logic belongs here.
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
		if (!audioTrack) {
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
