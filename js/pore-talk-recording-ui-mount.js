/*
 * NC-PoRe Talk recording UI mount adapter.
 *
 * Talk does not currently expose a public extension point for active-call
 * top-bar actions. Keep the Talk-specific DOM dependency isolated here.
 */
(function () {
	'use strict'

	const MOUNT_SELECTOR = '.top-bar__controls'
	const ROOT_ATTRIBUTE = 'data-pore-talk-recording-ui'
	const CALL_ROOT_SELECTOR = '[data-pore-talk-call-root]'
	const POLL_INTERVAL_MS = 500

	let observer = null
	let pollTimer = null
	let mountedHost = null
	let mountedRoot = null

	function getMountHost() {
		const explicitRoot = document.querySelector(CALL_ROOT_SELECTOR)
		if (explicitRoot) {
			return explicitRoot.querySelector(MOUNT_SELECTOR) || explicitRoot
		}
		return document.querySelector(MOUNT_SELECTOR)
	}

	function unmount() {
		if (mountedRoot && window.PoRETalkRecordingUi?.unmount) {
			try {
				window.PoRETalkRecordingUi.unmount(mountedRoot)
			} catch (error) {
				console.warn('[NC-PoRe] Failed to unmount Talk recording UI', error)
			}
		}
		if (mountedRoot?.parentNode) {
			mountedRoot.remove()
		}
		mountedRoot = null
		mountedHost = null
	}

	function mount() {
		if (!window.PoRETalkRecordingUi?.mount) {
			return
		}

		const host = getMountHost()
		if (!host) {
			unmount()
			return
		}

		if (mountedHost === host && mountedRoot?.isConnected) {
			return
		}

		unmount()

		const root = document.createElement('div')
		root.setAttribute(ROOT_ATTRIBUTE, 'true')
		root.className = 'pore-talk-recording-ui-mount'
		host.appendChild(root)

		try {
			window.PoRETalkRecordingUi.mount({
			mountElement: root,
		})
			mountedHost = host
			mountedRoot = root
		} catch (error) {
			root.remove()
			console.error('[NC-PoRe] Failed to mount Talk recording UI', error)
		}
	}

	function scheduleMount() {
		if (pollTimer !== null) {
			return
		}
		pollTimer = window.setTimeout(() => {
			pollTimer = null
			mount()
		}, POLL_INTERVAL_MS)
	}

	function start() {
		mount()
		observer = new MutationObserver(scheduleMount)
		observer.observe(document.body, {
			childList: true,
			subtree: true,
		})
	}

	function stop() {
		if (observer) {
			observer.disconnect()
			observer = null
		}
		if (pollTimer !== null) {
			window.clearTimeout(pollTimer)
			pollTimer = null
		}
		unmount()
	}

	window.PoRETalkRecordingUiMount = { start, stop, mount, unmount }

	if (document.readyState === 'loading') {
		document.addEventListener('DOMContentLoaded', start, { once: true })
	} else {
		start()
	}
})()
