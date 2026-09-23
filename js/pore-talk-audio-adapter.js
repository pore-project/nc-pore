/*
 * NC-PoRE — Nextcloud Talk audio host adapter.
 *
 * PoRE owns the microphone capture and preservation master. Talk receives only
 * a clone of that master through this adapter. The adapter is the only layer
 * that knows about Talk's private media-pipeline internals.
 *
 * IMPORTANT ARCHITECTURE NOTE:
 * During the V1 investigation we looked for a supported insertion point in
 * Talk before Talk's communication processing. No suitable insertion point
 * was available. The first usable Talk-side signal in the investigated path
 * was already downstream of the communication/WebRTC processing and was
 * available for recording as a lossy Opus communication representation.
 * That signal is therefore not a suitable PoRE master.
 *
 * The resolved V1 direction is deliberately:
 *
 *     microphone -> PoRE local capture (MASTER) -> 1:1 CLONE -> Talk
 *
 * This adapter implements only the PoRE -> Talk clone leg. It must not be
 * "simplified" back into recording Talk's current track. The separate
 * pore-talk-audio-connector only observes which microphone Talk selected and
 * tells PoRE when that selection changes, so PoRE can switch its own master
 * capture. PoRE never takes its recording source from Talk.
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
			this._masterTrack = null
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
			if (this._masterTrack === masterTrack && this._source.getOutputTrack('audio')?.readyState === 'live') return this._source.getOutputTrack('audio')
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
			this._masterTrack = masterTrack
			this._source.setTrack(clone)

			// INTENTIONAL: this is the superseded Talk-side input track (or the
			// previous PoRE clone), not the PoRE master. Once the Talk pipeline is
			// switched to the new PoRE clone, the old Talk-side track must be
			// retired. Never stop or mutate masterTrack here.
			if (oldTalkTrack && oldTalkTrack !== clone) oldTalkTrack.stop()
			return clone
		}

		clearMasterTrack() {
			if (!this._installed || !this._mediaDevicesSource || !this._audioTrackEnabler) return
			this._audioTrackEnabler.disconnectTrackSource('default', this._source, 'audio')
			this._source.setTrack(null)
			this._masterTrack = null
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

	const adapter = new TalkAudioAdapter()
	window.PoRETalkAudioAdapter = TalkAudioAdapter
	window.__poreTalkAudioAdapter = adapter

	const connectCurrentMasterTrack = () => {
		const track = window.__poreLocalAudioCapture?.getCurrentTrack?.()
		if (!track || track.readyState !== 'live') return
		try {
			adapter.connectMasterTrack(track)
			window.dispatchEvent(new CustomEvent('pore:talk-audio-adapter-connected', { detail: { masterTrackId: track.id } }))
		} catch (error) {
			window.dispatchEvent(new CustomEvent('pore:recording-error', { detail: { error } }))
		}
	}

	const attachAndConnect = () => {
		if (!adapter.attachToTalk()) return false
		connectCurrentMasterTrack()
		return true
	}

	window.addEventListener('pore:recording-capture-ready', connectCurrentMasterTrack)
	window.addEventListener(window.PoRETalkAudioPipelineReadyEvent || 'pore:talk-audio-pipeline-ready', attachAndConnect)

	window.addEventListener('pore:recording-master-track-changed', event => {
		const track = event.detail?.track
		if (!track) return
		try {
			adapter.connectMasterTrack(track)
			window.dispatchEvent(new CustomEvent('pore:talk-audio-adapter-connected', { detail: { masterTrackId: track.id, replaced: true } }))
		} catch (error) {
			window.dispatchEvent(new CustomEvent('pore:recording-error', { detail: { error } }))
		}
	})

	window.addEventListener('pore:recording-local-finalized', () => adapter.clearMasterTrack())

	attachAndConnect()
})()
