/*
 * NC-PoRE — early browser microphone capture integration.
 *
 * The Talk-specific lifecycle policy lives in the connector. This bootstrap
 * only installs the browser capture boundary early enough for Talk's normal
 * media pipeline to pass through unchanged.
 */

(() => {
	'use strict'

	const mediaDevices = navigator.mediaDevices
	const Connector = window.PoRETalkAudioCaptureConnector

	if (!mediaDevices || typeof mediaDevices.getUserMedia !== 'function' || !Connector) {
		return
	}

	if (mediaDevices.getUserMedia.__poreTalkConnectorInstalled) {
		return
	}

	const originalGetUserMedia = mediaDevices.getUserMedia.bind(mediaDevices)
	const connector = new Connector()

	const poreGetUserMedia = async function (constraints) {
		const stream = await originalGetUserMedia(constraints)
		connector.acceptStream(stream, constraints)
		return stream
	}

	poreGetUserMedia.__poreTalkConnectorInstalled = true
	mediaDevices.getUserMedia = poreGetUserMedia
	window.__poreTalkAudioConnector = connector
})()
