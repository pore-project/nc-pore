/*
 * NC-PoRE — Nextcloud Talk audio connector
 *
 * Talk is used only as the host-side device/lifecycle signal. PoRE does not
 * record Talk's communication track because that track may already have been
 * processed for communication. The connector observes Talk's selected source
 * track only long enough to identify the microphone and then opens an
 * independent browser capture from that device with communication processing
 * disabled where the browser exposes those controls.
 */

(() => {
	'use strict'

	const PORE_TALK_AUDIO_TRACK_EVENT = 'pore:talk-audio-track'
	const PORE_TALK_AUDIO_CAPTURE_ERROR_EVENT = 'pore:talk-audio-capture-error'

	class TalkAudioCaptureConnector {
		constructor({
			dispatchEvent = window.dispatchEvent.bind(window),
			getUserMedia = (...args) => navigator.mediaDevices.getUserMedia(...args),
		} = {}) {
			this._dispatchEvent = dispatchEvent
			this._getUserMedia = getUserMedia
			this._talkWebRTC = null
			this._trackEnabler = null
			this._trackSink = null
			this._source = null
			this._outputTrackId = null
			this._current = null
			this._captureGeneration = 0
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

			const sink = {
				connectTrackSource: (inputTrackId, source, outputTrackId = 'default') => {
					if (inputTrackId !== 'default') {
						throw new Error('PoRE Talk audio connector requires the default audio input')
					}
					this._source = source
					this._outputTrackId = outputTrackId
					source.on('outputTrackSet', this._handleOutputTrackSet)
					this._acceptTalkTrack(source.getOutputTrack(outputTrackId))
				},
				disconnectTrackSource: (inputTrackId, source, outputTrackId = 'default') => {
					if (inputTrackId !== 'default' || this._source !== source || this._outputTrackId !== outputTrackId) {
						return
					}
					source.off('outputTrackSet', this._handleOutputTrackSet)
					this._source = null
					this._outputTrackId = null
					this._replaceCurrent(null)
				},
			}
			sink._handleOutputTrackSet = (trackSource, outputTrackId, track) => {
				if (trackSource === this._source && outputTrackId === this._outputTrackId) {
					this._acceptTalkTrack(track)
				}
			}

			this._handleOutputTrackSet = sink._handleOutputTrackSet
			trackEnabler.connectTrackSink('default', sink)

			this._talkWebRTC = talkWebRTC
			this._trackEnabler = trackEnabler
			this._trackSink = sink
			return true
		}

		detachFromTalk() {
			this._detachFromTalk()
		}

		getCurrentSourceTrack() {
			return this._current?.sourceTrack ?? null
		}

		getCurrentCaptureTrack() {
			return this._current?.captureTrack ?? null
		}

		getCurrentCloneTrack() {
			return this.getCurrentCaptureTrack()
		}

		dispose() {
			this._detachFromTalk()
			this._replaceCurrent(null)
		}

		_acceptTalkTrack(sourceTrack) {
			const generation = ++this._captureGeneration

			if (!sourceTrack || sourceTrack.kind !== 'audio') {
				this._replaceCurrent(null)
				return
			}

			const deviceId = sourceTrack.getSettings?.().deviceId || null
			const audio = {
				echoCancellation: false,
				noiseSuppression: false,
				autoGainControl: false,
			}
			if (deviceId) {
				audio.deviceId = { exact: deviceId }
			}

			this._getUserMedia({ audio }).then((stream) => {
				if (generation !== this._captureGeneration) {
					stream.getTracks().forEach(track => track.stop())
					return
				}

				const captureTrack = stream.getAudioTracks()[0]
				if (!captureTrack) {
					stream.getTracks().forEach(track => track.stop())
					this._replaceCurrent(null)
					this._dispatchEvent(new CustomEvent(PORE_TALK_AUDIO_CAPTURE_ERROR_EVENT, {
						detail: { error: new Error('PoRE capture returned no audio track'), deviceId },
					}))
					return
				}

				this._replaceCurrent({ sourceTrack, captureTrack })
				this._dispatchEvent(new CustomEvent(PORE_TALK_AUDIO_TRACK_EVENT, {
					detail: { track: captureTrack, sourceTrack },
				}))
			}).catch(error => {
				if (generation !== this._captureGeneration) {
					return
				}
				this._replaceCurrent(null)
				this._dispatchEvent(new CustomEvent(PORE_TALK_AUDIO_CAPTURE_ERROR_EVENT, {
					detail: { error, deviceId },
				}))
			})
		}

		_detachFromTalk() {
			++this._captureGeneration
			if (this._trackEnabler && this._trackSink) {
				this._trackEnabler.disconnectTrackSink('default', this._trackSink)
			}
			this._talkWebRTC = null
			this._trackEnabler = null
			this._trackSink = null
			this._source = null
			this._outputTrackId = null
			this._replaceCurrent(null)
		}

		_replaceCurrent(next) {
			const previous = this._current
			this._current = next

			if (!previous?.captureTrack) {
				return
			}

			if (previous.captureTrack.readyState !== 'ended' && typeof previous.captureTrack.stop === 'function') {
				previous.captureTrack.stop()
			}
		}
	}

	window.PoRETalkAudioCaptureConnector = TalkAudioCaptureConnector
	window.PoRETalkAudioTrackEvent = PORE_TALK_AUDIO_TRACK_EVENT
	window.PoRETalkAudioCaptureErrorEvent = PORE_TALK_AUDIO_CAPTURE_ERROR_EVENT
})()
