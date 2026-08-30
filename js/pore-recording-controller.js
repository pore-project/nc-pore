/*
 * NC-PoRE — neutral browser recording boundary.
 *
 * This layer deliberately knows nothing about Nextcloud Talk. Host connectors
 * provide a MediaStreamTrack; this controller owns the browser-side recording
 * lifecycle and emits a finalized artifact event.
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
		}

		getState() {
			return this.state
		}

		isRecording() {
			return this.state === 'recording'
		}

		start(track) {
			if (this.isRecording()) {
				throw new Error('PoRE recording is already active')
			}
			if (!track || track.kind !== 'audio' || typeof track.clone !== 'function') {
				throw new Error('PoRE requires a live audio MediaStreamTrack')
			}
			if (track.readyState !== 'live') {
				throw new Error('PoRE cannot start from an ended audio track')
			}
			if (!window.MediaRecorder) {
				throw new Error('MediaRecorder is not available in this browser')
			}

			const mimeType = this._selectMimeType()
			this.stream = new MediaStream([track.clone()])
			this.chunks = []
			this.startedAt = new Date().toISOString()
			this.stoppedAt = null
			this.sequence += 1

			const options = mimeType ? { mimeType } : undefined
			try {
				this.mediaRecorder = new MediaRecorder(this.stream, options)
				this._bindRecorderEvents()
				this.mediaRecorder.start(1000)
				this.state = 'recording'
				window.dispatchEvent(new CustomEvent('pore:recording-started', {
					detail: {
						sequence: this.sequence,
						startedAt: this.startedAt,
						mimeType: this.mediaRecorder.mimeType,
						trackId: track.id,
						trackLabel: track.label,
					},
				}))
			} catch (error) {
				this.state = 'error'
				this._cleanup()
				throw error
			}
		}

		stop(reason = 'host') {
			if (!this.mediaRecorder || !this.isRecording()) {
				return Promise.resolve(null)
			}

			this.state = 'stopping'
			this.stoppedAt = new Date().toISOString()

			return new Promise((resolve, reject) => {
				const recorder = this.mediaRecorder
				const finalize = () => {
					try {
						const blob = new Blob(this.chunks, { type: recorder.mimeType || 'audio/webm' })
						const artifact = {
							kind: 'audio',
							format: blob.type,
							size: blob.size,
							sequence: this.sequence,
							startedAt: this.startedAt,
							stoppedAt: this.stoppedAt,
							stopReason: reason,
							blob,
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
					window.dispatchEvent(new CustomEvent('pore:recording-error', {
						detail: { error: event?.error || event },
					}))
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
					window.dispatchEvent(new CustomEvent('pore:recording-chunk', {
						detail: { sequence: this.sequence, size: event.data.size },
					}))
				}
			})
		}

		_selectMimeType() {
			const candidates = [
				'audio/webm;codecs=opus',
				'audio/webm',
				'audio/ogg;codecs=opus',
			]
			return candidates.find(type => MediaRecorder.isTypeSupported(type)) || ''
		}

		_cleanup() {
			this.stream?.getTracks().forEach(track => track.stop())
			this.stream = null
			this.mediaRecorder = null
			this.chunks = []
		}
	}

	window.PoREBrowserRecordingController = PoREBrowserRecordingController
})()
