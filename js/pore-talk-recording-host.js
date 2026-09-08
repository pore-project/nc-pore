/* NC-PoRE — Talk host adapter for the authoritative recording coordinator. */
(() => {
	'use strict'

	const API_VERSION = '/ocs/v2.php/apps/pore/v1/recordings/command'
	const PRODUCTION_API_VERSION = '/ocs/v2.php/apps/pore/v1/productions/command'
	const TALK_API_VERSION = '/ocs/v2.php/apps/spreed/api/v4'
	let coordinatorContext = null

	const url = path => window.OC?.generateUrl ? window.OC.generateUrl(path) : path

	const requestJson = async (target, options = {}) => {
		const response = await fetch(target, {
			credentials: 'same-origin',
			headers: {
			Accept: 'application/json',
			'OCS-APIRequest': 'true',
			...(options.body ? { 'Content-Type': 'application/x-www-form-urlencoded; charset=UTF-8' } : {}),
			...(window.OC?.requestToken ? { requesttoken: window.OC.requestToken } : {}),
		},
		...options,
	})
		const body = await response.json()
		if (!response.ok || body?.ocs?.meta?.status !== 'ok') {
			const error = new Error(body?.ocs?.data?.error_code || `PoRE command failed (${response.status})`)
			error.code = body?.ocs?.data?.error_code || null
			error.status = response.status
			throw error
		}
		return body.ocs.data
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
		const result = await requestJson(url(API_VERSION), { method: 'POST', body: params })
		publishState(result?.state)
		return result
	}

	const command = async (sessionId, recordingId, name, options = {}) => {
		if (name === 'begin') {
			// `begin` is a user intent, not a request to recreate an already
			// running Production/recording barrier. This matters after a local
			// capture failure or a UI reload: Core may already be waiting for READY.
			const production = await productionCommand(sessionId, 'ensure', options)
			if (production?.production_status === 'created') {
				await productionCommand(sessionId, 'start', options)
			}

			try {
				return await recordingCommand(sessionId, recordingId, 'snapshot', options)
			} catch (error) {
				if (error?.code !== 'recording_coordination_not_found' && error?.code !== 'invalid_state_transition') throw error
			}
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
		if (!token || !actorId) return

		const room = await requestJson(url(`${TALK_API_VERSION}/room/${encodeURIComponent(token)}`))
		const participants = await requestJson(url(`${TALK_API_VERSION}/room/${encodeURIComponent(token)}/participants`))
		const participantList = Array.isArray(participants) ? participants : []
		const participantIds = participantList.filter(p => p?.actorType === 'users').map(p => p.actorId).filter(Boolean)
		if (!participantList.some(p => p?.actorType === 'users' && p.actorId === actorId)) return

		const owner = participantList.find(p => p?.actorType === 'users' && p.participantType === 1)
		const ownerId = owner?.actorId || actorId
		const recordingId = `recording-${token}`
		coordinatorContext = { sessionId: token, recordingId, actorId, ownerId, participants: participantIds }

		window.dispatchEvent(new CustomEvent('pore:talk-production-identity', { detail: { conversationId: token, productionLabel: room?.displayName || room?.name || token } }))

		await productionCommand(token, 'ensure', { participants: participantIds, ownerId })
		const ensure = await recordingCommand(token, recordingId, 'ensure', { participants: participantIds, ownerId })
		if (ensure?.state) publishState(ensure.state)

		window.__poreTalkRecordingCoordinator = Object.freeze({
			sessionId: token,
			recordingId,
			participants: participantIds,
			ownerId,
			command: (name, artifactId = '') => command(token, recordingId, name, { participants: participantIds, ownerId, artifactId }),
		})

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
