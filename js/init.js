/* NC-PoRe — Talk recording UI bootstrap. */
(() => {
	'use strict'
	const Connector = window.PoRETalkAudioCaptureConnector
	const Recorder = window.PoREBrowserRecordingController
	const trackEventName = window.PoRETalkAudioTrackEvent || 'pore:talk-audio-track'
	const captureErrorEventName = window.PoRETalkAudioCaptureErrorEvent || 'pore:talk-audio-capture-error'
	if (!Connector || !Recorder) return
	const connector = new Connector()
	const recorder = new Recorder()
	let currentTrack = null
	let currentSourceMetadata = null
	window.__poreTalkAudioConnector = connector
	window.__poreRecordingController = recorder
	const findControls = () => document.getElementById('pore-talk-recording-controls')
	const formatCaptureSettings = track => {
		const settings = track?.getSettings?.() || {}
		const parts = []
		if (Number.isFinite(settings.sampleRate)) parts.push(`${Math.round(settings.sampleRate / 1000)} kHz`)
		if (Number.isFinite(settings.sampleSize)) parts.push(`${settings.sampleSize} bit`)
		if (Number.isFinite(settings.channelCount)) parts.push(settings.channelCount === 1 ? 'Mono' : `${settings.channelCount} Kanäle`)
		return parts.join(' · ')
	}
	const setStatus = message => { const root = findControls(); if (root?.status) root.status.textContent = message }
	const refreshUi = () => {
		const root = findControls() || createControls()
		const recording = recorder.isRecording()
		const liveTrack = !!currentTrack && currentTrack.readyState === 'live'
		root.dataset.recording = recording ? 'true' : 'false'
		root.startButton.disabled = !liveTrack || recording
		root.startButton.hidden = recording
		root.stopButton.hidden = !recording
		root.status.textContent = recording ? 'Lokale Aufnahme läuft' : 'Keine lokale Aufnahme'
		const settings = formatCaptureSettings(currentTrack)
		root.source.textContent = liveTrack ? `Mikrofon: ${currentTrack.label || currentTrack.id || 'verbunden'}${settings ? ` · ${settings}` : ''}` : 'Mikrofon: wird gesucht'
	}
	const createControls = () => {
		const root = document.createElement('div')
		root.id = 'pore-talk-recording-controls'; root.className = 'pore-talk-recording-controls'; root.setAttribute('aria-label', 'NC-PoRe Aufnahme')
		const status = document.createElement('span'); status.className = 'pore-talk-recording-controls__status'; status.setAttribute('role', 'status')
		const source = document.createElement('span'); source.className = 'pore-talk-recording-controls__source'
		const startButton = document.createElement('button'); startButton.type = 'button'; startButton.className = 'pore-talk-recording-controls__button pore-talk-recording-controls__button--start primary'; startButton.textContent = 'Aufnahme starten'
		startButton.addEventListener('click', async () => { try { await recorder.start(currentTrack, currentSourceMetadata || {}); refreshUi() } catch (error) { setStatus(`Fehler: ${error?.message || error}`) } })
		const stopButton = document.createElement('button'); stopButton.type = 'button'; stopButton.className = 'pore-talk-recording-controls__button pore-talk-recording-controls__button--stop'; stopButton.textContent = 'Aufnahme beenden'; stopButton.hidden = true
		stopButton.addEventListener('click', () => recorder.stop('host').then(() => refreshUi()).catch(error => setStatus(`Fehler: ${error?.message || error}`)))
		root.append(status, source, startButton, stopButton); document.body.appendChild(root)
		root.startButton = startButton; root.stopButton = stopButton; root.status = status; root.source = source
		return root
	}
	const offerArtifact = artifact => {
		const url = URL.createObjectURL(artifact.blob); const link = document.createElement('a'); link.href = url; link.download = `pore-talk-${artifact.sequence}.wav`; link.textContent = `Aufnahme gespeichert (${Math.round(artifact.size / 1024)} kB)`; link.className = 'pore-talk-recording-result'
		link.addEventListener('click', () => window.setTimeout(() => URL.revokeObjectURL(url), 1000), { once: true }); document.body.appendChild(link); window.setTimeout(() => link.remove(), 30000); setStatus('Aufnahme abgeschlossen')
		console.log('PoRE: recording finalized', artifact)
	}
	window.addEventListener(trackEventName, event => {
		const nextTrack = event.detail?.track || null; const previousTrack = currentTrack; const replaced = previousTrack && nextTrack && nextTrack !== previousTrack
		const nextMetadata = { deviceId: event.detail?.deviceId || nextTrack?.getSettings?.()?.deviceId || null }
		if (replaced && recorder.isRecording()) {
			recorder.noteSourceChange(previousTrack, nextTrack, new Date().toISOString(), { from: currentSourceMetadata || {}, to: nextMetadata })
			setStatus('Mikrofon wurde gewechselt – Aufnahme wird abgeschlossen')
			recorder.stop('talk-track-replaced').catch(error => setStatus(`Fehler: ${error?.message || error}`))
		}
		currentTrack = nextTrack; currentSourceMetadata = nextMetadata; refreshUi()
	})
	window.addEventListener(captureErrorEventName, event => { if (recorder.isRecording()) recorder.stop('talk-capture-error').catch(() => {}); setStatus(`Mikrofon konnte für PoRE nicht geöffnet werden: ${event.detail?.error?.message || 'unbekannter Fehler'}`); refreshUi() })
	window.addEventListener('pore:recording-finalized', event => offerArtifact(event.detail))
	window.addEventListener('pore:recording-error', event => setStatus(`Fehler: ${event.detail?.error?.message || event.detail?.error || 'unbekannt'}`))
	const tryAttach = () => { if (connector.attachToTalk()) { refreshUi(); return } window.setTimeout(tryAttach, 100) }
	tryAttach()
})()
