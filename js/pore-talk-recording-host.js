/* NC-PoRE — Talk host adapter for the authoritative recording coordinator. */
(() => {
	'use strict'

	const API_VERSION = '/ocs/v2.php/apps/pore/v1/recordings/command'
	const PRODUCTION_API_VERSION = '/ocs/v2.php/apps/pore/v1/productions/command'
	const TALK_API_VERSION = '/ocs/v2.php/apps/spreed/api/v4'
	let coordinatorContext = null
	let liveParticipantPollTimer = null
	let liveParticipantPollInFlight = false
	let liveParticipantIds = []

	const url = path => window.OC?.generateUrl ? window.OC.generateUrl(path) : path

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

	const getTalkParticipants = async token => {
		const participants = await requestJson(url(`${TALK_API_VERSION}/room/${encodeURIComponent(token)}/participants`))
		const participantList = Array.isArray(participants) ? participants : []
		return participantList
	}

	const getRecordingParticipantIds = participantList => participantList
		.filter(participant => participant?.actorType === 'users')
		.map(participant => participant.actorId)
		.filter(Boolean)

	const getCurrentRecordingParticipantIds = async token => getRecordingParticipantIds(await getTalkParticipants(token))

	const getParticipantLabel = (participantList, actorId, ownerId) => {
		const participant = participantList.find(item => item?.actorType === 'users' && item.actorId === actorId)
		const displayName = String(participant?.displayName || '').trim()
		if (displayName) return displayName
		if (actorId === ownerId) return 'Host'
		const participantIds = getRecordingParticipantIds(participantList)
			.filter(id => id !== ownerId)
		const index = participantIds.indexOf(actorId)
		return `Participant ${index >= 0 ? index + 1 : 1}`
	}

	const mergeParticipantOrder = participantIds => {
		if (!coordinatorContext) return participantIds
		const active = new Set(participantIds)
		const preserved = coordinatorContext.participants.filter(id => active.has(id))
		const appended = participantIds.filter(id => !preserved.includes(id))
		return [...preserved, ...appended]
	}

	const stopLiveParticipantPolling = () => {
		if (liveParticipantPollTimer) {
			window.clearInterval(liveParticipantPollTimer)
			liveParticipantPollTimer = null
		}
	}

	const refreshLiveParticipantCount = async token => {
		if (liveParticipantPollInFlight || !coordinatorContext || coordinatorContext.sessionId !== token) return
		liveParticipantPollInFlight = true
		try {
			const participantIds = await getCurrentRecordingParticipantIds(token)
			const changed = participantIds.length !== liveParticipantIds.length
				|| participantIds.some((participantId, index) => participantId !== liveParticipantIds[index])
			liveParticipantIds = participantIds
			coordinatorContext.participants = mergeParticipantOrder(participantIds)
			if (changed) {
				console.debug('[NC-PoRe] Talk participant count refreshed', { token, participantIds })
				window.dispatchEvent(new CustomEvent('pore:talk-participants-updated', {
					detail: {
						conversationId: token,
						participantCount: participantIds.length,
						participants: participantIds,
					},
				}))
			}
		} catch (error) {
			console.debug('[NC-PoRe] Talk participant count refresh failed', { token, error })
		} finally {
			liveParticipantPollInFlight = false
		}
	}

	const startLiveParticipantPolling = token => {
		stopLiveParticipantPolling()
		liveParticipantIds = coordinatorContext?.participants || []
		liveParticipantPollTimer = window.setInterval(() => { void refreshLiveParticipantCount(token) }, 3000)
		void refreshLiveParticipantCount(token)
	}

	const publishState = snapshot => {
		if (!snapshot || !coordinatorContext) return
		const actorId = coordinatorContext.actorId
		window.__poreTalkRecordingStateBridge?.publish({
			productionId: coordinatorContext.sessionId,
			recordingId: snapshot.recording_id,
			role: snapshot.role,
			state: snapshot.phase,
			confirmed: snapshot.confirmed,
			ready: snapshot.participants?.some(participant => participant.id === actorId && participant.ready) === true,
			readyCount: snapshot.participants?.filter(participant => participant.ready).length || 0,
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
			const production = await productionCommand(sessionId, 'ensure', options)
			if (production?.production_status !== 'active') return { production_status: production?.production_status || null, state: null }
			return recordingCommand(sessionId, recordingId, name, options)
		}
		if (name === 'begin') {
			stopLiveParticipantPolling()
			const currentParticipantIds = await getCurrentRecordingParticipantIds(sessionId)
			coordinatorContext.participants = mergeParticipantOrder(currentParticipantIds)
			const beginOptions = { ...options, participants: coordinatorContext.participants }
			const production = await productionCommand(sessionId, 'ensure', beginOptions)
			if (production?.production_status === 'created') {
				await productionCommand(sessionId, 'start', beginOptions)
			}
			await recordingCommand(sessionId, recordingId, 'ensure', beginOptions)
			return recordingCommand(sessionId, recordingId, 'begin', beginOptions)
		}
		return recordingCommand(sessionId, recordingId, name, options)
	}

	const findToken = () => window.location.pathname.match(/(?:\/apps\/spreed)?\/(?:call|room)\/([^/]+)/)?.[1] || null
	const getCurrentUserId = () => window.OC?.getCurrentUser?.()?.uid || window.OC?.currentUser?.uid || null

	const bootstrap = async () => {
		const token = findToken()
		const actorId = getCurrentUserId()
		console.debug('[NC-PoRe] Talk bootstrap: entry', { token, actorId })
		if (!token || !actorId) {
			console.warn('[NC-PoRe] Talk bootstrap: missing token or actorId', { token, actorId })
			return
		}

		const room = await requestJson(url(`${TALK_API_VERSION}/room/${encodeURIComponent(token)}`))
		console.debug('[NC-PoRe] Talk bootstrap: room loaded', { token, room })
		const participantList = await getTalkParticipants(token)
		const participantIds = getRecordingParticipantIds(participantList)
		console.debug('[NC-PoRe] Talk bootstrap: participants loaded', { token, actorId, participantList, participantIds })
		if (!participantList.some(p => p?.actorType === 'users' && p.actorId === actorId)) {
			console.warn('[NC-PoRe] Talk bootstrap: current actor not found in Talk participant list', { token, actorId, participantList })
			return
		}

		const owner = participantList.find(p => p?.actorType === 'users' && p.participantType === 1)
		const ownerId = owner?.actorId || actorId
		const recordingId = `recording-${token}`
		coordinatorContext = { sessionId: token, recordingId, actorId, ownerId, participants: participantIds }
		liveParticipantIds = participantIds
		const participantLabel = getParticipantLabel(participantList, actorId, ownerId)
		console.debug('[NC-PoRe] Talk bootstrap: coordinator context prepared', { token, actorId, ownerId, participantIds, participantLabel })

		window.dispatchEvent(new CustomEvent('pore:talk-production-identity', { detail: { conversationId: token, productionLabel: room?.displayName || room?.name || token } }))

		await productionCommand(token, 'ensure', { participants: participantIds, ownerId })
		console.debug('[NC-PoRe] Talk bootstrap: production ensured', { token, participantIds, ownerId })

		window.__poreTalkRecordingCoordinator = Object.freeze({
			sessionId: token,
			recordingId,
			participants: participantIds,
			ownerId,
			command: (name, artifactId = '') => command(token, recordingId, name, { participants: coordinatorContext?.participants || participantIds, ownerId, artifactId }),
		})
		console.debug('[NC-PoRe] Talk bootstrap: coordinator published', { token, recordingId, ownerId, participantIds })

		window.dispatchEvent(new CustomEvent('pore:recording-ui-context', {
			detail: {
				mountElement: window.PoRETalkRecordingUiMount?.getMountElement?.() || null,
				productionId: token,
				productionLabel: room?.displayName || room?.name || token,
				recordingId,
				participantLabel,
				role: ownerId === actorId ? 'host' : 'participant',
				state: 'preparing',
				listener: false,
				confirmed: false,
				ready: false,
				readyCount: 0,
				participantCount: participantIds.length,
				participants: participantIds.map(id => ({ id, ready: false })),
			},
		}))
		startLiveParticipantPolling(token)
	}

	window.PoRETalkRecordingHostAdapter = Object.freeze({ bootstrap, command: (...args) => command(...args) })
})()
