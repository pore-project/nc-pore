/*
 * NC-PoRE — neutral browser recording boundary.
 *
 * The recording controller consumes the independent PoRE capture supplied by
 * a host connector. It deliberately does not use MediaRecorder: browser codec
 * selection could turn the preservation master into a lossy stream.
 */

(() => {
	'use strict'

	const technicalId = prefix => {
		if (window.crypto?.randomUUID) return `${prefix}-${window.crypto.randomUUID()}`
		return `${prefix}-${Date.now()}-${Math.random().toString(16).slice(2)}`
	}

	class PoREBrowserRecordingController {
		constructor({ recorderFactory = () => new window.PoREBrowserPcmRecorder() } = {}) {
			this.recorderFactory = recorderFactory
			this.recorder = null
			this.state = 'idle'
			this.sequence = 0
			this.sourceChanges = []
			this.initialSource = null
			this.captureId = null
			this.recordingSessionId = null
		}

		getState() { return this.state }
		isRecording() { return this.state === 'recording' }

		async start(track, sourceMetadata = {}) {
			if (this.isRecording()) throw new Error('PoRE recording is already active')
			if (!track || track.kind !== 'audio') throw new Error('PoRE requires an owned audio MediaStreamTrack')
			if (track.readyState !== 'live') throw new Error('PoRE cannot start from an ended audio track')

			this.recorder = this.recorderFactory()
			this.sourceChanges = []
			this.sequence += 1
			this.captureId = sourceMetadata.captureId || technicalId('browser-capture')
			this.recordingSessionId = sourceMetadata.recordingSessionId || technicalId('browser-session')
			this.initialSource = this._sourceMetadata(track, {
				...sourceMetadata,
				captureId: this.captureId,
				recordingSessionId: this.recordingSessionId,
			})
			this.state = 'starting'

			try {
				await this.recorder.start(track, {
					...sourceMetadata,
					captureId: this.captureId,
					recordingSessionId: this.recordingSessionId,
				})
				this.state = 'recording'
				window.dispatchEvent(new CustomEvent('pore:recording-started', { detail: {
					sequence: this.sequence,
					source: this.initialSource,
				} }))
			} catch (error) {
				this.state = 'error'
				this.recorder = null
				this.captureId = null
				this.recordingSessionId = null
				throw error
			}
		}

		noteSourceChange(previousTrack, nextTrack, occurredAt = new Date().toISOString(), metadata = {}) {
			if (!this.isRecording()) return null
			const change = {
				type: 'audio-source-change',
				occurredAt,
				elapsedMs: Math.max(0, new Date(occurredAt).getTime() - new Date(this.initialSource.startedAt).getTime()),
				from: this._sourceMetadata(previousTrack, metadata.from || {}),
				to: this._sourceMetadata(nextTrack, metadata.to || {}),
			}
			this.sourceChanges.push(change)
			window.dispatchEvent(new CustomEvent('pore:recording-source-change', { detail: { sequence: this.sequence, change } }))
			return change
		}

		async stop(reason = 'host') {
			if (!this.recorder || !this.isRecording()) return null
			this.state = 'stopping'
			try {
				const artifact = await this.recorder.stop(reason)
				const enriched = artifact ? {
					...artifact,
					sequence: this.sequence,
					source: this.initialSource,
					sourceChanges: this.sourceChanges.slice(),
				} : null
				this.recorder = null
				this.state = 'idle'
				if (enriched) {
					window.dispatchEvent(new CustomEvent('pore:recording-local-finalized', { detail: enriched }))
				}
				return enriched
			} catch (error) {
				this.recorder = null
				this.state = 'error'
				throw error
			} finally {
				if (this.state === 'idle' || this.state === 'error') {
					this.captureId = null
					this.recordingSessionId = null
				}
			}
		}

		_sourceMetadata(track, metadata = {}) {
			const settings = track?.getSettings?.() || {}
			return {
				trackId: track?.id || null,
				trackLabel: track?.label || null,
				deviceId: metadata.deviceId || settings.deviceId || null,
				productionId: metadata.productionId || null,
				productionLabel: metadata.productionLabel || null,
				recordingId: metadata.recordingId || null,
				captureId: metadata.captureId || null,
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
			if (required.some(key => !source[key])) {
				throw new Error('PoRE browser artifact is missing authoritative or technical identity')
			}
			if (!(artifact.blob instanceof Blob)) throw new Error('PoRE browser artifact has no payload Blob')

			return {
				productionId: source.productionId,
				productionLabel: source.productionLabel || source.productionId,
				recordingId: source.recordingId,
				captureId: source.captureId,
				recordingSessionId: source.recordingSessionId,
				trackId: source.trackId || 'browser-track',
				sampleRate: artifact.sampleRate || source.sampleRate || null,
				channels: artifact.channels || source.channelCount || null,
				format: artifact.format || null,
				encoding: artifact.encoding || null,
				size: artifact.size || artifact.blob.size,
				sequence: artifact.sequence || null,
				startedAt: artifact.startedAt || source.startedAt || null,
				stoppedAt: artifact.stoppedAt || null,
				stopReason: artifact.stopReason || null,
				blob: artifact.blob,
			}
		}
	}

	window.PoREBrowserRecordingController = PoREBrowserRecordingController
})()
