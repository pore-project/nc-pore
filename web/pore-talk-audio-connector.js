/*
 * NC-PoRE — Nextcloud Talk device-selection observer
 *
 * Talk is deliberately NOT the PoRE audio source. The TrackEnabler is observed
 * only to learn which microphone Talk currently has selected. PoRE opens its
 * own capture from that device before Talk's communication processing/encoding
 * path. The resulting capture track is owned by PoRE.
 *
 * The connector may also expose the current Talk conversation identity. This is
 * host context only: callers map the provider-native conversation token to
 * Core.ProductionId without creating a second identity.
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
						throw new Error('PoRE Talk device observer requires the default audio input')
					}
					this._source = source
					this._outputTrackId = outputTrackId
					this._handleOutputTrackSet = (trackSource, selectedOutputTrackId, track) => {
						if (trackSource === this._source && selectedOutputTrackId === this._outputTrackId) {
							this._acceptTalkSelection(track)
						}
					}
					source.on('outputTrackSet', this._handleOutputTrackSet)
					this._acceptTalkSelection(source.getOutputTrack(outputTrackId))
				},
				disconnectTrackSource: (inputTrackId, source, outputTrackId = 'default') => {
					if (inputTrackId !== 'default' || this._source !== source || this._outputTrackId !== outputTrackId) {
						return
					}
					source.off('outputTrackSet', this._handleOutputTrackSet)
					this._source = null
					this._outputTrackId = null
					this._handleOutputTrackSet = null
					this._replaceCurrent(null)
				},
			}

			this._trackSink = sink
			this._trackEnabler = trackEnabler
			trackEnabler.connectTrackSink('default', sink)
			this._publishProductionIdentity()
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

		/**
		 * Returns the provider-native Talk conversation token, if Talk currently
		 * has a conversation joined. The value is intentionally not renamed or
		 * reinterpreted inside the connector.
		 */
		getCurrentConversationId() {
			return window.OCA?.Talk?.SimpleWebRTC?.webrtc?.signaling?.currentRoomToken ?? null
		}

		dispose() {
			this._detachFromTalk()
		}

		_publishProductionIdentity() {
			const conversationId = this.getCurrentConversationId()
			if (!conversationId) return

			this._dispatchEvent(new CustomEvent('pore:talk-production-identity', {
				detail: {
					provider: 'Talk',
					conversationId,
				},
			}))
		}

		_acceptTalkSelection(sourceTrack) {
			const generation = ++this._captureGeneration

			if (!sourceTrack || sourceTrack.kind !== 'audio') {
				this._replaceCurrent(null)
				return
			}

			this._publishProductionIdentity()

			const deviceId = sourceTrack.getSettings?.().deviceId || null
			const audio = {
				echoCancellation: false,
				noiseSuppression: false,
				autoGainControl: false,
			}
			if (deviceId) {
				audio.deviceId = { exact: deviceId }
			}

			this._getUserMedia({ audio }).then(stream => {
				if (generation !== this._captureGeneration) {
					stream.getTracks().forEach(track => track.stop())
					return
				}

				const captureTrack = stream.getAudioTracks()[0]
				if (!captureTrack) {
					stream.getTracks().forEach(track => track.stop())
					this._replaceCurrent(null)
					this._dispatchCaptureError(new Error('PoRE capture returned no audio track'), deviceId)
					return
				}

				this._replaceCurrent({ sourceTrack, captureTrack, deviceId })
				this._dispatchEvent(new CustomEvent(PORE_TALK_AUDIO_TRACK_EVENT, {
					detail: { track: captureTrack, sourceTrack, deviceId },
				}))
			}).catch(error => {
				if (generation !== this._captureGeneration) return
				this._replaceCurrent(null)
				this._dispatchCaptureError(error, deviceId)
			})
		}

		_dispatchCaptureError(error, deviceId) {
			this._dispatchEvent(new CustomEvent(PORE_TALK_AUDIO_CAPTURE_ERROR_EVENT, {
				detail: { error, deviceId },
			}))
		}

		_detachFromTalk() {
			++this._captureGeneration
			if (this._trackEnabler && this._trackSink) {
				this._trackEnabler.disconnectTrackSink('default', this._trackSink)
			}
			this._trackEnabler = null
			this._trackSink = null
			this._source = null
			this._outputTrackId = null
			this._handleOutputTrackSet = null
			this._replaceCurrent(null)
		}

		_replaceCurrent(next) {
			const previous = this._current
			this._current = next
			if (previous?.captureTrack?.readyState !== 'ended' && typeof previous?.captureTrack?.stop === 'function') {
				previous.captureTrack.stop()
			}
		}
	}

	window.PoRETalkAudioCaptureConnector = TalkAudioCaptureConnector
	window.PoRETalkAudioTrackEvent = PORE_TALK_AUDIO_TRACK_EVENT
	window.PoRETalkAudioCaptureErrorEvent = PORE_TALK_AUDIO_CAPTURE_ERROR_EVENT
})()
