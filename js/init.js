/*
 * NC-PoRe — Talk recording UI bootstrap.
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
	let localCapturePrepareInFlight = false
	let localCaptureArmed = false
	let localRecordingStartInFlight = false
	let openingSignetEmitted = false
	let coordinationPollTimer = null
	let coordinationPollInFlight = false
	let openingTriggerInFlight = false
	let openingSignetRequestInFlight = false
	let localStopInFlight = false
	let talkUiMountElement = null

	document.addEventListener('click', event => {
		const action = event.target?.closest?.('.pore-talk-recording__button')
		if (!action) return
		console.debug('[NC-PoRe] UI action: click', { action: action.textContent })
	}, { capture: true })

	const updateAuthoritativeState = snapshot => {
		if (!snapshot) return
		authoritativeState = snapshot
		if (snapshot.productionId) productionId = snapshot.productionId
		if (!context) return
		publish({
			productionId: snapshot.productionId || productionId,
			productionStatus: snapshot.productionStatus || null,
			recordingId: snapshot.recordingId,
			role: snapshot.role,
			state: snapshot.state,
			listener: snapshot.listener,
			confirmed: snapshot.confirmed,
			ready: snapshot.ready,
			openingConfirmed: snapshot.openingConfirmed,
			readyCount: snapshot.readyCount,
			openingConfirmedCount: snapshot.openingConfirmedCount,
			participantCount: snapshot.participantCount,
			participants: snapshot.participants,
			elapsedSeconds: snapshot.elapsedSeconds,
			startedAt: snapshot.startedAt,
			error: snapshot.error,
		})
	}

	const prepareLocalCapture = async () => {
		if (!authoritativeState || authoritativeState.role === 'listener') return null
		const existing = localCapture.getCurrentTrack?.()
		if (existing?.readyState === 'live') {
			localCaptureArmed = true
			return existing
		}
		if (localCapturePrepareInFlight || !connector.isAudioPipelineReady?.()) return null
		localCapturePrepareInFlight = true
		try {
			const track = await localCapture.open()
			localCaptureArmed = true
			window.dispatchEvent(new CustomEvent('pore:recording-capture-ready', {
				detail: {
					trackId: track.id,
					deviceId: localCapture.getCurrentDeviceId?.() || track.getSettings?.()?.deviceId || null,
				},
			}))
			return track
		} finally {
			localCapturePrepareInFlight = false
		}
	}

	const startLocalRecording = async () => {
		if (recorder.isRecording() || localRecordingStartInFlight) return
		if (authoritativeState?.role === 'listener') return
		if (!productionId) throw new Error('Talk production identity is not available')
		if (!authoritativeState?.recordingId) throw new Error('Authoritative recording identity is not available')
		localRecordingStartInFlight = true
		try {
			const track = await prepareLocalCapture()
			if (!track || track.readyState !== 'live') throw new Error('PoRE local microphone capture is not armed')
			const deviceId = localCapture.getCurrentDeviceId?.() || track.getSettings?.()?.deviceId || null
			await recorder.start(track, {
				...(context?.sourceMetadata || {}),
				productionId,
				recordingId: authoritativeState.recordingId,
				productionLabel: context?.productionLabel || context?.title || productionId,
				participantLabel: context?.participantLabel || null,
				deviceId,
			})
			window.dispatchEvent(new CustomEvent('pore:recording-local-ready'))
			const result = await window.__poreTalkRecordingCoordinator?.command?.('ready')
			if (result?.state) updateAuthoritativeState(window.PoRETalkRecordingStateNormalize(result.state))
		} finally {
			localRecordingStartInFlight = false
		}
	}

	const scheduleLocalCapturePreparation = () => {
		if (!authoritativeState || authoritativeState.role === 'listener' || localCaptureArmed || localCapturePrepareInFlight) return
		if (!connector.isAudioPipelineReady?.()) {
			window.setTimeout(scheduleLocalCapturePreparation, 250)
			return
		}
		void prepareLocalCapture().catch(error => {
			window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } }))
		})
	}

	const switchLocalCaptureForMicrophone = async deviceId => {
		if (!deviceId || authoritativeState?.role === 'listener') return
		if (!recorder.isRecording()) return
		const previousTrack = localCapture.getCurrentTrack?.()
		const nextTrack = await localCapture.replace(deviceId)
		if (!nextTrack || nextTrack === previousTrack) return
		try {
			await recorder.replaceTrack(nextTrack)
			localCapture.commitReplacement(nextTrack)
			if (recorder.isRecording()) recorder.noteSourceChange(previousTrack, nextTrack, new Date().toISOString(), { from: { deviceId: previousTrack?.getSettings?.()?.deviceId || null }, to: { deviceId } })
		} catch (error) {
			localCapture.discardPendingReplacement()
			localCaptureArmed = false
			throw error
		}
	}

	const emitOpeningSignet = async () => {
		if (openingSignetEmitted || !recorder.isRecording()) return
		if (typeof recorder.markOpeningSignet === 'function') {
			recorder.markOpeningSignet()
			if (typeof recorder.waitForOpeningSignet === 'function') await recorder.waitForOpeningSignet()
		} else {
			window.dispatchEvent(new CustomEvent('pore:recording-opening-signet'))
		}
		openingSignetEmitted = true
	}

	const pollCoordination = async () => {
		if (coordinationPollInFlight) return
		const coordinator = window.__poreTalkRecordingCoordinator
		if (!coordinator?.command) return
		coordinationPollInFlight = true
		try {
			const result = await coordinator.command('snapshot')
			let snapshot = result?.state ? window.PoRETalkRecordingStateNormalize(result.state) : null
			if (!snapshot) return
			updateAuthoritativeState(snapshot)
			scheduleLocalCapturePreparation()

			if (snapshot.state === 'recording'
				&& snapshot.role !== 'listener'
				&& !recorder.isRecording()
				&& !localRecordingStartInFlight) {
				try {
					await startLocalRecording()
					const refreshed = await coordinator.command('snapshot')
					if (refreshed?.state) {
						snapshot = window.PoRETalkRecordingStateNormalize(refreshed.state)
						updateAuthoritativeState(snapshot)
					}
				} catch (error) {
					window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } }))
				}
			}

			if (snapshot.state === 'recording'
				&& snapshot.role === 'host'
				&& startRequestedByHost
				&& snapshot.readyCount >= snapshot.participantCount
				&& !snapshot.openingTriggered
				&& !openingTriggerInFlight) {
				openingTriggerInFlight = true
				try {
					const triggered = await coordinator.command('trigger_opening')
					if (triggered?.state) {
						snapshot = window.PoRETalkRecordingStateNormalize(triggered.state)
						updateAuthoritativeState(snapshot)
					}
				} finally {
					openingTriggerInFlight = false
				}
			}

			if (snapshot.state === 'opening' && snapshot.role !== 'listener') {
				const me = snapshot.participants?.find(participant => participant.id === coordinator.actorId)
				if (me?.ready && !me.opening_confirmed && !openingSignetRequestInFlight && recorder.isRecording()) {
					openingSignetRequestInFlight = true
					try {
						await emitOpeningSignet()
						const confirmed = await coordinator.command('confirm_opening')
						if (confirmed?.state) {
							snapshot = window.PoRETalkRecordingStateNormalize(confirmed.state)
							updateAuthoritativeState(snapshot)
						}
					} finally {
						openingSignetRequestInFlight = false
					}
				}
			}

			if (snapshot.state === 'stopped' && !localStopInFlight && recorder.isRecording()) {
				localStopInFlight = true
				try {
					await stopLocalCapture('remote-stop', { closingSignet: true })
					const acknowledged = await coordinator.command('acknowledge_stop')
					if (acknowledged?.state) updateAuthoritativeState(window.PoRETalkRecordingStateNormalize(acknowledged.state))
				} catch (error) {
					window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } }))
				} finally {
					localStopInFlight = false
				}
			}
		} catch (error) {
			if (error?.code === 'recording_coordination_not_found' || error?.code === 'recording_not_found') return
			if (authoritativeState?.state !== 'preparing'
				&& authoritativeState?.state !== 'ready'
				&& authoritativeState?.state !== 'recording'
				&& authoritativeState?.state !== 'stopped') return
			window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } }))
		} finally {
			coordinationPollInFlight = false
		}
	}

	const startCoordinationPolling = () => {
		if (coordinationPollTimer) return
		void pollCoordination()
		coordinationPollTimer = window.setInterval(() => { void pollCoordination() }, 3000)
	}

	const startRequested = async () => {
		console.debug('[NC-PoRe] startRequested: entered')
		if (!window.__poreTalkRecordingCoordinator?.command) throw new Error('PoRE recording coordinator is not available')
		startRequestedByHost = true
		try {
			const preparedTrack = await prepareLocalCapture()
			if (!preparedTrack) throw new Error('PoRE local microphone capture is not armed')
		} catch (error) {
			window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } }))
			return
		}
		console.debug('[NC-PoRe] startRequested: before begin')
		const result = await window.__poreTalkRecordingCoordinator.command('begin')
		if (result?.state) updateAuthoritativeState(window.PoRETalkRecordingStateNormalize(result.state))
		try { await startLocalRecording() } catch (error) { window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } })) }
		startCoordinationPolling()
	}

	const stopRequested = async () => window.dispatchEvent(new CustomEvent('pore:recording-ui-stop-local', { detail: { reason: 'host' } }))

	const forceCloseRequested = async () => {
		const coordinator = window.__poreTalkRecordingCoordinator
		if (!coordinator?.command) return
		try {
			await coordinator.command('force_close')
			await pollCoordination()
		} catch (error) {
			window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } }))
		}
	}

	const render = nextContext => {
		if (!nextContext) return
		context = {
			...nextContext,
			...(productionId ? { productionId } : {}),
			...(authoritativeState || {}),
			...(talkUiMountElement ? { mountElement: talkUiMountElement } : {}),
			onStart: nextContext.onStart || startRequested,
			onStop: nextContext.onStop || stopRequested,
			onForceClose: nextContext.onForceClose || forceCloseRequested,
		}
		Ui.mount(context)
	}

	const publish = patch => {
		if (!context) return
		const nextContext = { ...context, ...patch, ...(talkUiMountElement ? { mountElement: talkUiMountElement } : {}) }
		const uiStateFields = ['productionId', 'productionStatus', 'recordingId', 'role', 'state', 'listener', 'confirmed', 'ready', 'openingConfirmed', 'readyCount', 'openingConfirmedCount', 'participantCount', 'elapsedSeconds', 'startedAt']
		const currentKey = JSON.stringify(uiStateFields.map(field => context[field] ?? null))
		const nextKey = JSON.stringify(uiStateFields.map(field => nextContext[field] ?? null))
		context = nextContext
		if (currentKey === nextKey) return
		Ui.mount(context)
	}

	window.addEventListener('pore:talk-participants-updated', event => {
		if (startRequestedByHost) return
		if (['preparing', 'ready', 'recording', 'completed'].includes(authoritativeState?.state)) return
		const participantCount = event.detail?.participantCount
		if (!Number.isInteger(participantCount)) return
		publish({ participantCount })
	})

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
			void switchLocalCaptureForMicrophone(deviceId).catch(error => window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } })))
		}
		publish({ localCaptureAvailable: true, localCaptureDeviceId: deviceId })
	})

	window.addEventListener('pore:recording-started', event => publish({ startedAt: event.detail?.startedAt || event.detail?.source?.startedAt }))

	window.addEventListener('pore:recording-local-finalized', event => {
		localCaptureArmed = false
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
		const captureId = artifactId
		if (!artifactId || !window.__poreTalkRecordingCoordinator?.command) return
		try {
			const result = await window.__poreTalkRecordingCoordinator.command('complete', artifactId)
			if (result?.state) updateAuthoritativeState(window.PoRETalkRecordingStateNormalize(result.state))
			await completionJob.updateTransportState(captureId, { coreCompletionStatus: 'completed' })
			await completionJob.removeCapture(captureId)
		} catch (error) { window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } })) }
	})

	window.addEventListener('pore:recording-ui-context', event => {
		render(event.detail)
		scheduleLocalCapturePreparation()
	})

	const stopLocalCapture = async (reason, { closingSignet = false } = {}) => {
		try {
			if (closingSignet && recorder.isRecording()) recorder.markClosingSignet()
			return await recorder.stop(reason)
		} finally {
			localCapture.stop()
			localCaptureArmed = false
		}
	}

	window.addEventListener('pore:recording-ui-stop-local', async event => {
		const reason = event.detail?.reason || 'host'
		if (localStopInFlight) return

		localStopInFlight = true
		try {
			if (reason === 'persistence-safety-stop') {
				let coreStopped = false
				try {
					const result = await window.__poreTalkRecordingCoordinator?.command?.('stop')
					if (result?.state) {
						coreStopped = true
						updateAuthoritativeState(window.PoRETalkRecordingStateNormalize(result.state))
					}
				} catch (error) {
					console.warn('[NC-PoRe] Core stop unavailable during persistence safety stop', error)
				}
				await stopLocalCapture(reason, { closingSignet: false })
				if (coreStopped) {
					const acknowledged = await window.__poreTalkRecordingCoordinator?.command?.('acknowledge_stop')
					if (acknowledged?.state) updateAuthoritativeState(window.PoRETalkRecordingStateNormalize(acknowledged.state))
				}
				return
			}

			const result = await window.__poreTalkRecordingCoordinator?.command?.('stop')
			if (!result?.state) return
			updateAuthoritativeState(window.PoRETalkRecordingStateNormalize(result.state))
			await stopLocalCapture(reason, { closingSignet: true })
			const acknowledged = await window.__poreTalkRecordingCoordinator?.command?.('acknowledge_stop')
			if (acknowledged?.state) updateAuthoritativeState(window.PoRETalkRecordingStateNormalize(acknowledged.state))
		} catch (error) {
			window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } }))
		} finally {
			localStopInFlight = false
		}
	})

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
			scheduleLocalCapturePreparation()
		} catch (error) { window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } })) }
		finally { hostBootstrapInFlight = false }
	}

	window.addEventListener('pore:recording-ui-mount', () => { void bootstrapTalkCall() })
	void announceRecoveryCandidates()
	tryAttach()
	scheduleLocalCapturePreparation()
	void bootstrapTalkCall()
})()