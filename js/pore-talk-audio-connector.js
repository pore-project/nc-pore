/*
 * NC-PoRE — Nextcloud Talk microphone observer.
 *
 * Talk is a communication host, not the PoRE recording source. This connector
 * observes Talk's current local microphone selection and reports changes to
 * the host-neutral PoRE capture layer. It never clones, owns, stops, or
 * forwards Talk's audio track.
 */

(() => {
	'use strict'

	const MICROPHONE_EVENT = 'pore:talk-microphone'

	class TalkMicrophoneObserverSink {
		constructor(onTrack) {
			this._onTrack = onTrack
			this._source = null
			this._outputTrackId = null
		}

		connectTrackSource(inputTrackId, trackSource, outputTrackId = 'default') {
			if (inputTrackId !== 'default' || this._source) throw new Error('PoRE Talk microphone observer can only be connected once')
			this._source = trackSource
			this._outputTrackId = outputTrackId
			trackSource.on('outputTrackSet', this._handleOutputTrackSet)
			trackSource.on('outputTrackEnabled', this._handleOutputTrackEnabled)
			this._onTrack(trackSource.getOutputTrack(outputTrackId))
		}

		disconnectTrackSource(inputTrackId, trackSource, outputTrackId = 'default') {
			if (inputTrackId !== 'default' || this._source !== trackSource || this._outputTrackId !== outputTrackId) return
			trackSource.off('outputTrackSet', this._handleOutputTrackSet)
			trackSource.off('outputTrackEnabled', this._handleOutputTrackEnabled)
			this._source = null
			this._outputTrackId = null
			this._onTrack(null)
		}

		_handleOutputTrackSet = (trackSource, outputTrackId, track) => {
			if (trackSource === this._source && outputTrackId === this._outputTrackId) this._onTrack(track)
		}

		_handleOutputTrackEnabled = () => {}
	}

	class TalkMicrophoneObserver {
		constructor({ dispatchEvent = window.dispatchEvent.bind(window) } = {}) {
			this._dispatchEvent = dispatchEvent
			this._mediaDevicesSource = null
			this._trackSink = null
			this._current = null
		}

		attachToTalk() {
			const source = window.OCA?.Talk?.SimpleWebRTC?.webrtc?._mediaDevicesSource
			if (!source || typeof source.connectTrackSink !== 'function' || typeof source.disconnectTrackSink !== 'function') return false
			if (this._mediaDevicesSource === source) return true
			this._detachFromTalk()
			const sink = new TalkMicrophoneObserverSink(track => this._observeTrack(track))
			source.connectTrackSink('audio', sink)
			this._mediaDevicesSource = source
			this._trackSink = sink
			return true
		}

		detachFromTalk() { this._detachFromTalk() }

		getCurrentMicrophone() { return this._current }

		dispose() { this._detachFromTalk(); this._current = null }

		_observeTrack(track) {
			const settings = track?.getSettings?.() || {}
			const deviceId = settings.deviceId || null
			const previous = this._current
			if (previous?.deviceId === deviceId && deviceId) return
			this._current = deviceId ? { deviceId, settings } : null
			this._dispatchEvent(new CustomEvent(MICROPHONE_EVENT, {
				detail: {
					previousDeviceId: previous?.deviceId || null,
					deviceId,
					settings,
					changed: previous?.deviceId !== deviceId,
				},
			}))
		}

		_detachFromTalk() {
			if (this._mediaDevicesSource && this._trackSink) this._mediaDevicesSource.disconnectTrackSink('audio', this._trackSink)
			this._mediaDevicesSource = null
			this._trackSink = null
		}
	}

	window.PoRETalkAudioCaptureConnector = TalkMicrophoneObserver
	window.PoRETalkMicrophoneEvent = MICROPHONE_EVENT
})()
