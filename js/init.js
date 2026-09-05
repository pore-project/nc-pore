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
	const CompletionJob = window.PoREBrowserCompletionJob
	const RuntimeTransport = window.PoREBrowserRuntimeTransport

	if (!Connector || !Recorder || !Ui || !StateBridge || !CompletionJob || !RuntimeTransport) return

	const connector = new Connector()
	const recorder = new Recorder()
	const stateBridge = new StateBridge()
	const persistenceStore = window.PoREBrowserPcmPersistenceStore ? new window.PoREBrowserPcmPersistenceStore() : null
	const completionJob = new CompletionJob({ persistenceStoreFactory: () => persistenceStore })
	const runtimeTransport = new RuntimeTransport({ completionJob })
	window.__poreTalkAudioConnector = connector
	window.__poreTalkRecordingController = recorder
	window.__poreTalkRecordingStateBridge = stateBridge
	window.__poreBrowserPcmPersistenceStore = persistenceStore
	window.__poreBrowserCompletionJob = completionJob
	window.__poreBrowserRuntimeTransport = runtimeTransport

	let context = null
	let sourceTrack = null
	let authoritativeState = null
	let productionId = null

	const startRequested = () => window.dispatchEvent(new CustomEvent('pore:recording-ui-start-local'))
	const stopRequested = () => window.dispatchEvent(new CustomEvent('pore:recording-ui-stop-local', { detail: { reason: 'host' } }))

	const render = nextContext => {
		if (!nextContext) return
		context = {
			...nextContext,
			...(productionId ? { productionId } : {}),
			...(authoritativeState || {}),
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
		if (!productionId) throw new Error('Talk production identity is not available')
		if (!authoritativeState?.recordingId) throw new Error('Authoritative recording identity is not available')
		await recorder.start(sourceTrack, {
			...(context?.sourceMetadata || {}),
			productionId,
			recordingId: authoritativeState.recordingId,
		})
		window.dispatchEvent(new CustomEvent('pore:recording-local-ready'))
	}

	const stopLocalCapture = async reason => recorder.stop(reason)

	window.addEventListener('pore:talk-production-identity', event => {
		const conversationId = event.detail?.conversationId || null
		if (!conversationId) return
		productionId = conversationId
		publish({ productionId })
	})

	window.addEventListener('pore:talk-audio-track', event => {
		if (sourceTrack && sourceTrack !== event.detail?.track && recorder.isRecording()) recorder.noteSourceChange(sourceTrack, event.detail?.track)
		sourceTrack = event.detail?.track || null
		if (sourceTrack) publish({ localCaptureAvailable: true })
	})

	window.addEventListener('pore:recording-started', event => {
		publish({ startedAt: event.detail?.startedAt || event.detail?.source?.startedAt })
	})

	window.addEventListener('pore:recording-local-finalized', event => {
		const artifact = event.detail
		publish({ artifact })
		if (!artifact) return
		try {
			const handoff = recorder.createPersistenceHandoff(artifact)
			void completionJob.enqueue(handoff).catch(error => {
				window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } }))
			})
			window.dispatchEvent(new CustomEvent('pore:recording-artifact-persistence-ready', { detail: handoff }))
		} catch (error) {
			window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } }))
		}
	})

	window.addEventListener('pore:recording-error', event => publish({ localCaptureError: event.detail?.error }))

	window.addEventListener('pore:recording-state', event => {
		const snapshot = event.detail
		if (!snapshot) return
		authoritativeState = snapshot
		if (snapshot.productionId) productionId = snapshot.productionId
		if (!context) return
		publish({
			productionId: snapshot.productionId || productionId,
			recordingId: snapshot.recordingId,
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
		try { await startLocalCapture() } catch (error) { window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } })) }
	})

	window.addEventListener('pore:recording-ui-stop-local', async event => {
		try { await stopLocalCapture(event.detail?.reason || 'host') } catch (error) { window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } })) }
	})

	const announceRecoveryCandidates = async () => {
		if (!persistenceStore) return
		try {
			const captures = await persistenceStore.listRecoverableCaptures()
			if (captures.length) window.dispatchEvent(new CustomEvent('pore:recording-recovery-available', { detail: { captures } }))
			await completionJob.recover()
		} catch (error) {
			window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } }))
		}
	}

	window.addEventListener('pore:recording-transport-ready', event => {
		window.dispatchEvent(new CustomEvent('pore:recording-completion-prepared', { detail: event.detail }))
	})

	const tryAttach = () => {
		if (connector.attachToTalk()) return
		window.setTimeout(tryAttach, 100)
	}

	void announceRecoveryCandidates()
	tryAttach()
})()