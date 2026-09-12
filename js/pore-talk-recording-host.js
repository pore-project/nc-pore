/* NC-PoRE — Talk host adapter for the authoritative recording coordinator. */
(() => {
	'use strict'

	const API_VERSION = '/ocs/v2.php/apps/pore/v1/recordings/command'
	const PRODUCTION_API_VERSION = '/ocs/v2.php/apps/pore/v1/productions/command'
	const TALK_API_VERSION = '/ocs/v2.php/apps/spreed/api/v4'
	let coordinatorContext = null

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
		console.debug('[NC-PoRe] Recording command: request returned', { sessionId, recordingId, name, result })
		publishState(result?.state)
		console.debug('[NC-PoRe] Recording command: state published', { sessionId, recordingId, name })
		return result
	}

	const command = async (sessionId, recordingId, name, options = {}) => {
		if (name === 'begin') {
			const currentParticipantIds = await getCurrentRecordingParticipantIds(sessionId)
			coordinatorContext.participants = currentParticipantIds
			const beginOptions = { ...options, participants: currentParticipantIds }
			const production = await productionCommand(sessionId, 'ensure', beginOptions)
			if (production?.production_status === 'created') {
				await productionCommand(sessionId, 'start', beginOptions)
			}
			return recordingCommand(sessionId, recordingId, 'begin', beginOptions)
		}
		return recordingCommand(sessionId, recordingId, name, options)
	}

	// Talk 34 uses /call/<token> (and /room/<token> in other contexts),
	// while older deployments may expose the token below /apps/spreed.
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
		console.debug('[NC-PoRe] Talk bootstrap: coordinator context prepared', { token, actorId, ownerId, participantIds })

		window.dispatchEvent(new CustomEvent('pore:talk-production-identity', { detail: { conversationId: token, productionLabel: room?.displayName || room?.name || token } }))

		await productionCommand(token, 'ensure', { participants: participantIds, ownerId })
		console.debug('[NC-PoRe] Talk bootstrap: production ensured', { token, participantIds, ownerId })
		const ensure = await recordingCommand(token, recordingId, 'ensure', { participants: participantIds, ownerId })
		console.debug('[NC-PoRe] Talk bootstrap: recording ensured', { token, recordingId, state: ensure?.state })
		if (ensure?.state) publishState(ensure.state)

		window.__poreTalkRecordingCoordinator = Object.freeze({
			sessionId: token,
			recordingId,
			participants: participantIds,
			ownerId,
			command: (name, artifactId = '') => command(token, recordingId, name, { participants: coordinatorContext?.participants || participantIds, ownerId, artifactId }),
		})
		console.debug('[NC-PoRe] Talk bootstrap: coordinator published', { token, recordingId, ownerId, participantIds })

		const state = ensure?.state
		window.dispatchEvent(new CustomEvent('pore:recording-ui-context', {
			detail: {
				mountElement: window.PoRETalkRecordingUiMount?.getMountElement?.() || null,
				productionId: token,
				productionLabel: room?.displayName || room?.name || token,
				recordingId,
				role: state?.role || (ownerId === actorId ? 'host' : 'participant'),
				state: state?.phase || 'preparing',
				listener: false,
				confirmed: state?.confirmed === true,
				ready: state?.participants?.some(participant => participant.id === actorId && participant.ready) === true,
				readyCount: state?.participants?.filter(participant => participant.ready).length || 0,
				participantCount: state?.participants?.length || participantIds.length,
				participants: state?.participants || [],
			},
		}))
	}

	window.PoRETalkRecordingHostAdapter = Object.freeze({ bootstrap, command: (...args) => command(...args) })
})()
