/*
 * NC-PoRe Talk recording UI mount adapter.
 *
 * Talk does not currently expose a public extension point for active-call
 * top-bar actions. Keep the Talk-specific DOM dependency isolated here.
 */
(function () {
	'use strict'

	const MOUNT_SELECTOR = '.top-bar.top-bar--in-call .top-bar__controls'
	const ROOT_ATTRIBUTE = 'data-pore-talk-recording-ui'
	const POLL_INTERVAL_MS = 500

	let observer = null
	let pollTimer = null
	let mountedHost = null
	let mountedRoot = null

	function getMountHost() {
		return document.querySelector(MOUNT_SELECTOR)
	}

	function unmount() {
		if (mountedRoot?.parentNode) mountedRoot.remove()
		mountedRoot = null
		mountedHost = null
	}

	function mount() {
		if (!window.PoRETalkRecordingUi?.mount) return

		const host = getMountHost()
		if (!host) {
			unmount()
			return
		}

		if (mountedHost === host && mountedRoot?.isConnected) return

		unmount()

		const root = document.createElement('div')
		root.setAttribute(ROOT_ATTRIBUTE, 'true')
		root.className = 'pore-talk-recording-ui-mount'
		host.appendChild(root)

		try {
			window.PoRETalkRecordingUi.mount({ mountElement: root })
			mountedHost = host
			mountedRoot = root
			window.dispatchEvent(new CustomEvent('pore:recording-ui-mount', { detail: { mountElement: root } }))
		} catch (error) {
			root.remove()
			console.error('[NC-PoRe] Failed to mount Talk recording UI', error)
		}
	}

	function scheduleMount() {
		if (pollTimer !== null) return
		pollTimer = window.setTimeout(() => {
			pollTimer = null
			mount()
		}, POLL_INTERVAL_MS)
	}

	function start() {
		if (observer) return
		mount()
		observer = new MutationObserver(scheduleMount)
		observer.observe(document.body, { childList: true, subtree: true })
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

	window.PoRETalkRecordingUiMount = {
		start,
		stop,
		mount,
		unmount,
		getMountElement: () => mountedRoot,
	}

	if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', start, { once: true })
	else start()
})()
