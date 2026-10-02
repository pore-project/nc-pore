/*
 * NC-PoRe Talk participant recording-status presentation.
 *
 * This module renders presentation only. It does not know how recording state
 * is produced and it exposes no recording commands.
 */
(function () {
	'use strict'

	const POPOVER_ID = 'pore-talk-participant-status-popover'
	const DETAIL_MARGIN = 8
	let popover = null
	let popoverAnchor = null
	let repositionHandler = null
	const details = new WeakMap()

	const fallbackStatuses = {
		preparing: { label: 'Vorbereitung', tone: 'preparing', symbol: '○' },
		listener: { label: 'Nicht beteiligt', tone: 'listener', symbol: '•' },
		error: { label: 'Problem bei der Aufnahme', tone: 'error', symbol: '!' },
		ready: { label: 'Aufnahme bereit', tone: 'ready', symbol: '●' },
		recording: { label: 'Aufnahme läuft', tone: 'recording', symbol: '●' },
		opening: { label: 'Aufnahme wird geöffnet', tone: 'opening', symbol: '●' },
		stopping: { label: 'Aufnahme wird übertragen', tone: 'transfer', symbol: '↗' },
		stopped: { label: 'Aufnahme beendet', tone: 'stopped', symbol: '■' },
		confirmed: { label: 'Aufnahme bestätigt', tone: 'confirmed', symbol: '✓' },
		productionClosed: { label: 'Produktion geschlossen', tone: 'production-closed', symbol: '■' },
	}

	const resolveStatus = ({ state = 'preparing', ready = false, listener = false, confirmed = false, productionStatus = null } = {}) => {
		const Ui = window.PoRETalkRecordingUi
		if (Ui?.resolveStatus) return Ui.resolveStatus({ state, ready, listener, confirmed, productionStatus })
		if (listener) return fallbackStatuses.listener
		if (confirmed && state === 'completed') return fallbackStatuses.confirmed
		if (state === 'error') return fallbackStatuses.error
		if (state === 'stopping') return fallbackStatuses.stopping
		if (state === 'stopped') return fallbackStatuses.stopped
		if (state === 'completed') return fallbackStatuses.confirmed
		if (state === 'opening') return fallbackStatuses.opening
		if (state === 'recording') return fallbackStatuses.recording
		if (ready) return fallbackStatuses.ready
		return fallbackStatuses.preparing
	}

	const getDetail = ({ state, productionStatus, errorMessage } = {}) => {
		if (errorMessage) return errorMessage
		if (state === 'stopped' && productionStatus === 'active') return 'Aufnahme beendet – Verarbeitung läuft.'
		if (state === 'opening') return 'Die Aufnahme wird gerade gestartet.'
		if (state === 'completed') return 'Die Aufnahme ist abgeschlossen und serverseitig bestätigt.'
		if (state === 'ready') return 'Bereit für die Aufnahme.'
		return ''
	}

	const positionPopover = () => {
		if (!popover || !popoverAnchor?.isConnected) return
		const rect = popoverAnchor.getBoundingClientRect()
		const viewportPadding = 8
		const popoverRect = popover.getBoundingClientRect()
		const left = Math.min(
			Math.max(viewportPadding, rect.right - popoverRect.width),
			Math.max(viewportPadding, window.innerWidth - popoverRect.width - viewportPadding),
		)
		const top = Math.min(
			rect.bottom + DETAIL_MARGIN,
			Math.max(viewportPadding, window.innerHeight - popoverRect.height - viewportPadding),
		)
		popover.style.left = Math.round(left) + 'px'
		popover.style.top = Math.round(top) + 'px'
	}

	const hidePopover = () => {
		if (popover) popover.hidden = true
		if (popoverAnchor) popoverAnchor.removeAttribute('aria-describedby')
		popoverAnchor = null
		if (repositionHandler) {
			window.removeEventListener('scroll', repositionHandler, true)
			window.removeEventListener('resize', repositionHandler)
			repositionHandler = null
		}
	}

	const showPopover = (anchor, text) => {
		if (!text) return
		hidePopover()
		if (!popover) {
			popover = document.createElement('div')
			popover.id = POPOVER_ID
			popover.className = 'pore-talk-participant-status__popover'
			popover.setAttribute('role', 'tooltip')
			document.body.appendChild(popover)
		}
		if (popoverAnchor && popoverAnchor !== anchor) popoverAnchor.removeAttribute('aria-describedby')
		popoverAnchor = anchor
		popover.textContent = text
		popover.hidden = false
		anchor.setAttribute('aria-describedby', POPOVER_ID)
		positionPopover()
		repositionHandler = () => positionPopover()
		window.addEventListener('scroll', repositionHandler, true)
		window.addEventListener('resize', repositionHandler)
	}

	const update = (element, { state = 'preparing', ready = false, listener = false, confirmed = false, productionStatus = null, errorMessage = '' } = {}) => {
		if (!element) return
		const status = resolveStatus({ state, ready, listener, confirmed, productionStatus })
		const detail = getDetail({ state, productionStatus, errorMessage })
		const accessibleLabel = detail ? status.label + '. ' + detail : status.label

		element.dataset.tone = status.tone
		element.setAttribute('aria-label', accessibleLabel)
		element.title = accessibleLabel
		element.firstElementChild?.setAttribute('data-symbol', status.symbol)
		details.set(element, detail)

		return status
	}

	const create = (context = {}) => {
		const element = document.createElement('span')
		element.className = 'pore-talk-participant-status'
		element.setAttribute('data-pore-talk-participant-status', 'true')
		element.setAttribute('role', 'img')
		element.tabIndex = 0

		const dot = document.createElement('span')
		dot.className = 'pore-talk-participant-status__dot'
		dot.setAttribute('aria-hidden', 'true')
		element.appendChild(dot)

		const show = () => showPopover(element, details.get(element) || element.getAttribute('aria-label') || '')
		const hide = () => hidePopover()
		element.addEventListener('mouseenter', show)
		element.addEventListener('mouseleave', hide)
		element.addEventListener('focus', show)
		element.addEventListener('blur', hide)
		update(element, context)
		return element
	}

	const updateExisting = (element, context) => {
		update(element, context)
		if (document.activeElement === element) showPopover(element, details.get(element) || element.getAttribute('aria-label') || '')
	}

	const destroy = element => {
		if (!element) return
		if (popoverAnchor === element) hidePopover()
		element.remove()
	}

	window.PoRETalkParticipantStatus = Object.freeze({
		resolveStatus,
		getDetail,
		create,
		update,
		updateExisting,
		destroy,
		hidePopover,
	})
})()
