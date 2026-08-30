/*
 * NC-PoRe — Talk recording UI bootstrap.
 *
 * The UI consumes the neutral track event from the Talk connector and delegates
 * recording lifecycle to the generic browser recording controller.
 *
 * ADR-074i: "Aufnahme beenden" is the PoRE stop boundary. Ending the Talk
 * room is deliberately not wired to recording stop.
 */

(() => {
	'use strict'

	const Connector = window.PoRETalkAudioCaptureConnector
	const Recorder = window.PoREBrowserRecordingController
	const trackEventName = window.PoRETalkAudioTrackEvent || 'pore:talk-audio-track'

	if (!Connector || !Recorder) {
		return
	}

	const connector = new Connector()
	const recorder = new Recorder()
	let currentTrack = null
	let currentSourceTrack = null

	window.__poreTalkAudioConnector = connector
	window.__poreRecordingController = recorder

	const isHost = () => {
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

	const findControls = () => document.getElementById('pore-talk-recording-controls')

	const refreshUi = () => {
		const existing = findControls()
		if (!isHost()) {
			existing?.remove()
			return
		}

		const root = existing || createControls()
		const recording = recorder.isRecording()
		const canStart = !!currentTrack && currentTrack.readyState === 'live' && !recording

		root.startButton.disabled = !canStart
		root.startButton.hidden = recording
		root.stopButton.hidden = !recording
		root.status.textContent = recording ? 'Aufnahme läuft' : 'Keine Aufnahme'
	}

	const createControls = () => {
		const root = document.createElement('div')
		root.id = 'pore-talk-recording-controls'
		root.style.cssText = 'position:fixed;right:24px;bottom:24px;z-index:100000;display:flex;align-items:center;gap:8px;padding:8px 10px;background:var(--color-main-background,#fff);border:1px solid var(--color-border,#bbb);border-radius:8px;box-shadow:0 4px 18px rgba(0,0,0,.18);'

		const startButton = document.createElement('button')
		startButton.type = 'button'
		startButton.textContent = 'Aufnahme starten'
		startButton.addEventListener('click', () => {
			if (!currentTrack) {
				setStatus('Kein aktiver Talk-Audio-Track verfügbar')
				return
			}
			try {
				recorder.start(currentTrack)
				refreshUi()
			} catch (error) {
				setStatus(`Fehler: ${error?.message || error}`)
			}
		})

		const stopButton = document.createElement('button')
		stopButton.type = 'button'
		stopButton.textContent = 'Aufnahme beenden'
		stopButton.addEventListener('click', () => {
			recorder.stop('host').then(() => refreshUi()).catch(error => setStatus(`Fehler: ${error?.message || error}`))
		})

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

	const setStatus = message => {
		const root = findControls()
		if (root?.status) {
			root.status.textContent = message
		}
	}

	const offerArtifact = artifact => {
		const url = URL.createObjectURL(artifact.blob)
		const link = document.createElement('a')
		link.href = url
		link.download = `pore-talk-${artifact.sequence}.webm`
		link.textContent = `Aufnahme gespeichert (${Math.round(artifact.size / 1024)} kB)`
		link.style.cssText = 'position:fixed;right:24px;bottom:78px;z-index:100001;padding:8px 10px;background:var(--color-primary-element,#0082c9);color:var(--color-primary-element-text,#fff);border-radius:6px;text-decoration:none;'
		link.addEventListener('click', () => window.setTimeout(() => URL.revokeObjectURL(url), 1000), { once: true })
		document.body.appendChild(link)
		window.setTimeout(() => link.remove(), 30000)
		setStatus('Aufnahme abgeschlossen')
		console.log('PoRE: recording finalized', {
			sequence: artifact.sequence,
			size: artifact.size,
			type: artifact.format,
			stopReason: artifact.stopReason,
		})
	}

	window.addEventListener(trackEventName, event => {
		const nextTrack = event.detail?.track || null
		const nextSourceTrack = event.detail?.sourceTrack || null
		const replaced = currentTrack && nextTrack && nextTrack !== currentTrack

		currentTrack = nextTrack
		currentSourceTrack = nextSourceTrack

		if (replaced && recorder.isRecording()) {
			setStatus('Talk-Audioquelle wurde ersetzt – Aufnahme wird abgeschlossen')
			recorder.stop('talk-track-replaced').catch(error => setStatus(`Fehler: ${error?.message || error}`))
		}

		refreshUi()
	})

	window.addEventListener('pore:recording-finalized', event => offerArtifact(event.detail))
	window.addEventListener('pore:recording-error', event => setStatus(`Fehler: ${event.detail?.error?.message || event.detail?.error || 'unbekannt'}`))

	const observer = new MutationObserver(() => refreshUi())
	observer.observe(document.documentElement, { childList: true, subtree: true })

	const tryAttach = () => {
		if (connector.attachToTalk()) {
			console.log('PoRE: ADR073 TrackEnabler sink attached')
			refreshUi()
			return
		}
		window.setTimeout(tryAttach, 100)
	}

	tryAttach()
})()
