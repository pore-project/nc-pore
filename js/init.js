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
	const HostAdapter = window.PoRETalkRecordingHostAdapter

	if (!Connector || !Recorder || !Ui || !StateBridge || !CompletionJob || !RuntimeTransport || !HostAdapter) return

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
	let startRequestedByHost = false
	let localCaptureStartInFlight = false
	let localCaptureReady = false
	let openingSignetEmitted = false
	let coordinationPollTimer = null
	let hostStartInFlight = false

	const updateAuthoritativeState = snapshot => {
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
	}

	const startLocalCapture = async () => {
		if (localCaptureReady || localCaptureStartInFlight) return
		if (!sourceTrack) throw new Error('Talk audio track is not available')
		if (!productionId) throw new Error('Talk production identity is not available')
		if (!authoritativeState?.recordingId) throw new Error('Authoritative recording identity is not available')
		localCaptureStartInFlight = true
		try {
			await recorder.start(sourceTrack, {
				...(context?.sourceMetadata || {}),
				productionId,
				recordingId: authoritativeState.recordingId,
				productionLabel: context?.productionLabel || context?.title || productionId,
			})
			localCaptureReady = true
			window.dispatchEvent(new CustomEvent('pore:recording-local-ready'))
			const result = await window.__poreTalkRecordingCoordinator?.command?.('ready')
			if (result?.state) updateAuthoritativeState(window.PoRETalkRecordingStateNormalize(result.state))
		} finally {
			localCaptureStartInFlight = false
		}
	}

	const emitOpeningSignet = () => {
		if (openingSignetEmitted || !recorder.isRecording()) return
		openingSignetEmitted = true
		if (typeof recorder.markOpeningSignet === 'function') recorder.markOpeningSignet()
		else window.dispatchEvent(new CustomEvent('pore:recording-opening-signet'))
	}

	const pollCoordination = async () => {
		const coordinator = window.__poreTalkRecordingCoordinator
		if (!coordinator?.command) return
		try {
			const result = await coordinator.command('snapshot')
			const snapshot = result?.state ? window.PoRETalkRecordingStateNormalize(result.state) : null
			if (!snapshot) return
			updateAuthoritativeState(snapshot)

			if (snapshot.state === 'preparing' && !localCaptureReady && snapshot.role !== 'listener' && startRequestedByHost || snapshot.state === 'preparing' && !localCaptureReady && snapshot.role !== 'listener') {
				await startLocalCapture()
			}

			if (snapshot.state === 'ready' && snapshot.role === 'host' && startRequestedByHost && !hostStartInFlight && snapshot.readyCount >= snapshot.participantCount) {
				hostStartInFlight = true
				try {
					const started = await coordinator.command('start')
					if (started?.state) updateAuthoritativeState(window.PoRETalkRecordingStateNormalize(started.state))
				} finally {
					hostStartInFlight = false
				}
			}

			if (snapshot.state === 'recording') emitOpeningSignet()
		} catch (error) {
			if (authoritativeState?.state !== 'preparing' && authoritativeState?.state !== 'ready' && authoritativeState?.state !== 'recording') return
			window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } }))
		}
	}

	const startCoordinationPolling = () => {
		if (coordinationPollTimer) return
		void pollCoordination()
		coordinationPollTimer = window.setInterval(() => { void pollCoordination() }, 500)
	}

	const startRequested = async () => {
		if (!window.__poreTalkRecordingCoordinator?.command) throw new Error('PoRE recording coordinator is not available')
		startRequestedByHost = true
		const result = await window.__poreTalkRecordingCoordinator.command('begin')
		if (result?.state) updateAuthoritativeState(window.PoRETalkRecordingStateNormalize(result.state))
		startCoordinationPolling()
	}

	const stopRequested = async () => {
		window.dispatchEvent(new CustomEvent('pore:recording-ui-stop-local', { detail: { reason: 'host' } }))
	}

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

	const stopLocalCapture = async reason => recorder.stop(reason)

	window.addEventListener('pore:talk-production-identity', event => {
		const conversationId = event.detail?.conversationId || null
		if (!conversationId) return
		productionId = conversationId
		publish({ productionId, productionLabel: event.detail?.productionLabel || conversationId })
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
	window.addEventListener('pore:recording-state', event => updateAuthoritativeState(event.detail))

	window.addEventListener('pore:recording-transport-completed', async event => {
		const artifactId = event.detail?.artifact_id || event.detail?.artifactId
		if (!artifactId || !window.__poreTalkRecordingCoordinator?.command) return
		try {
			const result = await window.__poreTalkRecordingCoordinator.command('complete', artifactId)
			if (result?.state) updateAuthoritativeState(window.PoRETalkRecordingStateNormalize(result.state))
		} catch (error) {
			window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } }))
		}
	})

	window.addEventListener('pore:recording-ui-context', event => render(event.detail))

	window.addEventListener('pore:recording-ui-stop-local', async event => {
		try {
			await stopLocalCapture(event.detail?.reason || 'host')
			const result = await window.__poreTalkRecordingCoordinator?.command?.('stop')
			if (result?.state) updateAuthoritativeState(window.PoRETalkRecordingStateNormalize(result.state))
		} catch (error) {
			window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } }))
		}
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
	void HostAdapter.bootstrap().then(() => startCoordinationPolling()).catch(error => window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } })))
})()
