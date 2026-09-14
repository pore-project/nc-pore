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
		preparing: { label: 'Vorbereitung', tone: 'preparing', symbol: '○' },
		listener: { label: 'Listener', tone: 'listener', symbol: '•' },
		error: { label: 'Nicht bereit', tone: 'error', symbol: '!' },
		ready: { label: 'Aufnahme bereit', tone: 'ready', symbol: '●' },
		recording: { label: 'Aufnahme läuft', tone: 'recording', symbol: '●' },
		opening: { label: 'Aufnahme wird geöffnet', tone: 'opening', symbol: '●' },
		stopping: { label: 'Aufnahme wird übertragen', tone: 'transfer', symbol: '↗' },
		confirmed: { label: 'Aufnahme bestätigt', tone: 'confirmed', symbol: '✓' },
	})

	const SETTINGS_URL = '/ocs/v2.php/apps/pore/v1/settings'
	const DEFAULT_ROOT = 'audio'

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
			.pore-talk-recording-ui-mount .pore-talk-recording__main { padding: 5px; }
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

	const createLogo = () => {
		const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg')
		svg.setAttribute('viewBox', '0 0 24 24')
		svg.setAttribute('aria-hidden', 'true')
		svg.className.baseVal = 'pore-talk-recording__logo'
		const path = document.createElementNS('http://www.w3.org/2000/svg', 'path')
		path.setAttribute('fill', 'currentColor')
		path.setAttribute('d', 'M12 .7a12 12 0 0 0-3.79 23.39c.6.11.82-.26.82-.58v-2.05c-3.34.73-4.04-1.61-4.04-1.61-.55-1.39-1.33-1.76-1.33-1.76-1.09-.75.08-.74.08-.74 1.2.08 1.84 1.23 1.84 1.23 1.07 1.83 2.81 1.3 3.5.99.11-.78.42-1.3.76-1.6-2.67-.3-5.47-1.34-5.47-5.95 0-1.31.47-2.38 1.23-3.22-.12-.3-.53-1.52.12-3.18 0 0 1-.32 3.3 1.23a11.5 11.5 0 0 1 6-.01c2.29-1.55 3.29-1.23 3.29-1.23.65 1.66.24 2.88.12 3.18.77.84 1.23 1.91 1.23 3.22 0 4.62-2.81 5.64-5.49 5.94.43.37.81 1.1.81 2.22v3.29c0 .32.22.69.83.57A12 12 0 0 0 12 .7Z')
		svg.appendChild(path)
		return svg
	}

	const createChevron = () => {
		const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg')
		svg.setAttribute('viewBox', '0 0 24 24')
		svg.setAttribute('aria-hidden', 'true')
		svg.className.baseVal = 'pore-talk-recording__chevron'
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
		main.title = 'NC-PoRE'
		main.setAttribute('aria-label', 'NC-PoRE')
		main.appendChild(createLogo())

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
		title.textContent = 'NC-PoRE'
		panel.appendChild(title)

		const statusText = document.createElement('p')
		statusText.className = 'pore-talk-recording__status'
		panel.appendChild(statusText)

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
				void requestSettings('GET').then(data => { input.value = data.storage_root || ''; input.placeholder = data.default_storage_root || DEFAULT_ROOT }).catch(() => {})
				settings.hidden = false
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
				role = 'none', state = 'preparing', listener = false, ready = false, confirmed = false,
				readyCount = 0, participantCount = 0, elapsedSeconds = 0, onStart = null, onStop = null,
			} = context || {}
			const status = resolveStatus({ state, listener, ready, confirmed })
			root.dataset.status = status.tone
			root.setAttribute('aria-label', `NC-PoRE: ${status.label}`)

			const wasOpen = !panel.hidden
			statusText.textContent = status.label

			const showReadiness = role === 'host' && participantCount > 0 && !listener && !confirmed
			readiness.hidden = !showReadiness
			if (showReadiness) readiness.textContent = `${readyCount} / ${participantCount} bereit`

			const showElapsed = state === 'recording' && !listener
			elapsed.hidden = !showElapsed
			if (showElapsed) elapsed.textContent = formatElapsed(elapsedSeconds)

			const canStart = role === 'host' && !listener && state === 'preparing' && !ready && typeof onStart === 'function'
			const canStop = role === 'host' && !listener && state === 'recording' && typeof onStop === 'function'
			actionHandler = canStart ? onStart : canStop ? onStop : null
			action.hidden = !actionHandler
			if (canStart) action.textContent = 'Aufnahme starten'
			if (canStop) action.textContent = 'Aufnahme beenden'

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

	window.PoRETalkRecordingUi = Object.freeze({ STATUS, formatElapsed, resolveStatus, create, mount })
})()
