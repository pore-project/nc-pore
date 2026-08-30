/*
 * NC-PoRE — Nextcloud Talk host connector.
 *
 * ADR-073i: integrate at the currently active Talk TrackEnabler output.
 * ADR-074i: recording stop is an explicit host action; ending the Talk room
 * is not the recording boundary.
 */

(() => {
	'use strict'

	class PoRETalkAudioCaptureConnector {
		constructor() {
			this.trackEnabler = null
			this.currentTrack = null
			this.attached = false
			this.uiObserver = null
			this.recordingController = null
			this._outputTrackSetHandler = this._handleOutputTrackSet.bind(this)
			this._outputTrackEnabledHandler = this._handleOutputTrackEnabled.bind(this)
		}

		attachToTalk() {
			const webrtc = window.OCA?.Talk?.SimpleWebRTC?.webrtc
			const enabler = webrtc?._audioTrackEnabler
			const controller = window.PoREBrowserRecordingController

			if (!enabler || !controller) {
				return false
			}

			if (!this.attached) {
				this.trackEnabler = enabler
				this.recordingController = new controller()
				this._installSink()
				this._refreshTrack()
				this._installUiObserver()
				this.attached = true
				console.log('PoRE: Talk TrackEnabler integration ready')
			}

			this._refreshTrack()
			this._refreshUi()
			return !!this.currentTrack
		}

		/* TrackSource sink contract used by Talk's TrackEnabler.connectTrackSink(). */
		connectTrackSource(inputTrackId, trackSource, outputTrackId = 'default') {
			if (trackSource !== this.trackEnabler || outputTrackId !== 'default') {
				return
			}
			this._setCurrentTrack(trackSource.getOutputTrack(outputTrackId))
		}

		disconnectTrackSource(inputTrackId, trackSource, outputTrackId = 'default') {
			if (trackSource === this.trackEnabler && outputTrackId === 'default') {
				this._setCurrentTrack(null)
			}
		}

		on(event, handler) {
			if (event === 'outputTrackSet') {
				this._outputTrackSetHandler = handler
			}
			if (event === 'outputTrackEnabled') {
				this._outputTrackEnabledHandler = handler
			}
		}

		_handleOutputTrackSet(trackId, track) {
			if (trackId === 'default') {
				this._setCurrentTrack(track)
			}
		}

		_handleOutputTrackEnabled(trackId, enabled) {
			if (trackId === 'default' && this.currentTrack) {
				this.currentTrack.enabled = enabled
			}
		}

		_installSink() {
			/*
			 * Use Talk's own TrackSource/Sink wiring instead of replacing
			 * getUserMedia() or touching the RTCPeerConnection sender.
			 */
			this.trackEnabler.connectTrackSink('default', this, 'default')
			this.trackEnabler.on?.('outputTrackSet', this._outputTrackSetHandler)
			this.trackEnabler.on?.('outputTrackEnabled', this._outputTrackEnabledHandler)
		}

		_refreshTrack() {
			if (!this.trackEnabler) {
				return
			}
			try {
				this._setCurrentTrack(this.trackEnabler.getOutputTrack('default'))
			} catch {
				this._setCurrentTrack(null)
			}
		}

		_setCurrentTrack(track) {
			if (track && (track.kind !== 'audio' || track.readyState === 'ended')) {
				track = null
			}
			this.currentTrack = track || null
			window.dispatchEvent(new CustomEvent('pore:talk-audio-track-changed', {
				detail: {
					track: this.currentTrack,
					trackId: this.currentTrack?.id || null,
					trackLabel: this.currentTrack?.label || null,
				},
			}))
			this._refreshUi()
		}

		_installUiObserver() {
			this.uiObserver = new MutationObserver(() => this._refreshUi())
			this.uiObserver.observe(document.documentElement, { childList: true, subtree: true })
			window.addEventListener('pore:recording-started', () => this._refreshUi())
			window.addEventListener('pore:recording-finalized', event => this._offerArtifact(event.detail))
			window.addEventListener('pore:recording-error', event => this._showError(event.detail?.error))
		}

		_refreshUi() {
			if (!this.attached) {
				return
			}

			const host = this._isHost()
			const existing = document.getElementById('pore-talk-recording-controls')
			if (!host) {
				existing?.remove()
				return
			}

			const container = existing || this._createControls()
			const recording = this.recordingController?.isRecording() === true
			const canStart = !!this.currentTrack && this.currentTrack.readyState === 'live' && !recording

			container.startButton.disabled = !canStart
			container.startButton.hidden = recording
			container.stopButton.hidden = !recording
			container.status.textContent = recording ? 'Aufnahme läuft' : 'Keine Aufnahme'
		}

		_isHost() {
			const buttons = [...document.querySelectorAll('button, [role="button"]')]
			return buttons.some(button => {
				const text = [button.textContent, button.getAttribute('aria-label'), button.getAttribute('title')]
					.filter(Boolean).join(' ').toLowerCase()
				return text.includes('end meeting for everyone') ||
					text.includes('für alle beenden') ||
					text.includes('meeting für alle') ||
					text.includes('besprechung für alle') ||
					text.includes('anruf für alle')
			})
		}

		_createControls() {
			const root = document.createElement('div')
			root.id = 'pore-talk-recording-controls'
			root.style.cssText = 'position:fixed;right:24px;bottom:24px;z-index:100000;display:flex;align-items:center;gap:8px;padding:8px 10px;background:var(--color-main-background,#fff);border:1px solid var(--color-border,#bbb);border-radius:8px;box-shadow:0 4px 18px rgba(0,0,0,.18);'

			const startButton = document.createElement('button')
			startButton.type = 'button'
			startButton.textContent = 'Aufnahme starten'
			startButton.addEventListener('click', () => this._startRecording())

			const stopButton = document.createElement('button')
			stopButton.type = 'button'
			stopButton.textContent = 'Aufnahme beenden'
			stopButton.addEventListener('click', () => this._stopRecording())

			const status = document.createElement('span')
			status.setAttribute('role', 'status')
			status.textContent = 'Keine Aufnahme'

			root.append(startButton, stopButton, status)
			document.body.appendChild(root)
			root.startButton = startButton
			root.stopButton = stopButton
			root.status = status
			return root
		}

		_startRecording() {
			if (!this.currentTrack) {
				this._showError(new Error('Kein aktiver Talk-Audio-Track verfügbar'))
				return
			}
			try {
				this.recordingController.start(this.currentTrack)
				this._refreshUi()
			} catch (error) {
				this._showError(error)
			}
		}

		_stopRecording() {
			/*
			 * This is deliberately the PoRE recording boundary. Do not call
			 * webrtc.stop() and do not leave the Talk room here.
			 */
			this.recordingController.stop('host').catch(error => this._showError(error))
		}

		_offerArtifact(artifact) {
			const url = URL.createObjectURL(artifact.blob)
			const link = document.createElement('a')
			link.href = url
			link.download = `pore-talk-${artifact.sequence}.webm`
			link.textContent = `Aufnahme gespeichert (${Math.round(artifact.size / 1024)} kB)`
			link.style.cssText = 'position:fixed;right:24px;bottom:78px;z-index:100001;padding:8px 10px;background:var(--color-primary-element,#0082c9);color:var(--color-primary-element-text,#fff);border-radius:6px;text-decoration:none;'
			link.addEventListener('click', () => window.setTimeout(() => URL.revokeObjectURL(url), 1000), { once: true })
			document.body.appendChild(link)
			window.setTimeout(() => link.remove(), 30000)
			this._refreshUi()
			console.log('PoRE: recording finalized', {
				sequence: artifact.sequence,
				size: artifact.size,
				type: artifact.format,
				stopReason: artifact.stopReason,
			})
		}

		_showError(error) {
			console.error('PoRE: recording error', error)
			const root = document.getElementById('pore-talk-recording-controls')
			if (root?.status) {
				root.status.textContent = `Fehler: ${error?.message || error || 'unbekannt'}`
			}
		}
	}

	window.PoRETalkAudioCaptureConnector = PoRETalkAudioCaptureConnector
})()
