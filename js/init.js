/*
 * NC-PoRE — Talk recording UI bootstrap.
 *
 * Talk supplies the mount point and role/context. This module only renders
 * PoRE's recording surface and forwards explicit user intent.
 */

(() => {
	'use strict'

	const Connector = window.PoRETalkAudioCaptureConnector
	const Ui = window.PoRETalkRecordingUi

	if (!Connector || !Ui) return

	const connector = new Connector()
	window.__poreTalkAudioConnector = connector

	const render = context => {
		if (!context) return
		Ui.mount(context)
	}

	window.addEventListener('pore:recording-ui-context', event => render(event.detail))

	const tryAttach = () => {
		if (connector.attachToTalk()) return
		window.setTimeout(tryAttach, 100)
	}

	tryAttach()
})()
