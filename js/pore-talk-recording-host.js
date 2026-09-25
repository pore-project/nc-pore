/* NC-PoRE — Talk host adapter for the authoritative recording coordinator. */
(() => {
	'use strict'

	const API_VERSION = '/ocs/v2.php/apps/pore/v1/recordings/command'
	const PRODUCTION_API_VERSION = '/ocs/v2.php/apps/pore/v1/productions/command'
	const TALK_API_VERSION = '/ocs/v2.php/apps/spreed/api/v4'
	const PORE_TALK_CONTEXT_API = '/ocs/v2.php/apps/pore/v1/talk/context'
	const TALK_SESSION_TAB_ID = 'x-nextcloud-talk-session-tab-id'
	let coordinatorContext = null
	const STOP_RETRY_INTERVAL_MS = 1000
	let stopRetryTimer = null
	let stopAcknowledgedParticipantIds = new Set()

	const url = path => window.OC?.generateUrl ? window.OC.generateUrl(path) : path

	const getTalkSessionTabId = () => {
		try {
			const value = window.sessionStorage?.getItem(TALK_SESSION_TAB_ID) || ''
			return /^[A-Za-z0-9]{64}$/.test(value) ? value : null
		} catch (_) {
			return null
		}
	}

	const coordinationChannel = () => window.__poreRecordingCoordinationChannel || null

	const publishRecordingSignal = async type => {
		const channel = coordinationChannel()
		if (!channel?.publish) throw new Error('PoRE recording coordination channel is not available')
		return channel.publish(type)
	}

	const clearStopRetry = () => {
		if (stopRetryTimer !== null) {
			window.clearTimeout(stopRetryTimer)
			stopRetryTimer = null
		}
	}

	const resetStopAcknowledgements = () => {
		stopAcknowledgedParticipantIds = new Set()
	}

	const hostOwnsCoordinator = () => coordinatorContext?.ownerId === coordinatorContext?.actorId

	const scheduleStopRetry = () => {
		clearStopRetry()
		if (!hostOwnsCoordinator() || !coordinatorContext || coordinatorContext.participants.length === 0) return

		const retry = async () => {
			stopRetryTimer = null
			if (!hostOwnsCoordinator() || !coordinatorContext) return
			const pending = coordinatorContext.participants.some(id => !stopAcknowledgedParticipantIds.has(id))
			if (!pending) return

			try {
				await publishRecordingSignal('stop')
			} catch (error) {
				console.warn('[NC-PoRe] Stop coordination retry failed', error)
			}

			if (coordinatorContext.participants.some(id => !stopAcknowledgedParticipantIds.has(id))) {
				stopRetryTimer = window.setTimeout(() => void retry(), STOP_RETRY_INTERVAL_MS)
			}
		}

		stopRetryTimer = window.setTimeout(() => void retry(), STOP_RETRY_INTERVAL_MS)
	}

	window.addEventListener('pore:recording-signal', event => {
		const signal = event.detail
		if (!coordinatorContext) return
		if (signal?.type === 'stop_acknowledged' && hostOwnsCoordinator()) {
			if (coordinatorContext.participants.includes(signal.actorId)) {
				stopAcknowledgedParticipantIds.add(signal.actorId)
				if (!coordinatorContext.participants.some(id => !stopAcknowledgedParticipantIds.has(id))) {
					clearStopRetry()
				}
			}
			return
		}
		if (signal?.type === 'production_closed') clearStopRetry()
	})


	const requestJson = async (target, options = {}) => {
		let response
		try {
			console.debug('[NC-PoRe] requestJson: before fetch', { target, method: options.method || 'GET' })
			response = await fetch(target, {
				credentials: 'same-origin',
				headers: {
					Accept: 'application/json',
					'OCS-APIRequest': 'true',
					...(options.body ? { 'Content-Type': 'application/x-www-form-urlencoded; charset=UTF-8' } : {}),
					...(window.OC?.requestToken ? { requesttoken: window.OC.requestToken } : {}),
					...(getTalkSessionTabId() ? { [TALK_SESSION_TAB_ID]: getTalkSessionTabId() } : {}),
				},
				...options,
			})
		} catch (error) {
			console.error('[NC-PoRe] requestJson: fetch threw', { target, method: options.method || 'GET', error, name: error?.name, message: error?.message })
			throw error
		}
		console.debug('[NC-PoRe] requestJson: fetch returned', { target, status: response.status, ok: response.ok })
		const body = await response.json()
		if (!response.ok || body?.ocs?.meta?.status !== 'ok') {
			const error = new Error(body?.ocs?.data?.error_code || `PoRE command failed (${response.status})`)
			error.code = body?.ocs?.data?.error_code || null
			error.status = response.status
			throw error
		}
		return body.ocs.data
	}

	const getTalkParticipants = async talkConversationId => {
		const participants = await requestJson(url(`${TALK_API_VERSION}/call/${encodeURIComponent(talkConversationId)}`))
		const talkParticipantList = Array.isArray(participants) ? participants : []
		return talkParticipantList
	}

	const getTalkRecordingParticipantIds = talkParticipantList => talkParticipantList
		.filter(talkParticipant => ['users', 'guests'].includes(talkParticipant?.actorType))
		.map(talkParticipant => talkParticipant.actorId)
		.filter(Boolean)

	const normalizeTalkContext = talkContextPayload => {
		const talkActorId = String(talkContextPayload?.actor_id || '').trim()
		const talkActorType = String(talkContextPayload?.actor_type || '').trim()
		const talkDisplayName = String(talkContextPayload?.display_name || '').trim()
		const talkParticipantType = talkContextPayload?.participant_type ?? null
		const talkSessionId = String(talkContextPayload?.talk_session_id || '').trim()
		const talkOwnerId = String(talkContextPayload?.owner_id || '').trim()
		const talkGuest = talkContextPayload?.guest === true

		return {
			talkActorId,
			talkActorType,
			talkDisplayName,
			talkParticipantType,
			talkSessionId,
			talkOwnerId,
			talkGuest,
		}
	}

	const getTalkContext = async talkConversationId => {
		const talkContextPayload = await requestJson(url(`${PORE_TALK_CONTEXT_API}?sessionId=${encodeURIComponent(talkConversationId)}`))
		const talkContext = normalizeTalkContext(talkContextPayload)
		if (!talkContext.talkActorId || !talkContext.talkActorType) throw new Error('PoRE Talk actor context is unavailable')
		return talkContext
	}

	const getCurrentRecordingParticipantIds = async talkConversationId => {
		const participants = await requestJson(url(`${TALK_API_VERSION}/call/${encodeURIComponent(talkConversationId)}`))
		return getTalkRecordingParticipantIds(Array.isArray(participants) ? participants : [])
	}

	const getTalkParticipantLabel = (talkParticipantList, talkActorId, talkOwnerId) => {
		const talkParticipant = talkParticipantList.find(item => ['users', 'guests'].includes(item?.actorType) && item.actorId === talkActorId)
		const talkDisplayName = String(talkParticipant?.displayName || '').trim()
		if (talkDisplayName) return talkDisplayName
		if (talkActorId === talkOwnerId) return 'Host'
		const talkParticipantIds = getTalkRecordingParticipantIds(talkParticipantList).filter(id => id !== talkOwnerId)
		const index = talkParticipantIds.indexOf(talkActorId)
		return talkParticipant?.actorType === 'guests'
			? `Gast ${index >= 0 ? index + 1 : 1}`
			: `Participant ${index >= 0 ? index + 1 : 1}`
	}

	const mergeParticipantOrder = participantIds => {
		if (!coordinatorContext) return participantIds
		const active = new Set(participantIds)
		const preserved = coordinatorContext.participants.filter(id => active.has(id))
		const appended = participantIds.filter(id => !preserved.includes(id))
		return [...preserved, ...appended]
	}

	const publishState = snapshot => {
		if (!snapshot || !coordinatorContext) return
		const actorId = coordinatorContext.actorId
		window.__poreTalkRecordingStateBridge?.publish({
			productionId: coordinatorContext.sessionId,
			productionStatus: snapshot.production_status || snapshot.productionStatus || null,
			recordingId: snapshot.recording_id,
			role: snapshot.role,
			state: snapshot.phase,
			confirmed: snapshot.confirmed,
			ready: snapshot.participants?.some(participant => participant.id === actorId && participant.ready) === true,
			openingConfirmed: snapshot.participants?.some(participant => participant.id === actorId && participant.opening_confirmed) === true,
			readyCount: snapshot.participants?.filter(participant => participant.ready).length || 0,
			openingConfirmedCount: snapshot.participants?.filter(participant => participant.opening_confirmed).length || 0,
			participantCount: snapshot.participants?.length || coordinatorContext.participants.length,
			participants: snapshot.participants || [],
			artifactId: snapshot.artifact_id || null,
		})
	}

	const productionCommand = async (sessionId, name, { participants = [], ownerId = '' } = {}) => {
		const params = new URLSearchParams({
			sessionId,
			command: name,
			requestId: window.crypto?.randomUUID ? window.crypto.randomUUID() : `${Date.now()}-${Math.random()}`,
			participants: JSON.stringify(participants),
			ownerId,
		})
		return requestJson(url(PRODUCTION_API_VERSION), { method: 'POST', body: params })
	}

	const recordingCommand = async (sessionId, recordingId, name, { participants = [], ownerId = '', artifactId = '' } = {}) => {
		const params = new URLSearchParams({
			sessionId,
			recordingId,
			command: name,
			requestId: window.crypto?.randomUUID ? window.crypto.randomUUID() : `${Date.now()}-${Math.random()}`,
			participants: JSON.stringify(participants),
			ownerId,
			artifactId,
		})
		console.debug('[NC-PoRe] Recording command: before request', { sessionId, recordingId, name, participants, ownerId, artifactId })
		const result = await requestJson(url(API_VERSION), { method: 'POST', body: params })
		const state = result?.state
		if (state) {
			const readyParticipants = Array.isArray(state.participants) ? state.participants.filter(participant => participant?.ready).map(participant => participant.id) : []
			console.debug('[NC-PoRe] Recording command: state summary', {
				sessionId,
				recordingId,
				name,
				phase: state.phase || null,
				participantCount: Array.isArray(state.participants) ? state.participants.length : 0,
				readyCount: readyParticipants.length,
				readyParticipantIds: readyParticipants,
				participants: Array.isArray(state.participants) ? state.participants.map(participant => ({ id: participant?.id || null, ready: participant?.ready === true })) : [],
			})
		}
		publishState(state)
		console.debug('[NC-PoRe] Recording command: state published', { sessionId, recordingId, name })
		return result
	}

	const command = async (sessionId, recordingId, name, options = {}) => {
		if (name === 'snapshot') {
			let production
			try {
				production = await productionCommand(sessionId, 'get', options)
			} catch (error) {
				if (error?.code === 'session_not_found') return { production_status: null, state: null }
				throw error
			}
			if (!['active', 'completed'].includes(production?.production_status)) {
				return { production_status: production?.production_status || null, state: null }
			}
			const recording = await recordingCommand(sessionId, recordingId, name, options)
			if (recording?.state) {
				recording.state = { ...recording.state, production_status: production.production_status }
				publishState(recording.state)
			}
			return { ...recording, production_status: production.production_status }
		}
		if (name === 'force_close') {
			clearStopRetry()
			resetStopAcknowledgements()
			const result = await productionCommand(sessionId, 'force_close', options)
			await publishRecordingSignal('production_closed')
			return result
		}
		if (name === 'trigger_opening') {
			const result = await recordingCommand(sessionId, recordingId, name, options)
			await publishRecordingSignal('opening')
			return result
		}
		if (name === 'confirm_opening') {
			const result = await recordingCommand(sessionId, recordingId, name, options)
			await publishRecordingSignal('opening_confirmed')
			return result
		}
		if (name === 'begin') {
			clearStopRetry()
			resetStopAcknowledgements()
			const currentParticipantIds = await getCurrentRecordingParticipantIds(sessionId)
			coordinatorContext.participants = mergeParticipantOrder(currentParticipantIds)
			const beginOptions = { ...options, participants: coordinatorContext.participants }
			const production = await productionCommand(sessionId, 'ensure', beginOptions)
			if (production?.production_status === 'created') {
				await productionCommand(sessionId, 'start', beginOptions)
			}
			await recordingCommand(sessionId, recordingId, 'ensure', beginOptions)
			const result = await recordingCommand(sessionId, recordingId, 'begin', beginOptions)
			await publishRecordingSignal('begin')
			return result
		}
		if (name === 'ready') {
			const result = await recordingCommand(sessionId, recordingId, name, options)
			await publishRecordingSignal('ready')
			return result
		}

		if (name === 'stop') {
			clearStopRetry()
			resetStopAcknowledgements()
			const result = await recordingCommand(sessionId, recordingId, name, options)
			await publishRecordingSignal('stop')
			scheduleStopRetry()
			return result
		}
		if (name === 'acknowledge_stop') {
			const result = await recordingCommand(sessionId, recordingId, name, options)
			await publishRecordingSignal('stop_acknowledged')
			return result
		}

		return recordingCommand(sessionId, recordingId, name, options)
	}

	const findTalkConversationId = () => window.location.pathname.match(/(?:\/apps\/spreed)?\/(?:call|room)\/([^/]+)/)?.[1] || null

	const bootstrap = async () => {
		const talkConversationId = findTalkConversationId()
		console.debug('[NC-PoRe] Talk bootstrap: entry', { talkConversationId })
		if (!talkConversationId) {
			console.warn('[NC-PoRe] Talk bootstrap: missing conversation id', { talkConversationId })
			return
		}

		const talkContext = await getTalkContext(talkConversationId)
		const talkActorId = talkContext.talkActorId
		const talkActorType = talkContext.talkActorType
		console.debug('[NC-PoRe] Talk bootstrap: actor context loaded', { talkConversationId, talkContext })

		const room = await requestJson(url(`${TALK_API_VERSION}/room/${encodeURIComponent(talkConversationId)}`))
		console.debug('[NC-PoRe] Talk bootstrap: room loaded', { talkConversationId, room })
		const talkParticipantList = await getTalkParticipants(talkConversationId)
		const talkParticipantIds = getTalkRecordingParticipantIds(talkParticipantList)
		console.debug('[NC-PoRe] Talk bootstrap: participants loaded', { talkConversationId, talkActorId, talkParticipantList, talkParticipantIds })
		if (!talkParticipantList.some(talkParticipant => ['users', 'guests'].includes(talkParticipant?.actorType) && talkParticipant.actorId === talkActorId)) {
			console.warn('[NC-PoRe] Talk bootstrap: current actor not found in Talk participant list', { talkConversationId, talkActorId, talkParticipantList })
			return
		}

		const talkOwner = talkParticipantList.find(talkParticipant => talkParticipant?.actorType === 'users' && talkParticipant.participantType === 1)
		const talkOwnerId = talkContext.talkOwnerId || talkOwner?.actorId || ''
		const recordingId = `recording-${talkConversationId}`
		coordinatorContext = { sessionId: talkConversationId, recordingId, actorId: talkActorId, actorType: talkActorType, ownerId: talkOwnerId, participants: talkParticipantIds }
		const participantLabel = talkContext.talkDisplayName || getTalkParticipantLabel(talkParticipantList, talkActorId, talkOwnerId)
		console.debug('[NC-PoRe] Talk bootstrap: coordinator context prepared', { talkConversationId, talkActorId, talkOwnerId, talkParticipantIds, participantLabel })

		window.dispatchEvent(new CustomEvent('pore:talk-production-identity', { detail: { conversationId: talkConversationId, productionLabel: room?.displayName || room?.name || talkConversationId } }))

		window.__poreTalkRecordingCoordinator = Object.freeze({
			sessionId: talkConversationId,
			recordingId,
			actorId: talkActorId,
			recordingParticipantId: talkActorId,
			participants: talkParticipantIds,
			ownerId: talkOwnerId,
			command: (name, artifactId = '') => command(talkConversationId, recordingId, name, { participants: coordinatorContext?.participants || talkParticipantIds, ownerId: talkOwnerId, artifactId }),
		})
		console.debug('[NC-PoRe] Talk bootstrap: coordinator published', { talkConversationId, recordingId, talkActorId, talkActorType, talkOwnerId, talkParticipantIds })

		const channel = coordinationChannel()
		if (!channel?.connect) throw new Error('PoRE recording coordination channel is not available')
		void channel.connect(talkConversationId, recordingId).catch(error => window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } })))

		window.dispatchEvent(new CustomEvent('pore:recording-ui-context', {
			detail: {
				mountElement: window.PoRETalkRecordingUiMount?.getMountElement?.() || null,
				productionId: talkConversationId,
				productionLabel: room?.displayName || room?.name || talkConversationId,
				recordingId,
				participantLabel,
				role: talkOwnerId === talkActorId ? 'host' : 'participant',
				guest: talkContext.talkGuest,
				state: 'preparing',
				listener: false,
				confirmed: false,
				ready: false,
				openingConfirmed: false,
				readyCount: 0,
				openingConfirmedCount: 0,
				participantCount: 0,
				productionStatus: null,
				participants: [],
			},
		}))
		
	}

	window.PoRETalkRecordingHostAdapter = Object.freeze({ bootstrap, command: (...args) => command(...args) })
})()
