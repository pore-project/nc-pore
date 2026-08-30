/*
 * NC-PoRE — Talk audio integration.
 *
 * The Talk-specific connector attaches to Talk's TrackEnabler output rather
 * than intercepting getUserMedia().
 */

(() => {
	'use strict'

	console.log('PoRE: ADR073 CONNECTOR INIT')

	const Connector = window.PoRETalkAudioCaptureConnector

	if (!Connector) {
		return
	}

	const connector = new Connector()
	window.__poreTalkAudioConnector = connector

	const tryAttach = () => {
		if (connector.attachToTalk()) {
			console.log('PoRE: ADR073 TrackEnabler sink attached')
			return
		}

		window.setTimeout(tryAttach, 100)
	}

	tryAttach()
})()