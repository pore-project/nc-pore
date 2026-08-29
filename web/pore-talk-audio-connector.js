/*
 * NC-PoRe — Nextcloud Talk audio connector
 *
 * This file is loaded as an early Nextcloud init script. It deliberately uses
 * no ES-module syntax because Nextcloud's init-script loader supplies classic
 * scripts. The connector is exposed only as a small browser integration object;
 * PoRE's generic recording code does not depend on Talk.
 */

(() => {
	'use strict'

	const PORE_TALK_AUDIO_TRACK_EVENT = 'pore:talk-audio-track'

	class TalkAudioCaptureConnector {
		constructor({ dispatchEvent = window.dispatchEvent.bind(window) } = {}) {
			this._dispatchEvent = dispatchEvent
			this._current = null
		}

		acceptStream(stream, constraints) {
			if (!constraints?.audio || !stream || typeof stream.getAudioTracks !== 'function') {
				return null
			}

			const sourceTrack = stream.getAudioTracks()[0]
			if (!sourceTrack || typeof sourceTrack.clone !== 'function') {
				return null
			}

			if (this._current?.sourceTrack === sourceTrack) {
				return this._current.cloneTrack
			}

			this._replaceCurrent(null)

			const cloneTrack = sourceTrack.clone()
			const current = {
				sourceTrack,
				cloneTrack,
				onEnded: null,
			}

			current.onEnded = () => {
				if (this._current !== current) {
					return
				}

				this._replaceCurrent(null)
			}

			if (typeof sourceTrack.addEventListener === 'function') {
				sourceTrack.addEventListener('ended', current.onEnded)
			}

			this._current = current
			this._dispatchEvent(new CustomEvent(PORE_TALK_AUDIO_TRACK_EVENT, {
				detail: {
					track: cloneTrack,
					sourceTrack,
					stream,
					constraints,
				},
			}))

			return cloneTrack
		}

		getCurrentSourceTrack() {
			return this._current?.sourceTrack ?? null
		}

		getCurrentCloneTrack() {
			return this._current?.cloneTrack ?? null
		}

		dispose() {
			this._replaceCurrent(null)
		}

		_replaceCurrent(next) {
			const previous = this._current
			this._current = next

			if (!previous) {
				return
			}

			if (previous.onEnded && typeof previous.sourceTrack.removeEventListener === 'function') {
				previous.sourceTrack.removeEventListener('ended', previous.onEnded)
			}

			if (previous.cloneTrack && typeof previous.cloneTrack.stop === 'function') {
				previous.cloneTrack.stop()
			}
		}
	}

	window.PoRETalkAudioCaptureConnector = TalkAudioCaptureConnector
	window.PoRETalkAudioTrackEvent = PORE_TALK_AUDIO_TRACK_EVENT
})()
