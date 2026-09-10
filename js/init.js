/*
 * NC-PoRE — Talk recording UI bootstrap.
 *
 * Talk supplies the mount point, role/context and microphone selection events.
 * PoRE owns the actual local microphone capture and recording path.
 */

(() => {
	'use strict'

	const Connector = window.PoRETalkAudioCaptureConnector
	const LocalCapture = window.PoRELocalAudioCapture
	const Recorder = window.PoREBrowserRecordingController
	const Ui = window.PoRETalkRecordingUi
	const StateBridge = window.PoRETalkRecordingStateBridge
	const CompletionJob = window.PoREBrowserCompletionJob
	const RuntimeTransport = window.PoREBrowserRuntimeTransport
	const HostAdapter = window.PoRETalkRecordingHostAdapter

	if (!Connector || !LocalCapture || !Recorder || !Ui || !StateBridge || !CompletionJob || !RuntimeTransport || !HostAdapter) return

	const connector = new Connector()
	const localCapture = new LocalCapture()
	const recorder = new Recorder()
	const stateBridge = new StateBridge()
	const persistenceStore = window.PoREBrowserPcmPersistenceStore ? new window.PoREBrowserPcmPersistenceStore() : null
	const completionJob = new CompletionJob({ persistenceStoreFactory: () => persistenceStore })
	const runtimeTransport = new RuntimeTransport({ completionJob })
	window.__poreTalkAudioConnector = connector
	window.__poreLocalAudioCapture = localCapture
	window.__poreTalkRecordingController = recorder
	window.__poreTalkRecordingStateBridge = stateBridge
	window.__poreBrowserPcmPersistenceStore = persistenceStore
	window.__poreBrowserCompletionJob = completionJob
	window.__poreBrowserRuntimeTransport = runtimeTransport

	let context = null
	let authoritativeState = null
	let productionId = null
	let startRequestedByHost = false
	let localCaptureStartInFlight = false
	let localCaptureReady = false
	let openingSignetEmitted = false
	let coordinationPollTimer = null
	let hostStartInFlight = false
	let talkUiMountElement = null

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

	const startLocalCapture = async ({ signalReady = true } = {}) => {
		if (localCaptureReady || localCaptureStartInFlight) return
		const microphone = connector.getCurrentMicrophone?.()
		if (!microphone?.deviceId) throw new Error('Talk microphone selection is not available')
		if (!productionId) throw new Error('Talk production identity is not available')
		if (!authoritativeState?.recordingId) throw new Error('Authoritative recording identity is not available')
		localCaptureStartInFlight = true
		try {
			const track = await localCapture.open(microphone.deviceId)
			await recorder.start(track, {
				...(context?.sourceMetadata || {}),
				productionId,
				recordingId: authoritativeState.recordingId,
				productionLabel: context?.productionLabel || context?.title || productionId,
				deviceId: microphone.deviceId,
			})
			localCaptureReady = true
			window.dispatchEvent(new CustomEvent('pore:recording-local-ready'))
			if (signalReady) {
				const result = await window.__poreTalkRecordingCoordinator?.command?.('ready')
				if (result?.state) updateAuthoritativeState(window.PoRETalkRecordingStateNormalize(result.state))
			}
		} finally {
			localCaptureStartInFlight = false
		}
	}

	const restartLocalCaptureForMicrophone = async deviceId => {
		if (!deviceId || !startRequestedByHost || authoritativeState?.role === 'listener') return
		if (!recorder.isRecording()) return
		await recorder.stop('microphone-change')
		localCaptureReady = false
		await startLocalCapture({ signalReady: false })
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
			if (snapshot.state === 'preparing' && snapshot.role !== 'listener' && !snapshot.ready && !localCaptureReady && !localCaptureStartInFlight && connector.getCurrentMicrophone?.()?.deviceId) {
				try { await startLocalCapture() } catch (error) { window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } })) }
			}
			if (snapshot.state === 'ready' && snapshot.role === 'host' && startRequestedByHost && !hostStartInFlight && snapshot.readyCount >= snapshot.participantCount) {
				hostStartInFlight = true
				try {
					const started = await coordinator.command('start')
					if (started?.state) updateAuthoritativeState(window.PoRETalkRecordingStateNormalize(started.state))
				} finally { hostStartInFlight = false }
			}
			if (snapshot.state === 'recording') {
				emitOpeningSignet()
				if (coordinationPollTimer) { window.clearInterval(coordinationPollTimer); coordinationPollTimer = null }
			}
		} catch (error) {
			if (error?.code === 'recording_coordination_not_found') return
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
		try { await startLocalCapture() } catch (error) { window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } })) }
		startCoordinationPolling()
	}

	const stopRequested = async () => window.dispatchEvent(new CustomEvent('pore:recording-ui-stop-local', { detail: { reason: 'host' } }))

	const render = nextContext => {
		if (!nextContext) return
		context = {
			...nextContext,
			...(productionId ? { productionId } : {}),
			...(authoritativeState || {}),
			...(talkUiMountElement ? { mountElement: talkUiMountElement } : {}),
			onStart: nextContext.onStart || startRequested,
			onStop: nextContext.onStop || stopRequested,
		}
		Ui.mount(context)
	}

	const publish = patch => {
		if (!context) return
		const nextContext = { ...context, ...patch, ...(talkUiMountElement ? { mountElement: talkUiMountElement } : {}) }
		const uiStateFields = ['productionId', 'recordingId', 'role', 'state', 'listener', 'confirmed', 'ready', 'readyCount', 'participantCount', 'elapsedSeconds', 'startedAt']
		const currentKey = JSON.stringify(uiStateFields.map(field => context[field] ?? null))
		const nextKey = JSON.stringify(uiStateFields.map(field => nextContext[field] ?? null))
		context = nextContext
		if (currentKey === nextKey) return
		Ui.mount(context)
	}

	window.addEventListener('pore:recording-ui-mount', event => {
		talkUiMountElement = event.detail?.mountElement || null
		if (context) { context = { ...context, mountElement: talkUiMountElement }; Ui.mount(context) }
	})

	window.addEventListener('pore:talk-production-identity', event => {
		const conversationId = event.detail?.conversationId || null
		if (!conversationId) return
		productionId = conversationId
		publish({ productionId, productionLabel: event.detail?.productionLabel || conversationId })
	})

	window.addEventListener('pore:talk-microphone', event => {
		const deviceId = event.detail?.deviceId || null
		if (!deviceId) return
		if (event.detail?.previousDeviceId && event.detail.previousDeviceId !== deviceId && recorder.isRecording()) {
			void restartLocalCaptureForMicrophone(deviceId).catch(error => window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } })))
		}
		publish({ localCaptureAvailable: true, localCaptureDeviceId: deviceId })
	})

	window.addEventListener('pore:recording-started', event => publish({ startedAt: event.detail?.startedAt || event.detail?.source?.startedAt }))

	window.addEventListener('pore:recording-local-finalized', event => {
		localCaptureReady = false
		openingSignetEmitted = false
		const artifact = event.detail
		publish({ artifact })
		if (!artifact) return
		try {
			const handoff = recorder.createPersistenceHandoff(artifact)
			void completionJob.enqueue(handoff).catch(error => window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } })))
			window.dispatchEvent(new CustomEvent('pore:recording-artifact-persistence-ready', { detail: handoff }))
		} catch (error) { window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } })) }
	})

	window.addEventListener('pore:recording-error', event => publish({ localCaptureError: event.detail?.error }))
	window.addEventListener('pore:recording-state', event => updateAuthoritativeState(event.detail))

	window.addEventListener('pore:recording-transport-completed', async event => {
		const artifactId = event.detail?.artifact_id || event.detail?.artifactId
		if (!artifactId || !window.__poreTalkRecordingCoordinator?.command) return
		try {
			const result = await window.__poreTalkRecordingCoordinator.command('complete', artifactId)
			if (result?.state) updateAuthoritativeState(window.PoRETalkRecordingStateNormalize(result.state))
		} catch (error) { window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } })) }
	})

	window.addEventListener('pore:recording-ui-context', event => render(event.detail))

	window.addEventListener('pore:recording-ui-stop-local', async event => {
		try {
			await stopLocalCapture(event.detail?.reason || 'host')
			const result = await window.__poreTalkRecordingCoordinator?.command?.('stop')
			if (result?.state) updateAuthoritativeState(window.PoRETalkRecordingStateNormalize(result.state))
		} catch (error) { window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } })) }
	})

	const stopLocalCapture = async reason => {
		const artifact = await recorder.stop(reason)
		localCapture.stop()
		return artifact
	}

	const announceRecoveryCandidates = async () => {
		if (!persistenceStore) return
		try {
			const captures = await persistenceStore.listRecoverableCaptures()
			if (captures.length) window.dispatchEvent(new CustomEvent('pore:recording-recovery-available', { detail: { captures } }))
			await completionJob.recover()
		} catch (error) { window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } })) }
	}

	window.addEventListener('pore:recording-transport-ready', event => window.dispatchEvent(new CustomEvent('pore:recording-completion-prepared', { detail: event.detail })))

	const tryAttach = () => {
		if (connector.attachToTalk()) return
		window.setTimeout(tryAttach, 100)
	}

	let hostBootstrapInFlight = false
	let lastBootstrappedCallPath = null

	const bootstrapTalkCall = async () => {
		const callPath = window.location.pathname.match(/(?:\/apps\/spreed)?\/(?:call|room)\/([^/]+)/)?.[0] || null
		if (!callPath || hostBootstrapInFlight || lastBootstrappedCallPath === callPath) return
		hostBootstrapInFlight = true
		try {
			await HostAdapter.bootstrap()
			if (window.__poreTalkRecordingCoordinator?.sessionId) lastBootstrappedCallPath = callPath
			startCoordinationPolling()
		} catch (error) { window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } })) }
		finally { hostBootstrapInFlight = false }
	}

	window.addEventListener('pore:recording-ui-mount', () => { void bootstrapTalkCall() })
	void announceRecoveryCandidates()
	tryAttach()
	void bootstrapTalkCall()
})()
