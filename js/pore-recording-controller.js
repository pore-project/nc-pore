/*
 * NC-PoRe — neutral browser recording boundary.
 *
 * The recording controller consumes the independent PoRE capture supplied by
 * the host-neutral local capture component. It deliberately does not use
 * MediaRecorder: browser codec selection could turn the preservation master
 * into a lossy stream.
 */

(() => {
	'use strict'

	const technicalId = prefix => {
		if (window.crypto?.randomUUID) return `${prefix}-${window.crypto.randomUUID()}`
		return `${prefix}-${Date.now()}-${Math.random().toString(16).slice(2)}`
	}

	class PoRELocalAudioCapture {
		constructor({ mediaDevices = navigator.mediaDevices } = {}) {
			this._mediaDevices = mediaDevices
			this._stream = null
			this._track = null
			this._deviceId = null
			this._pendingStream = null
			this._pendingTrack = null
			this._pendingDeviceId = null
		}

		getCurrentTrack() { return this._track }
		getCurrentDeviceId() { return this._deviceId }

		async open(deviceId) {
			const { stream, track } = await this._getStream(deviceId)
			this._discardPendingReplacement()
			this._replaceStream(stream, deviceId, track)
			return track
		}

		async replace(deviceId) {
			if (!deviceId || deviceId === this._deviceId) return this._track
			const { stream, track } = await this._getStream(deviceId)
			this._discardPendingReplacement()
			this._pendingStream = stream
			this._pendingTrack = track
			this._pendingDeviceId = deviceId
			return track
		}

		commitReplacement(track) {
			if (!this._pendingStream || !this._pendingTrack || (track && this._pendingTrack !== track)) return this._track
			const nextStream = this._pendingStream
			const nextTrack = this._pendingTrack
			const nextDeviceId = this._pendingDeviceId
			this._pendingStream = null
			this._pendingTrack = null
			this._pendingDeviceId = null
			this._replaceStream(nextStream, nextDeviceId, nextTrack)
			return nextTrack
		}

		discardPendingReplacement() { this._discardPendingReplacement() }

		stop() {
			this._discardPendingReplacement()
			this._replaceStream(null, null, null)
		}

		async _getStream(deviceId) {
			if (!this._mediaDevices?.getUserMedia) throw new Error('PoRE local microphone capture is not available')
			if (!deviceId) throw new Error('PoRE requires the currently selected Talk microphone')
			const stream = await this._mediaDevices.getUserMedia({ audio: {
				deviceId: { exact: deviceId },
				echoCancellation: false,
				noiseSuppression: false,
				autoGainControl: false,
			} })
			const track = stream.getAudioTracks?.()[0] || null
			if (!track) {
				stream.getTracks?.().forEach(item => item.stop())
				throw new Error('PoRE local microphone capture returned no audio track')
			}
			return { stream, track }
		}

		_discardPendingReplacement() {
			this._pendingStream?.getTracks?.().forEach(item => item.stop())
			this._pendingStream = null
			this._pendingTrack = null
			this._pendingDeviceId = null
		}

		_replaceStream(stream, deviceId, track) {
			const previous = this._stream
			this._stream = stream
			this._track = track
			this._deviceId = deviceId
			previous?.getTracks?.().forEach(item => item.stop())
		}
	}

	class PoREBrowserRecordingController {
		constructor({ recorderFactory = options => new window.PoREBrowserPcmRecorder(options) } = {}) {
			this.recorderFactory = recorderFactory
			this.recorder = null
			this.state = 'idle'
			this.sequence = 0
			this.sourceChanges = []
			this.initialSource = null
			this.captureId = null
			this.recordingSessionId = null
			this.currentTrack = null
		}

		getState() { return this.state }
		isRecording() { return this.state === 'recording' }

		_createRecorder() {
			return this.recorderFactory({
				onPersistenceSafetyStop: detail => {
					window.dispatchEvent(new CustomEvent('pore:recording-ui-stop-local', { detail: { reason: 'persistence-safety-stop', persistence: detail } }))
				},
			})
		}

		async primeAudioContext() {
			if (!this.recorder) this.recorder = this._createRecorder()
			if (typeof this.recorder.primeAudioContext !== 'function') throw new Error('PoRE PCM recorder cannot prime Web Audio')
			await this.recorder.primeAudioContext()
		}

		async start(track, sourceMetadata = {}) {
			if (this.isRecording()) throw new Error('PoRE recording is already active')
			if (!track || track.kind !== 'audio') throw new Error('PoRE requires an owned audio MediaStreamTrack')
			if (track.readyState !== 'live') throw new Error('PoRE cannot start from an ended audio track')
			if (!this.recorder) this.recorder = this._createRecorder()
			this.sourceChanges = []
			this.sequence += 1
			this.captureId = sourceMetadata.captureId || technicalId('browser-capture')
			this.recordingSessionId = sourceMetadata.recordingSessionId || technicalId('browser-session')
			this.initialSource = this._sourceMetadata(track, { ...sourceMetadata, captureId: this.captureId, recordingSessionId: this.recordingSessionId })
			this.state = 'starting'

			try {
				await this.recorder.start(track, { ...sourceMetadata, captureId: this.captureId, recordingSessionId: this.recordingSessionId })
				this.currentTrack = track
				this.state = 'recording'
				window.dispatchEvent(new CustomEvent('pore:recording-started', { detail: { sequence: this.sequence, startedAt: this.recorder.startedAt || this.initialSource.startedAt, source: this.initialSource } }))
			} catch (error) {
				this.state = 'error'; this.recorder = null; this.currentTrack = null; this.captureId = null; this.recordingSessionId = null; throw error
			}
		}

		async replaceTrack(track) {
			if (!this.recorder || !this.isRecording()) throw new Error('PoRE microphone replacement requires an active local recording')
			if (!track || track.kind !== 'audio') throw new Error('PoRE requires an owned audio MediaStreamTrack')
			if (track.readyState !== 'live') throw new Error('PoRE cannot replace the active microphone with an ended audio track')
			if (typeof this.recorder.replaceTrack !== 'function') throw new Error('PoRE PCM recorder cannot replace the active microphone')
			const previousTrack = this.currentTrack
			const nextTrack = await this.recorder.replaceTrack(track) || track
			this.currentTrack = nextTrack
			window.dispatchEvent(new CustomEvent('pore:recording-master-track-changed', {
				detail: { sequence: this.sequence, previousTrack, track: nextTrack, trackId: nextTrack.id, deviceId: nextTrack.getSettings?.()?.deviceId || null },
			}))
			return nextTrack
		}

		markOpeningSignet(at = new Date().toISOString()) {
			if (!this.recorder || !this.isRecording()) throw new Error('PoRE opening signet requires an active local capture')
			return this.recorder.markOpeningSignet(at)
		}

		async waitForOpeningSignet() {
			if (!this.recorder || !this.isRecording()) throw new Error('PoRE opening signet requires an active local capture')
			if (typeof this.recorder.waitForOpeningSignet !== 'function') return true
			return this.recorder.waitForOpeningSignet()
		}

		markClosingSignet(at = new Date().toISOString()) {
			if (!this.recorder || !this.isRecording()) throw new Error('PoRE closing signet requires an active local capture')
			return this.recorder.markClosingSignet(at)
		}

		async waitForClosingSignet() {
			if (!this.recorder || !this.isRecording()) throw new Error('PoRE closing signet requires an active local capture')
			if (typeof this.recorder.waitForClosingSignet !== 'function') return true
			return this.recorder.waitForClosingSignet()
		}

		noteSourceChange(previousTrack, nextTrack, occurredAt = new Date().toISOString(), metadata = {}) {
			if (!this.isRecording()) return null
			const change = {
				type: 'audio-source-change', occurredAt,
				elapsedMs: Math.max(0, new Date(occurredAt).getTime() - new Date(this.initialSource.startedAt).getTime()),
				from: this._sourceMetadata(previousTrack, metadata.from || {}),
				to: this._sourceMetadata(nextTrack, metadata.to || {}),
			}
			this.sourceChanges.push(change)
			window.dispatchEvent(new CustomEvent('pore:recording-source-change', { detail: { sequence: this.sequence, change } }))
			return change
		}

		async stop(reason = 'host', { closingSignet = false } = {}) {
			if (!this.recorder || !this.isRecording()) return null
			this.state = 'stopping'
			try {
				if (closingSignet && typeof this.recorder.markClosingSignet === 'function') this.recorder.markClosingSignet()
				const artifact = await this.recorder.stop(reason)
				const enriched = artifact ? { ...artifact, sequence: this.sequence, source: { ...(this.initialSource || {}), ...(artifact.source || {}) }, sourceChanges: this.sourceChanges.slice() } : null
				this.recorder = null; this.currentTrack = null; this.state = 'idle'
				if (enriched) window.dispatchEvent(new CustomEvent('pore:recording-local-finalized', { detail: enriched }))
				return enriched
			} catch (error) {
				this.recorder = null; this.currentTrack = null; this.state = 'error'; throw error
			} finally {
				if (this.state === 'idle' || this.state === 'error') { this.captureId = null; this.recordingSessionId = null }
			}
		}

		_sourceMetadata(track, metadata = {}) {
			const settings = track?.getSettings?.() || {}
			return {
				trackId: track?.id || null, trackLabel: track?.label || null,
				deviceId: metadata.deviceId || settings.deviceId || null,
				productionId: metadata.productionId || null, productionLabel: metadata.productionLabel || null,
				recordingId: metadata.recordingId || null, captureId: metadata.captureId || null,
				recordingSessionId: metadata.recordingSessionId || null,
				sampleRate: Number.isFinite(settings.sampleRate) ? settings.sampleRate : null,
				sampleSize: Number.isFinite(settings.sampleSize) ? settings.sampleSize : null,
				channelCount: Number.isFinite(settings.channelCount) ? settings.channelCount : null,
				startedAt: metadata.startedAt || new Date().toISOString(),
			}
		}

		createPersistenceHandoff(artifact) {
			if (!artifact) return null
			const source = artifact.source || {}
			const required = ['productionId', 'recordingId', 'captureId', 'recordingSessionId']
			if (required.some(key => !source[key])) throw new Error('PoRE browser artifact is missing authoritative or technical identity')
			return {
				productionId: source.productionId, productionLabel: source.productionLabel || source.productionId,
				recordingId: source.recordingId, captureId: source.captureId, recordingSessionId: source.recordingSessionId,
				trackId: source.trackId || 'browser-track', sampleRate: artifact.sampleRate || source.sampleRate || null,
				channels: artifact.channels || source.channelCount || null, format: artifact.format || null,
				encoding: artifact.encoding || null, size: artifact.size || null, sequence: artifact.sequence || null,
				startedAt: artifact.startedAt || source.startedAt || null, stoppedAt: artifact.stoppedAt || null,
				stopReason: artifact.stopReason || null, openingSignet: artifact.openingSignet || source.openingSignet || null,
				closingSignet: artifact.closingSignet || source.closingSignet || null,
			}
		}
	}

	window.PoRELocalAudioCapture = PoRELocalAudioCapture

	// Prime the same recorder instance that the eventual capture will use.
	window.addEventListener('pointerdown', event => {
		const target = event.target
		if (!(target instanceof Element) || !target.closest('.pore-talk-recording__button')) return
		const controller = window.__poreTalkRecordingController
		if (!controller?.primeAudioContext) return
		void controller.primeAudioContext().catch(error => {
			window.dispatchEvent(new CustomEvent('pore:recording-error', { detail: { error } }))
		})
	}, true)

	window.PoREBrowserRecordingController = PoREBrowserRecordingController
})()
