/*
 * NC-PoRE — Talk recording UI bootstrap.
 *
 * Talk supplies the mount point and role/context. Core/Application supplies the
 * authoritative recording state through the state bridge. Local recorder events
 * remain technical capture signals and never become a second lifecycle machine.
 */

(() => {
	'use strict'

	const Connector = window.PoRETalkAudioCaptureConnector
	const Recorder = window.PoREBrowserRecordingController
	const Ui = window.PoRETalkRecordingUi
	const StateBridge = window.PoRETalkRecordingStateBridge

	if (!Connector || !Recorder || !Ui || !StateBridge) return

	const connector = new Connector()
	const recorder = new Recorder()
	const stateBridge = new StateBridge()
	window.__poreTalkAudioConnector = connector
	window.__poreTalkRecordingController = recorder
	window.__poreTalkRecordingStateBridge = stateBridge

	let context = null
	let sourceTrack = null

	const startRequested = () => window.dispatchEvent(new CustomEvent('pore:recording-ui-start-local'))
	const stopRequested = () => window.dispatchEvent(new CustomEvent('pore:recording-ui-stop-local', { detail: { reason: 'host' } }))

	const render = nextContext => {
		if (!nextContext) return
		const authoritative = stateBridge.getSnapshot()
		context = {
			...nextContext,
			...(authoritative || {}),
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
		window.dispatchEvent(new CustomEvent('pore:recording-local-ready'))
	}

	const stopLocalCapture = async reason => {
		const artifact = await recorder.stop(reason)
		window.dispatchEvent(new CustomEvent('pore:recording-local-finalized', { detail: artifact }))
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
			startedAt: event.detail?.startedAt || event.detail?.source?.startedAt,
		})
	})

	window.addEventListener('pore:recording-finalized', event => {
		publish({ artifact: event.detail })
	})

	window.addEventListener('pore:recording-error', event => {
		publish({ localCaptureError: event.detail?.error })
	})

	window.addEventListener('pore:recording-state', event => {
		const snapshot = event.detail
		if (!snapshot) return
		stateBridge._snapshot = snapshot
		if (!context) return
		publish({
			role: snapshot.role,
			state: snapshot.state,
			listener: snapshot.listener,
			confirmed: snapshot.confirmed,
			ready: snapshot.ready,
			readyCount: snapshot.readyCount,
			participantCount: snapshot.participantCount,
			participants: snapshot.participants,
			elapsedSeconds: snapshot.elapsedSeconds,
			startedAt: snapshot.startedAt,
			error: snapshot.error,
		})
	})

	window.addEventListener('pore:recording-ui-context', event => render(event.detail))

	window.addEventListener('pore:recording-ui-start-local', async () => {
		try {
			await startLocalCapture()
		} catch (error) {
			window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } }))
		}
	})

	window.addEventListener('pore:recording-ui-stop-local', async event => {
		try {
			await stopLocalCapture(event.detail?.reason || 'host')
		} catch (error) {
			window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } }))
		}
	})

	const tryAttach = () => {
		if (connector.attachToTalk()) return
		window.setTimeout(tryAttach, 100)
	}

	tryAttach()
})()
