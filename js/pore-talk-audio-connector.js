/*
 * NC-PoRe — Nextcloud Talk audio connector
 *
 * Talk-specific lifecycle policy lives here. The connector attaches to
 * Talk's audio pipeline after TrackEnabler and before NoiseSuppressor.
 *
 * ADR-073i: the connector owns Talk track discovery and replacement only.
 * ADR-074i: recording stop remains outside Talk room termination.
 */

(() => {
	'use strict'

	const PORE_TALK_AUDIO_TRACK_EVENT = 'pore:talk-audio-track'

	class TalkAudioTrackSink {
		constructor(onTrack) {
			this._onTrack = onTrack
			this._source = null
			this._outputTrackId = null
		}

		connectTrackSource(inputTrackId, trackSource, outputTrackId = 'default') {
			if (inputTrackId !== 'default' || this._source) {
				throw new Error('PoRE Talk audio sink can only be connected once to the default input')
			}

			this._source = trackSource
			this._outputTrackId = outputTrackId
			trackSource.on('outputTrackSet', this._handleOutputTrackSet)
			trackSource.on('outputTrackEnabled', this._handleOutputTrackEnabled)
			this._onTrack(trackSource.getOutputTrack(outputTrackId))
		}

		disconnectTrackSource(inputTrackId, trackSource, outputTrackId = 'default') {
			if (inputTrackId !== 'default' || this._source !== trackSource || this._outputTrackId !== outputTrackId) {
				return
			}

			trackSource.off('outputTrackSet', this._handleOutputTrackSet)
			trackSource.off('outputTrackEnabled', this._handleOutputTrackEnabled)
			this._source = null
			this._outputTrackId = null
			this._onTrack(null)
		}

		_handleOutputTrackSet = (trackSource, outputTrackId, track) => {
			if (trackSource === this._source && outputTrackId === this._outputTrackId) {
				this._onTrack(track)
			}
		}

		_handleOutputTrackEnabled = () => {}
	}

	class TalkAudioCaptureConnector {
		constructor({ dispatchEvent = window.dispatchEvent.bind(window) } = {}) {
			this._dispatchEvent = dispatchEvent
			this._current = null
			this._talkWebRTC = null
			this._trackEnabler = null
			this._trackSink = null
		}

		attachToTalk() {
			const talkWebRTC = window.OCA?.Talk?.SimpleWebRTC?.webrtc
			const trackEnabler = talkWebRTC?._audioTrackEnabler

			if (!trackEnabler || typeof trackEnabler.connectTrackSink !== 'function'
				|| typeof trackEnabler.disconnectTrackSink !== 'function') {
				return false
			}

			if (this._trackEnabler === trackEnabler) {
				return true
			}

			this._detachFromTalk()

			const sink = new TalkAudioTrackSink((track) => this._acceptTrack(track))
			trackEnabler.connectTrackSink('default', sink)

			this._talkWebRTC = talkWebRTC
			this._trackEnabler = trackEnabler
			this._trackSink = sink
			return true
		}

		detachFromTalk() {
			this._detachFromTalk()
		}

		_acceptTrack(sourceTrack) {
			if (!sourceTrack || typeof sourceTrack.clone !== 'function') {
				this._replaceCurrent(null)
				return null
			}

			if (this._current?.sourceTrack === sourceTrack) {
				return this._current.cloneTrack
			}

			this._replaceCurrent(null)

			const cloneTrack = sourceTrack.clone()
			const current = { sourceTrack, cloneTrack, onEnded: null }

			current.onEnded = () => {
				if (this._current === current) {
					this._replaceCurrent(null)
				}
			}

			if (typeof sourceTrack.addEventListener === 'function') {
				sourceTrack.addEventListener('ended', current.onEnded)
			}

			this._current = current
			this._dispatchEvent(new CustomEvent(PORE_TALK_AUDIO_TRACK_EVENT, {
				detail: { track: cloneTrack, sourceTrack },
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
			this._detachFromTalk()
			this._replaceCurrent(null)
		}

		_detachFromTalk() {
			if (this._trackEnabler && this._trackSink) {
				this._trackEnabler.disconnectTrackSink('default', this._trackSink)
			}
			this._talkWebRTC = null
			this._trackEnabler = null
			this._trackSink = null
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
