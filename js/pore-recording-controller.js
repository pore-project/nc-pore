/*
 * NC-PoRE — neutral browser recording boundary.
 *
 * The recording controller consumes the independent PoRE capture supplied by
 * a host connector. It deliberately does not use MediaRecorder: browser codec
 * selection could turn the preservation master into a lossy stream.
 */

(() => {
	'use strict'

	class PoREBrowserRecordingController {
		constructor({ recorderFactory = () => new window.PoREBrowserPcmRecorder() } = {}) {
			this.recorderFactory = recorderFactory
			this.recorder = null
			this.state = 'idle'
			this.sequence = 0
			this.sourceChanges = []
			this.initialSource = null
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
			this.initialSource = this._sourceMetadata(track, sourceMetadata)
			this.state = 'starting'

			try {
				await this.recorder.start(track)
				this.state = 'recording'
				window.dispatchEvent(new CustomEvent('pore:recording-started', { detail: {
					sequence: this.sequence,
					source: this.initialSource,
				} }))
			} catch (error) {
				this.state = 'error'
				this.recorder = null
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
			}
		}

		_sourceMetadata(track, metadata = {}) {
			const settings = track?.getSettings?.() || {}
			return {
				trackId: track?.id || null,
				trackLabel: track?.label || null,
				deviceId: metadata.deviceId || settings.deviceId || null,
				productionId: metadata.productionId || null,
				sampleRate: Number.isFinite(settings.sampleRate) ? settings.sampleRate : null,
				sampleSize: Number.isFinite(settings.sampleSize) ? settings.sampleSize : null,
				channelCount: Number.isFinite(settings.channelCount) ? settings.channelCount : null,
				startedAt: this._startTime || new Date().toISOString(),
			}
		}
	}

	window.PoREBrowserRecordingController = PoREBrowserRecordingController
})()
