/*
 * NC-PoRE — Talk-native recording control surface.
 *
 * Talk supplies the mount point and role/context. Core/Application supplies the
 * authoritative recording state. This module renders a compact Talk-like
 * control and keeps the recording action inside its own popover.
 */

(() => {
	'use strict'

	const STATUS = Object.freeze({
		preparing: { label: 'Vorbereitung', compact: 'PoRE', tone: 'preparing', symbol: '○' },
		listener: { label: 'Nicht beteiligt', compact: '', tone: 'listener', symbol: '•' },
		error: { label: 'Problem bei der Aufnahme', compact: 'Problem', tone: 'error', symbol: '!' },
		ready: { label: 'Aufnahme bereit', compact: 'Bereit', tone: 'ready', symbol: '●' },
		recording: { label: 'Aufnahme läuft', compact: 'Aufnahme', tone: 'recording', symbol: '●' },
		opening: { label: 'Aufnahme wird geöffnet', compact: 'Start …', tone: 'opening', symbol: '●' },
		stopping: { label: 'Aufnahme wird übertragen', compact: 'Transfer', tone: 'transfer', symbol: '↗' },
		stopped: { label: 'Aufnahme beendet', compact: 'Beendet', tone: 'stopped', symbol: '■' },
		confirmed: { label: 'Aufnahme bestätigt', compact: 'Bestätigt', tone: 'confirmed', symbol: '✓' },
		productionClosed: { label: 'Produktion geschlossen', compact: 'Geschlossen', tone: 'production-closed', symbol: '■' },
	})

	const SETTINGS_URL = '/ocs/v2.php/apps/pore/v1/settings'
	const DEFAULT_ROOT = 'audio'

	const formatElapsed = seconds => {
		const value = Math.max(0, Math.floor(Number(seconds) || 0))
		const minutes = Math.floor(value / 60)
		const remainder = value % 60
		return `${String(minutes).padStart(2, '0')}:${String(remainder).padStart(2, '0')}`
	}

	const elapsedSecondsFromStartedAt = (startedAt, now = Date.now(), fallback = 0) => {
		const timestamp = Date.parse(startedAt || '')
		if (!Number.isFinite(timestamp)) return Math.max(0, Number(fallback) || 0)
		return Math.max(0, (now - timestamp) / 1000)
	}

	const toUserFacingError = error => {
		const code = String(error?.code || error?.name || '').trim()
		if (code === 'NotAllowedError' || code === 'permission_denied') return 'Der Zugriff auf das Mikrofon wurde nicht freigegeben.'
		if (code === 'NotFoundError' || code === 'device_not_found') return 'Es wurde kein verwendbares Mikrofon gefunden.'
		if (code === 'talk_context_unauthorized') return 'Die Aufnahmefunktion ist in diesem Gespräch nicht verfügbar.'
		return 'Bei der Aufnahme ist ein Problem aufgetreten. Bitte den Vorgang erneut versuchen.'
	}

	const compactStatus = status => status?.compact || 'PoRE'

	const resolveStatus = ({ state = 'preparing', listener = false, ready = false, confirmed = false, productionStatus = null }) => {
		if (listener) return STATUS.listener
		if (confirmed && state === 'completed') return STATUS.confirmed
		if (productionStatus === 'completed' && state === 'stopped') return STATUS.productionClosed
		if (state === 'error') return STATUS.error
		if (state === 'stopped') return STATUS.stopped
		if (state === 'stopping') return STATUS.stopping
		if (state === 'opening') return STATUS.opening
		if (state === 'recording') return STATUS.recording
		if (ready) return STATUS.ready
		return STATUS.preparing
	}

	const ensureStyles = () => {
		if (document.getElementById('pore-talk-recording-ui-styles')) return
		const style = document.createElement('style')
		style.id = 'pore-talk-recording-ui-styles'
		style.textContent = `
			.pore-talk-recording-ui-mount { display: flex; align-items: center; flex: 0 0 auto; }
			.pore-talk-recording-ui-mount .pore-talk-recording__control { display: flex; align-items: center; }
			.pore-talk-recording-ui-mount .pore-talk-recording__main,
			.pore-talk-recording-ui-mount .pore-talk-recording__menu-toggle {
				box-sizing: border-box; height: 34px; min-width: 34px; border: 0;
				background: transparent; color: var(--color-main-text, #222);
				border-radius: var(--border-radius-large, 8px); cursor: pointer;
				display: inline-flex; align-items: center; justify-content: center;
			}
			.pore-talk-recording-ui-mount .pore-talk-recording__main:hover,
			.pore-talk-recording-ui-mount .pore-talk-recording__menu-toggle:hover {
				background: var(--color-background-hover, rgba(0,0,0,.08));
			}
			.pore-talk-recording-ui-mount .pore-talk-recording__main:focus-visible,
			.pore-talk-recording-ui-mount .pore-talk-recording__menu-toggle:focus-visible {
				outline: 2px solid var(--color-primary-element, #0082c9); outline-offset: -2px;
			}
			.pore-talk-recording-ui-mount .pore-talk-recording__main { gap: 6px; padding: 0 7px; }
			.pore-talk-recording-ui-mount .pore-talk-recording__main-label { white-space: nowrap; font-size: .85em; font-weight: 600; }
			.pore-talk-recording-ui-mount .pore-talk-recording__main .pore-talk-recording__indicator { width: 18px; height: 18px; flex-basis: 18px; font-size: .75em; }
			.pore-talk-recording-ui-mount .pore-talk-recording__status-detail { margin: -6px 0 12px; color: var(--color-text-maxcontrast, #666); }
			.pore-talk-recording-ui-mount .pore-talk-recording__error { margin: 0 0 12px; padding: 8px 10px; border-radius: var(--border-radius-large, 8px); background: var(--color-error-hover, rgba(200,0,0,.08)); color: var(--color-error, #b40000); }
			.pore-talk-recording-ui-mount .pore-talk-recording__error[hidden] { display: none; }
			.pore-talk-recording-ui-mount .pore-talk-recording__menu-toggle { padding: 0; }
			.pore-talk-recording-ui-mount .pore-talk-recording__logo { width: 24px; height: 24px; display: block; }
			.pore-talk-recording-ui-mount .pore-talk-recording__chevron { width: 16px; height: 16px; }
			.pore-talk-recording-ui-mount .pore-talk-recording__panel {
				position: fixed; z-index: 10000; width: 300px; max-width: calc(100vw - 24px);
				padding: 16px; box-sizing: border-box; border-radius: 10px;
				background: var(--color-main-background, #fff); color: var(--color-main-text, #222);
				box-shadow: 0 8px 24px rgba(0,0,0,.22); border: 1px solid var(--color-border, #ccc);
			}
			.pore-talk-recording-ui-mount .pore-talk-recording__panel[hidden] { display: none; }
			.pore-talk-recording-ui-mount .pore-talk-recording__panel-title { margin: 0 0 12px; font-weight: 600; }
			.pore-talk-recording-ui-mount .pore-talk-recording__status { margin: 0 0 12px; }
			.pore-talk-recording-ui-mount .pore-talk-recording__readiness,
			.pore-talk-recording-ui-mount .pore-talk-recording__elapsed { display: block; margin-top: 4px; font-size: .9em; opacity: .75; }
			.pore-talk-recording-ui-mount .pore-talk-recording__button {
				width: 100%; min-height: 36px; padding: 7px 12px; border: 0;
				border-radius: var(--border-radius-large, 8px); background: var(--color-primary-element, #0082c9);
				color: var(--color-primary-element-text, #fff); cursor: pointer; font-weight: 600;
			}
			.pore-talk-recording-ui-mount .pore-talk-recording__settings { margin-top: 14px; padding-top: 12px; border-top: 1px solid var(--color-border, #ddd); }
			.pore-talk-recording-ui-mount .pore-talk-recording__settings label { display: block; margin-bottom: 5px; font-size: .9em; }
			.pore-talk-recording-ui-mount .pore-talk-recording__settings input { box-sizing: border-box; width: 100%; min-height: 34px; padding: 5px 8px; border: 1px solid var(--color-border, #bbb); border-radius: 6px; background: var(--color-main-background, #fff); color: var(--color-main-text, #222); }
			.pore-talk-recording-ui-mount .pore-talk-recording__settings-status { min-height: 1.2em; margin: 4px 0 0; font-size: .8em; opacity: .75; }
		`
		document.head.appendChild(style)
	}

	const createChevron = () => {
		const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg')
		svg.setAttribute('viewBox', '0 0 24 24')
		svg.setAttribute('aria-hidden', 'true')
		svg.setAttribute('class', 'pore-talk-recording__chevron')
		const path = document.createElementNS('http://www.w3.org/2000/svg', 'path')
		path.setAttribute('fill', 'currentColor')
		path.setAttribute('d', 'M7.41 15.41 12 10.83l4.59 4.58L18 14l-6-6-6 6z')
		svg.appendChild(path)
		return svg
	}

	const requestSettings = async (method, storageRoot = null) => {
		const options = { method, credentials: 'same-origin', headers: { Accept: 'application/json', 'OCS-APIRequest': 'true' } }
		if (method === 'PUT') {
			const form = new URLSearchParams()
			form.set('storageRoot', storageRoot ?? '')
			options.body = form
		}
		const response = await fetch(SETTINGS_URL, options)
		const payload = await response.json()
		if (!response.ok || payload?.ocs?.meta?.status !== 'ok') throw new Error(payload?.ocs?.meta?.message || 'Einstellungen konnten nicht gespeichert werden')
		return payload.ocs.data
	}

	const create = initialContext => {
		ensureStyles()

		const root = document.createElement('section')
		root.className = 'pore-talk-recording'

		const control = document.createElement('div')
		control.className = 'pore-talk-recording__control'

		const main = document.createElement('button')
		main.type = 'button'
		main.className = 'pore-talk-recording__main button-vue button-vue--size-normal button-vue--tertiary'
		const mainIndicator = document.createElement('span')
		mainIndicator.className = 'pore-talk-recording__indicator'
		mainIndicator.setAttribute('aria-hidden', 'true')
		const mainLabel = document.createElement('span')
		mainLabel.className = 'pore-talk-recording__main-label'
		main.append(mainIndicator, mainLabel)

		const toggle = document.createElement('button')
		toggle.type = 'button'
		toggle.className = 'pore-talk-recording__menu-toggle button-vue button-vue--size-normal button-vue--tertiary action-item__menutoggle'
		toggle.title = 'NC-PoRE öffnen'
		toggle.setAttribute('aria-label', 'NC-PoRE öffnen')
		toggle.setAttribute('aria-expanded', 'false')
		toggle.appendChild(createChevron())

		const panel = document.createElement('div')
		panel.className = 'pore-talk-recording__panel'
		panel.hidden = true

		const title = document.createElement('h3')
		title.className = 'pore-talk-recording__panel-title'
		title.id = 'pore-talk-recording-panel-title'
		title.textContent = 'NC-PoRE'
		panel.setAttribute('role', 'dialog')
		panel.setAttribute('aria-labelledby', title.id)
		panel.appendChild(title)

		const statusText = document.createElement('p')
		statusText.className = 'pore-talk-recording__status'
		statusText.setAttribute('role', 'status')
		statusText.setAttribute('aria-live', 'polite')
		panel.appendChild(statusText)

		const statusDetail = document.createElement('p')
		statusDetail.className = 'pore-talk-recording__status-detail'
		panel.appendChild(statusDetail)

		const errorText = document.createElement('p')
		errorText.className = 'pore-talk-recording__error'
		errorText.setAttribute('role', 'alert')
		panel.appendChild(errorText)

		const readiness = document.createElement('span')
		readiness.className = 'pore-talk-recording__readiness'
		panel.appendChild(readiness)

		const elapsed = document.createElement('span')
		elapsed.className = 'pore-talk-recording__elapsed'
		panel.appendChild(elapsed)

		const action = document.createElement('button')
		action.type = 'button'
		action.className = 'pore-talk-recording__button'
		let actionHandler = null
		let elapsedTimer = null

		const stopElapsedTimer = () => {
			if (elapsedTimer) window.clearInterval(elapsedTimer)
			elapsedTimer = null
		}

		const updateElapsed = ({ startedAt = null, elapsedSeconds = 0 } = {}) => {
			elapsed.textContent = formatElapsed(elapsedSecondsFromStartedAt(startedAt, Date.now(), elapsedSeconds))
		}

		const syncElapsedTimer = ({ state, listener, startedAt, elapsedSeconds }) => {
			if (state !== 'recording' || listener || !startedAt) {
				stopElapsedTimer()
				return
			}
			updateElapsed({ startedAt, elapsedSeconds })
			if (elapsedTimer) return
			elapsedTimer = window.setInterval(() => updateElapsed({ startedAt, elapsedSeconds }), 500)
		}
		action.addEventListener('click', event => {
			event.stopPropagation()
			if (typeof actionHandler === 'function') void actionHandler(event)
		})
		panel.appendChild(action)

		const settings = document.createElement('div')
		settings.className = 'pore-talk-recording__settings'
		settings.innerHTML = '<label for="pore-talk-storage-root">Speicherort</label><input id="pore-talk-storage-root" type="text" autocomplete="off" placeholder="audio"><p class="pore-talk-recording__settings-status" aria-live="polite"></p>'
		const input = settings.querySelector('input')
		const settingsStatus = settings.querySelector('.pore-talk-recording__settings-status')
		let guestMode = false
		input.addEventListener('blur', async () => {
			const value = input.value.trim()
			settingsStatus.textContent = ''
			input.disabled = true
			try {
				const saved = await requestSettings('PUT', value)
				input.value = saved.storage_root || ''
				settingsStatus.textContent = 'Gespeichert'
			} catch (error) {
				settingsStatus.textContent = error.message
			} finally {
				input.disabled = false
			}
		})
		settings.hidden = true
		panel.appendChild(settings)

		const positionPanel = () => {
			const rect = toggle.getBoundingClientRect()
			panel.style.left = `${Math.max(12, Math.min(window.innerWidth - 312, rect.right - 300))}px`
			panel.style.top = `${Math.max(12, rect.top - panel.offsetHeight - 8)}px`
		}
		const setOpen = open => {
			panel.hidden = !open
			toggle.setAttribute('aria-expanded', String(open))
			if (open) {
				if (!guestMode) {
					void requestSettings('GET').then(data => { input.value = data.storage_root || ''; input.placeholder = data.default_storage_root || DEFAULT_ROOT }).catch(() => {})
					settings.hidden = false
				} else {
					settings.hidden = true
				}
				requestAnimationFrame(positionPanel)
			}
		}

		toggle.addEventListener('click', event => { event.stopPropagation(); setOpen(panel.hidden) })
		main.addEventListener('click', event => { event.stopPropagation(); setOpen(!panel.hidden) })

		control.append(main, toggle)
		root.appendChild(control)
		root.appendChild(panel)

		const renderPanel = context => {
			const {
				role = 'none', state = 'preparing', listener = false, guest = false, ready = false, confirmed = false,
				readyCount = 0, openingConfirmedCount = 0, participantCount = 0, elapsedSeconds = 0,
				productionStatus = null, errorMessage = '', onStart = null, onStop = null, onForceClose = null,
			} = context || {}
			guestMode = guest === true
			const wasOpen = !panel.hidden

			if (role === 'listener' || role === 'participant' || listener) {
				root.hidden = true
				root.setAttribute('aria-hidden', 'true')
				panel.hidden = true
				toggle.setAttribute('aria-expanded', 'false')
				stopElapsedTimer()
				return
			}
			root.hidden = false
			root.setAttribute('aria-hidden', 'false')
			settings.hidden = guestMode
			const status = resolveStatus({ state, listener, ready, confirmed, productionStatus })
			root.dataset.status = status.tone
			root.setAttribute('aria-label', `NC-PoRE: ${status.label}`)
			mainIndicator.className = `pore-talk-recording__indicator pore-talk-recording__indicator--${status.tone}`
			mainIndicator.textContent = status.symbol
			mainLabel.textContent = compactStatus(status)
			main.setAttribute('aria-label', `NC-PoRE: ${status.label}`)
			main.title = status.label
			statusText.textContent = status.label
			statusDetail.textContent = ''
			errorText.hidden = !errorMessage
			errorText.textContent = errorMessage || ''

			const showReadiness = role === 'host' && participantCount > 0 && !listener && !confirmed
			readiness.hidden = !showReadiness
			if (showReadiness) {
				readiness.textContent = `${readyCount} / ${participantCount} bereit`
				if (openingConfirmedCount < participantCount) readiness.textContent += ` · Opening ${openingConfirmedCount} / ${participantCount}`
			}

			const showElapsed = state === 'recording' && !listener
			elapsed.hidden = !showElapsed
			if (showElapsed) updateElapsed({ startedAt: context?.startedAt, elapsedSeconds })
			syncElapsedTimer({ state, listener, startedAt: context?.startedAt, elapsedSeconds })

			if (state === 'stopped' && productionStatus === 'active' && !confirmed) {
				statusDetail.textContent = 'Aufnahme beendet – Verarbeitung läuft.'
			} else if (state === 'completed' && !confirmed) {
				statusDetail.textContent = 'Aufnahme ist technisch abgeschlossen, die Bestätigung steht noch aus.'
			} else if (state === 'opening') {
				statusDetail.textContent = participantCount > 1
					? `${openingConfirmedCount} / ${participantCount} bereit für die Aufnahme.`
					: 'Die Aufnahme wird gerade gestartet.'
			}

			const canStart = role === 'host' && !listener && state === 'preparing' && !ready && typeof onStart === 'function'
			const canStop = role === 'host' && !listener && state === 'recording' && typeof onStop === 'function'
			const canForceClose = role === 'host' && !listener && state === 'stopped' && productionStatus === 'active' && !confirmed && typeof onForceClose === 'function'
			actionHandler = canStart ? onStart : canStop ? onStop : canForceClose ? onForceClose : null
			action.hidden = !actionHandler
			if (canStart) action.textContent = 'Aufnahme starten'
			if (canStop) action.textContent = 'Aufnahme beenden'
			if (canForceClose) action.textContent = 'Produktion endgültig schließen'

			if (wasOpen) setOpen(true)
		}

		root.__poreUpdate = renderPanel
		renderPanel(initialContext)
		return root
	}

	const mount = context => {
		const mountElement = context?.mountElement instanceof Element ? context.mountElement : document.querySelector('[data-pore-talk-call-root]')
		if (!mountElement) return null
		const existing = mountElement.querySelector(':scope > .pore-talk-recording')
		if (existing?.__poreUpdate) {
			existing.__poreUpdate(context)
			return existing
		}
		const ui = create(context)
		mountElement.appendChild(ui)
		return ui
	}

	const closeOpenPanels = event => {
		if (event?.target?.closest?.('.pore-talk-recording')) return
		document.querySelectorAll('.pore-talk-recording__panel:not([hidden])').forEach(panel => { panel.hidden = true; panel.previousElementSibling?.querySelector('.pore-talk-recording__menu-toggle')?.setAttribute('aria-expanded', 'false') })
	}
	const repositionOpenPanels = () => {
		document.querySelectorAll('.pore-talk-recording__panel:not([hidden])').forEach(panel => {
			const root = panel.closest('.pore-talk-recording')
			const toggle = root?.querySelector('.pore-talk-recording__menu-toggle')
			if (!toggle) return
			const rect = toggle.getBoundingClientRect()
			panel.style.left = `${Math.max(12, Math.min(window.innerWidth - 312, rect.right - 300))}px`
			panel.style.top = `${Math.max(12, rect.top - panel.offsetHeight - 8)}px`
		})
	}
	if (!window.__poreTalkRecordingUiGlobalListeners) {
		document.addEventListener('click', closeOpenPanels, { capture: true })
		window.addEventListener('resize', repositionOpenPanels)
		window.__poreTalkRecordingUiGlobalListeners = true
	}

	window.PoRETalkRecordingUi = Object.freeze({ STATUS, formatElapsed, elapsedSecondsFromStartedAt, resolveStatus, toUserFacingError, compactStatus, create, mount })
})()
