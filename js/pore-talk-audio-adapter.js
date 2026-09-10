/*
 * NC-PoRE — Nextcloud Talk audio host adapter.
 *
 * PoRE owns the microphone capture and preservation master. Talk receives only
 * a clone of that master through this adapter. The adapter is the only layer
 * that knows about Talk's private media-pipeline internals.
 */

(() => {
	'use strict'

	class PoRETalkAudioSource {
		constructor() {
			this._track = null
			this._listeners = new Map()
			this._endedHandler = null
		}

		on(event, handler) {
			if (!this._listeners.has(event)) this._listeners.set(event, new Set())
			this._listeners.get(event).add(handler)
		}

		off(event, handler) {
			this._listeners.get(event)?.delete(handler)
		}

		_emit(event, ...args) {
			for (const handler of this._listeners.get(event) || []) handler(this, ...args)
		}

		getOutputTrack(trackId = 'audio') {
			if (trackId !== 'audio') throw new Error(`Invalid PoRE Talk audio track id: ${trackId}`)
			return this._track
		}

		connectTrackSink(outputTrackId, trackSink, inputTrackId = 'default') {
			trackSink.connectTrackSource(inputTrackId, this, outputTrackId)
		}

		disconnectTrackSink(outputTrackId, trackSink, inputTrackId = 'default') {
			trackSink.disconnectTrackSource(inputTrackId, this, outputTrackId)
		}

		setTrack(track) {
			if (this._endedHandler && this._track) this._track.removeEventListener('ended', this._endedHandler)
			this._track = track || null
			if (this._track) {
				this._endedHandler = () => {
					if (this._track === track) this.setTrack(null)
				}
				this._track.addEventListener('ended', this._endedHandler)
			} else {
				this._endedHandler = null
			}
			this._emit('outputTrackSet', 'audio', this._track)
		}

		setEnabled(enabled) {
			if (!this._track) return
			this._track.enabled = enabled
			this._emit('outputTrackEnabled', 'audio', enabled)
		}
	}

	class TalkAudioAdapter {
		constructor() {
			this._mediaDevicesSource = null
			this._audioTrackEnabler = null
			this._source = new PoRETalkAudioSource()
			this._installed = false
		}

		attachToTalk() {
			const webrtc = window.OCA?.Talk?.SimpleWebRTC?.webrtc
			const source = webrtc?._mediaDevicesSource
			const audioTrackEnabler = webrtc?._audioTrackEnabler
			if (!source || !audioTrackEnabler || typeof source.connectTrackSink !== 'function' || typeof source.disconnectTrackSink !== 'function') return false
			if (this._mediaDevicesSource === source && this._audioTrackEnabler === audioTrackEnabler) return true
			this.detachFromTalk()
			this._mediaDevicesSource = source
			this._audioTrackEnabler = audioTrackEnabler
			return true
		}

		connectMasterTrack(masterTrack) {
			if (!masterTrack || masterTrack.kind !== 'audio') throw new Error('PoRE Talk adapter requires an audio master track')
			if (!this._mediaDevicesSource || !this._audioTrackEnabler) {
				if (!this.attachToTalk()) throw new Error('Nextcloud Talk audio pipeline is not available')
			}

			const oldTalkTrack = this._mediaDevicesSource.getOutputTrack?.('audio') || null
			if (!this._installed) {
				this._mediaDevicesSource.disconnectTrackSink('audio', this._audioTrackEnabler)
				this._audioTrackEnabler.connectTrackSource('default', this._source, 'audio')
				this._installed = true
			}

			const clone = masterTrack.clone()
			this._source.setTrack(clone)
			if (oldTalkTrack && oldTalkTrack !== clone) oldTalkTrack.stop()
			return clone
		}

		clearMasterTrack() {
			if (!this._installed || !this._mediaDevicesSource || !this._audioTrackEnabler) return
			this._audioTrackEnabler.disconnectTrackSource('default', this._source, 'audio')
			this._source.setTrack(null)
			this._mediaDevicesSource.connectTrackSink('audio', this._audioTrackEnabler)
			this._installed = false
			try {
				this._mediaDevicesSource.setAudioAllowed?.(false)
				this._mediaDevicesSource.setAudioAllowed?.(true)
			} catch (_) {}
		}

		detachFromTalk() {
			if (this._installed) this.clearMasterTrack()
			this._mediaDevicesSource = null
			this._audioTrackEnabler = null
		}

		getCurrentTalkMicrophone() {
			const track = this._mediaDevicesSource?.getOutputTrack?.('audio')
			const settings = track?.getSettings?.() || {}
			return settings.deviceId ? { deviceId: settings.deviceId, settings } : null
		}
	}

	window.PoRETalkAudioAdapter = TalkAudioAdapter
})()
