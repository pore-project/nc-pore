/*
 * NC-PoRe — Nextcloud Talk audio connector
 *
 * Talk-specific lifecycle policy lives here. The connector observes the
 * browser capture boundary but does not alter Talk's returned MediaStream.
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

			// Do not replace a live PoRE capture merely because Talk asked for
			// another stream. Talk can create auxiliary getUserMedia streams.
			// The first live audio source is therefore retained until its own
			// lifecycle ends. A subsequent distinct track becomes eligible only
			// after the current source has ended.
			if (this._current) {
				if (this._current.sourceTrack === sourceTrack) {
					return this._current.cloneTrack
				}

				if (this._current.sourceTrack.readyState !== 'ended') {
					return null
				}

				this._replaceCurrent(null)
			}

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
