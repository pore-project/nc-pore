/*
 * NC-PoRE — neutral browser recording boundary.
 *
 * This layer deliberately knows nothing about Nextcloud Talk. Host connectors
 * provide an owned MediaStreamTrack; this controller owns the browser-side
 * recording lifecycle and emits a finalized artifact event.
 *
 * The recording stop boundary is owned by the explicit PoRE recording-stop
 * action, not by Talk room termination.
 */

(() => {
	'use strict'

	class PoREBrowserRecordingController {
		constructor() {
			this.state = 'idle'
			this.mediaRecorder = null
			this.stream = null
			this.chunks = []
			this.startedAt = null
			this.stoppedAt = null
			this.sequence = 0
			this.sourceChanges = []
		}

		getState() { return this.state }
		isRecording() { return this.state === 'recording' }

		start(track) {
			if (this.isRecording()) throw new Error('PoRE recording is already active')
			if (!track || track.kind !== 'audio') throw new Error('PoRE requires an owned audio MediaStreamTrack')
			if (track.readyState !== 'live') throw new Error('PoRE cannot start from an ended audio track')
			if (!window.MediaRecorder) throw new Error('MediaRecorder is not available in this browser')

			const mimeType = this._selectMimeType()
			// The Talk connector already owns the clone. Do not clone it again here.
			this.stream = new MediaStream([track])
			this.chunks = []
			this.sourceChanges = []
			this.startedAt = new Date().toISOString()
			this.stoppedAt = null
			this.sequence += 1

			try {
				this.mediaRecorder = new MediaRecorder(this.stream, mimeType ? { mimeType } : undefined)
				this._bindRecorderEvents()
				this.mediaRecorder.start(1000)
				this.state = 'recording'
				window.dispatchEvent(new CustomEvent('pore:recording-started', { detail: {
					sequence: this.sequence,
					startedAt: this.startedAt,
					mimeType: this.mediaRecorder.mimeType,
					trackId: track.id,
					trackLabel: track.label,
				} }))
			} catch (error) {
				this.state = 'error'
				this._cleanup()
				throw error
			}
		}

		noteSourceChange(previousTrack, nextTrack, occurredAt = new Date().toISOString()) {
			if (!this.isRecording()) return null

			const change = {
				type: 'audio-source-change',
				occurredAt,
				elapsedMs: Math.max(0, new Date(occurredAt).getTime() - new Date(this.startedAt).getTime()),
				from: {
					trackId: previousTrack?.id || null,
					trackLabel: previousTrack?.label || null,
				},
				to: {
					trackId: nextTrack?.id || null,
					trackLabel: nextTrack?.label || null,
				},
			}
			this.sourceChanges.push(change)
			window.dispatchEvent(new CustomEvent('pore:recording-source-change', { detail: {
				sequence: this.sequence,
				change,
			} }))
			return change
		}

		stop(reason = 'host') {
			if (!this.mediaRecorder || !this.isRecording()) return Promise.resolve(null)

			this.state = 'stopping'
			this.stoppedAt = new Date().toISOString()

			return new Promise((resolve, reject) => {
				const recorder = this.mediaRecorder
				const finalize = () => {
					try {
						const blob = new Blob(this.chunks, { type: recorder.mimeType || 'audio/webm' })
						const artifact = {
							kind: 'audio', format: blob.type, size: blob.size, sequence: this.sequence,
							startedAt: this.startedAt, stoppedAt: this.stoppedAt, stopReason: reason, blob,
							sourceChanges: this.sourceChanges.slice(),
						}
						this._cleanup()
						this.state = 'idle'
						window.dispatchEvent(new CustomEvent('pore:recording-finalized', { detail: artifact }))
						resolve(artifact)
					} catch (error) {
						this.state = 'error'
						this._cleanup()
						reject(error)
					}
				}
				const fail = event => {
					this.state = 'error'
					this._cleanup()
					window.dispatchEvent(new CustomEvent('pore:recording-error', { detail: { error: event?.error || event } }))
					reject(event?.error || event)
				}
				recorder.addEventListener('stop', finalize, { once: true })
				recorder.addEventListener('error', fail, { once: true })
				recorder.stop()
			})
		}

		_bindRecorderEvents() {
			this.mediaRecorder.addEventListener('dataavailable', event => {
				if (event.data && event.data.size > 0) {
					this.chunks.push(event.data)
					window.dispatchEvent(new CustomEvent('pore:recording-chunk', { detail: { sequence: this.sequence, size: event.data.size } }))
				}
			})
		}

		_selectMimeType() {
			const candidates = ['audio/webm;codecs=opus', 'audio/webm', 'audio/ogg;codecs=opus']
			return candidates.find(type => MediaRecorder.isTypeSupported(type)) || ''
		}

		_cleanup() {
			this.stream?.getTracks().forEach(track => track.stop())
			this.stream = null
			this.mediaRecorder = null
			this.chunks = []
			this.sourceChanges = []
		}
	}

	window.PoREBrowserRecordingController = PoREBrowserRecordingController
})()
