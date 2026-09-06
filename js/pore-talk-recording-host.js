/* NC-PoRE — Talk host adapter for the authoritative recording coordinator. */
(() => {
	'use strict'

	const API_VERSION = '/ocs/v2.php/apps/pore/v1/recordings/command'
	const TALK_API_VERSION = '/ocs/v2.php/apps/spreed/api/v4'

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
			throw new Error(body?.ocs?.data?.error_code || `PoRE command failed (${response.status})`)
		}
		return body.ocs.data
	}

	const command = async (sessionId, recordingId, name, { participants = [], ownerId = '', artifactId = '' } = {}) => {
		const params = new URLSearchParams({
			sessionId,
			recordingId,
			command: name,
			requestId: window.crypto?.randomUUID ? window.crypto.randomUUID() : `${Date.now()}-${Math.random()}`,
			participants: JSON.stringify(participants),
			ownerId,
			artifactId,
		})
		return requestJson(url(API_VERSION), { method: 'POST', body: params })
	}

	const findToken = () => {
		const match = window.location.pathname.match(/\/apps\/spreed\/(?:call|room)\/([^/]+)/)
		return match?.[1] || null
	}

	const getCurrentUserId = () => window.OC?.getCurrentUser?.()?.uid || window.OC?.currentUser?.uid || null

	const bootstrap = async () => {
		const token = findToken()
		const actorId = getCurrentUserId()
		if (!token || !actorId) return

		const room = await requestJson(url(`${TALK_API_VERSION}/room/${encodeURIComponent(token)}`))
		const participants = await requestJson(url(`${TALK_API_VERSION}/room/${encodeURIComponent(token)}/participants`))
		const participantIds = (Array.isArray(participants) ? participants : [])
			.filter(participant => participant?.actorType === 'users')
			.map(participant => participant.actorId)
			.filter(Boolean)
		const current = (Array.isArray(participants) ? participants : []).find(participant => participant?.actorType === 'users' && participant.actorId === actorId)
		if (!current) return

		const owner = (Array.isArray(participants) ? participants : []).find(participant => participant?.actorType === 'users' && participant.participantType === 1)
		const ownerId = owner?.actorId || actorId
		const recordingId = `recording-${token}`

		window.dispatchEvent(new CustomEvent('pore:talk-production-identity', {
			detail: {
				conversationId: token,
				productionLabel: room?.displayName || room?.name || token,
			},
		}))

		const ensure = await command(token, recordingId, 'ensure', { participants: participantIds, ownerId })
		const begin = await command(token, recordingId, 'begin', { participants: participantIds, ownerId })
		let snapshot = begin?.state || ensure?.state || null
		try {
			const ready = await command(token, recordingId, 'ready', { participants: participantIds, ownerId })
			snapshot = ready?.state || snapshot
		} catch (error) {
			if (!String(error?.message || error).includes('invalid_state_transition')) throw error
		}

		if (snapshot) {
			window.__poreTalkRecordingStateBridge?.publish({
				productionId: token,
				recordingId: snapshot.recording_id,
				role: snapshot.role,
				state: snapshot.phase,
				confirmed: snapshot.confirmed,
				ready: snapshot.participants?.some(participant => participant.id === actorId && participant.ready) === true,
				readyCount: snapshot.participants?.filter(participant => participant.ready).length || 0,
				participantCount: snapshot.participants?.length || participantIds.length,
				participants: snapshot.participants || [],
				artifactId: snapshot.artifact_id || null,
			})
		}

		window.__poreTalkRecordingCoordinator = Object.freeze({
			sessionId: token,
			recordingId,
			participants: participantIds,
			ownerId,
			command: name => command(token, recordingId, name, { participants: participantIds, ownerId }),
		})
	}

	window.PoRETalkRecordingHostAdapter = Object.freeze({ bootstrap, command })
	void bootstrap().catch(error => window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } })))
})()
