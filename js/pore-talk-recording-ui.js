/*
 * NC-PoRE — Talk-native recording status/control surface.
 *
 * Talk supplies the mount point and role/context. This module only renders
 * PoRE's recording surface and forwards explicit user intent.
 */

(() => {
	'use strict'

	const STATUS = Object.freeze({
		preparing: { label: 'Vorbereitung', tone: 'preparing', symbol: '○' },
		listener: { label: 'Listener', tone: 'listener', symbol: '•' },
		error: { label: 'Nicht bereit', tone: 'error', symbol: '!' },
		ready: { label: 'Aufnahme bereit', tone: 'ready', symbol: '●' },
		recording: { label: 'Aufnahme läuft', tone: 'recording', symbol: '●' },
		opening: { label: 'Aufnahme wird geöffnet', tone: 'opening', symbol: '●' },
		stopping: { label: 'Aufnahme wird übertragen', tone: 'transfer', symbol: '↗' },
		confirmed: { label: 'Aufnahme bestätigt', tone: 'confirmed', symbol: '✓' },
	})

	const formatElapsed = seconds => {
		const value = Math.max(0, Math.floor(Number(seconds) || 0))
		const minutes = Math.floor(value / 60)
		const remainder = value % 60
		return `${String(minutes).padStart(2, '0')}:${String(remainder).padStart(2, '0')}`
	}

	const resolveStatus = ({ state = 'preparing', listener = false, ready = false, confirmed = false }) => {
		if (listener) return STATUS.listener
		if (confirmed) return STATUS.confirmed
		if (state === 'error') return STATUS.error
		if (state === 'stopping') return STATUS.stopping
		if (state === 'recording' && ready) return STATUS.recording
		if (state === 'opening' && ready) return STATUS.opening
		if (ready) return STATUS.ready
		return STATUS.preparing
	}

	const create = ({
		role = 'none',
		state = 'preparing',
		listener = false,
		ready = false,
		confirmed = false,
		readyCount = 0,
		participantCount = 0,
		elapsedSeconds = 0,
		onStart = null,
		onStop = null,
	}) => {
		const status = resolveStatus({ state, listener, ready, confirmed })
		const root = document.createElement('section')
		root.className = 'pore-talk-recording'
		root.dataset.status = status.tone
		root.setAttribute('aria-label', `NC-PoRE: ${status.label}`)

		const indicator = document.createElement('span')
		indicator.className = `pore-talk-recording__indicator pore-talk-recording__indicator--${status.tone}`
		indicator.setAttribute('aria-hidden', 'true')
		indicator.textContent = status.symbol

		const content = document.createElement('div')
		content.className = 'pore-talk-recording__content'

		const label = document.createElement('strong')
		label.className = 'pore-talk-recording__label'
		label.textContent = status.label
		content.appendChild(label)

		if (role === 'host' && participantCount > 0 && !listener && !confirmed) {
			const readiness = document.createElement('span')
			readiness.className = 'pore-talk-recording__readiness'
			readiness.textContent = `${readyCount} / ${participantCount} bereit`
			content.appendChild(readiness)
		}

		if (state === 'recording' && !listener) {
			const elapsed = document.createElement('span')
			elapsed.className = 'pore-talk-recording__elapsed'
			elapsed.textContent = formatElapsed(elapsedSeconds)
			elapsed.setAttribute('aria-label', `Aufnahmedauer ${formatElapsed(elapsedSeconds)}`)
			content.appendChild(elapsed)
		}

		root.append(indicator, content)

		if (role === 'host' && !listener) {
			const controls = document.createElement('div')
			controls.className = 'pore-talk-recording__controls'

			if (state === 'ready' && readyCount === participantCount && onStart) {
				const start = document.createElement('button')
				start.type = 'button'
				start.className = 'pore-talk-recording__button'
				start.textContent = 'Aufnahme starten'
				start.addEventListener('click', onStart)
				controls.appendChild(start)
			}

			if (state === 'recording' && onStop) {
				const stop = document.createElement('button')
				stop.type = 'button'
				stop.className = 'pore-talk-recording__button pore-talk-recording__button--stop'
				stop.textContent = 'Aufnahme beenden'
				stop.addEventListener('click', onStop)
				controls.appendChild(stop)
			}

			if (controls.childElementCount > 0) root.appendChild(controls)
		}

		return root
	}

	const mount = context => {
		const root = context?.mountElement instanceof Element
			? context.mountElement
			: document.querySelector('[data-pore-talk-call-root]')

		if (!root) return null

		root.querySelector(':scope > .pore-talk-recording')?.remove()
		const ui = create(context)
		root.appendChild(ui)
		return ui
	}

	window.PoRETalkRecordingUi = Object.freeze({
		STATUS,
		formatElapsed,
		resolveStatus,
		create,
		mount,
	})
})()
