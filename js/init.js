/*
 * NC-PoRe — Talk recording UI bootstrap.
 *
 * Talk supplies the mount point and role/context. This module bridges the
 * Talk audio track into the neutral browser recorder and keeps the UI driven
 * by explicit recording lifecycle events.
 */

(() => {
	'use strict'

	const Connector = window.PoRETalkAudioCaptureConnector
	const Recorder = window.PoREBrowserRecordingController
	const Ui = window.PoRETalkRecordingUi

	if (!Connector || !Recorder || !Ui) return

	const connector = new Connector()
	const recorder = new Recorder()
	window.__poreTalkAudioConnector = connector
	window.__poreTalkRecordingController = recorder

	let context = null
	let sourceTrack = null

	const startRequested = () => window.dispatchEvent(new CustomEvent('pore:recording-ui-start-local'))
	const stopRequested = () => window.dispatchEvent(new CustomEvent('pore:recording-ui-stop-local', { detail: { reason: 'host' } }))

	const render = nextContext => {
		if (!nextContext) return
		context = {
			...nextContext,
			onStart: nextContext.onStart || startRequested,
			onStop: nextContext.onStop || stopRequested,
		}
		Ui.mount(context)
	}

	const publish = patch => {
		if (!context) return
		context = { ...context, ...patch }
		Ui.mount(context)
	}

	const startLocalCapture = async () => {
		if (!sourceTrack) throw new Error('Talk audio track is not available')
		await recorder.start(sourceTrack, context?.sourceMetadata || {})
		publish({ state: 'recording', ready: true })
	}

	const stopLocalCapture = async reason => {
		const artifact = await recorder.stop(reason)
		publish({ state: 'stopping', ready: false, artifact })
		return artifact
	}

	window.addEventListener('pore:talk-audio-track', event => {
		if (sourceTrack && sourceTrack !== event.detail?.track && recorder.isRecording()) {
			recorder.noteSourceChange(sourceTrack, event.detail?.track)
		}
		sourceTrack = event.detail?.track || null
		if (sourceTrack) publish({ localCaptureAvailable: true })
	})

	window.addEventListener('pore:recording-started', event => {
		publish({
			state: 'recording',
			ready: true,
			startedAt: event.detail?.startedAt || event.detail?.source?.startedAt,
		})
	})

	window.addEventListener('pore:recording-finalized', event => {
		publish({ state: 'stopping', ready: false, artifact: event.detail })
	})

	window.addEventListener('pore:recording-error', event => {
		publish({ state: 'error', ready: false, error: event.detail?.error })
	})

	window.addEventListener('pore:recording-ui-context', event => render(event.detail))

	window.addEventListener('pore:recording-ui-start-local', async () => {
		try {
			await startLocalCapture()
		} catch (error) {
			publish({ state: 'error', ready: false, error })
		}
	})

	window.addEventListener('pore:recording-ui-stop-local', async event => {
		try {
			await stopLocalCapture(event.detail?.reason || 'host')
		} catch (error) {
			publish({ state: 'error', ready: false, error })
		}
	})

	const tryAttach = () => {
		if (connector.attachToTalk()) return
		window.setTimeout(tryAttach, 100)
	}

	tryAttach()
})()
