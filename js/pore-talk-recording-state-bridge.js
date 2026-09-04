/*
 * NC-PoRE — authoritative recording-state bridge for the Talk UI.
 *
 * The bridge does not own recording state. The Talk/Core integration publishes
 * snapshots from the authoritative Application/Core boundary; this adapter
 * normalizes that snapshot and forwards it to the recording UI.
 */

(() => {
	'use strict'

	const EVENT = 'pore:recording-state'
	const REQUIRED_STATES = new Set(['preparing', 'ready', 'opening', 'recording', 'stopping', 'done', 'error'])

	const normalize = snapshot => {
		if (!snapshot || typeof snapshot !== 'object') return null
		const state = REQUIRED_STATES.has(snapshot.state) ? snapshot.state : 'preparing'
		const participants = Array.isArray(snapshot.participants) ? snapshot.participants : []
		const readyParticipants = participants.filter(participant => participant?.ready === true)
		return Object.freeze({
			role: snapshot.role || 'none',
			state,
			listener: snapshot.role === 'listener' || snapshot.listener === true,
			confirmed: snapshot.confirmed === true || state === 'done',
			ready: snapshot.ready === true,
			readyCount: Number.isFinite(snapshot.readyCount) ? snapshot.readyCount : readyParticipants.length,
			participantCount: Number.isFinite(snapshot.participantCount) ? snapshot.participantCount : participants.length,
			participants,
			elapsedSeconds: Number.isFinite(snapshot.elapsedSeconds) ? snapshot.elapsedSeconds : 0,
			startedAt: snapshot.startedAt || null,
			error: snapshot.error || null,
		})
	}

	class TalkRecordingStateBridge {
		constructor({ dispatchEvent = window.dispatchEvent.bind(window) } = {}) {
			this._dispatchEvent = dispatchEvent
			this._snapshot = null
		}

		publish(snapshot) {
			const normalized = normalize(snapshot)
			if (!normalized) return null
			this._snapshot = normalized
			this._dispatchEvent(new CustomEvent(EVENT, { detail: normalized }))
			return normalized
		}

		getSnapshot() { return this._snapshot }
	}

	window.PoRETalkRecordingStateBridge = TalkRecordingStateBridge
	window.PoRETalkRecordingStateEvent = EVENT
	window.PoRETalkRecordingStateNormalize = normalize
})()
