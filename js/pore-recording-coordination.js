/* NC-PoRE — host-neutral browser transport for distributed recording coordination. */
(() => {
	'use strict'

	const PUBLISH_API_VERSION = '/ocs/v2.php/apps/pore/v1/recordings/coordination/publish'
	const EVENTS_API_VERSION = '/apps/pore/v1/recordings/coordination/events'
	const PROTOCOL_VERSION = 1
	const EVENT_NAME = 'pore-recording'
	const DEFAULT_READY_TIMEOUT_MS = 5000
	const TALK_SESSION_TAB_ID = 'x-nextcloud-talk-session-tab-id'

	const url = path => window.OC?.generateUrl ? window.OC.generateUrl(path) : path

	const getTalkSessionTabId = () => {
		try {
			const value = window.sessionStorage?.getItem(TALK_SESSION_TAB_ID) || ''
			return /^[A-Za-z0-9]{64}$/.test(value) ? value : null
		} catch (_) {
			return null
		}
	}

	class PoRERecordingCoordinationChannel {
		constructor() {
			this.source = null
			this.sessionId = null
			this.recordingId = null
			this.readyPromise = null
			this.resolveReady = null
			this.connected = false
		}

		connect(sessionId, recordingId) {
			if (!sessionId || !recordingId) return Promise.reject(new Error('PoRE recording coordination identity is incomplete'))
			if (this.sessionId === sessionId && this.recordingId === recordingId && this.source) {
				if (this.source.readyState === 1 && this.connected) return Promise.resolve()
				return this.readyPromise || Promise.reject(new Error('PoRE recording coordination transport is not ready'))
			}

			this.disconnect()
			this.sessionId = sessionId
			this.recordingId = recordingId
			this.readyPromise = new Promise(resolve => { this.resolveReady = resolve })

			if (typeof window.EventSource !== 'function') {
				const error = new Error('PoRE recording coordination transport requires EventSource')
				this.readyPromise = Promise.reject(error)
				this.resolveReady = null
				return this.readyPromise
			}

			const query = new URLSearchParams({ sessionId, recordingId })
			const source = new window.EventSource(url(EVENTS_API_VERSION) + '?' + query.toString())
			this.source = source

			source.addEventListener('open', () => {
				const wasConnected = this.connected
				this.connected = true
				this.resolveReady?.()
				this.resolveReady = null
				window.dispatchEvent(new CustomEvent('pore:recording-coordination-ready', {
					detail: { sessionId: this.sessionId, recordingId: this.recordingId, reconnected: wasConnected },
				}))
			})

			source.addEventListener(EVENT_NAME, event => {
				let signal
				try {
					signal = JSON.parse(event.data)
				} catch (error) {
					window.dispatchEvent(new CustomEvent('pore:recording-coordination-error', { detail: { error } }))
					return
				}
				if (signal?.version !== PROTOCOL_VERSION) return
				if (signal.sessionId !== this.sessionId || signal.recordingId !== this.recordingId) return
				window.dispatchEvent(new CustomEvent('pore:recording-signal', {
					detail: { ...signal, from: signal.actorId || null },
				}))
			})

			source.addEventListener('error', () => {
				const wasConnected = this.connected
				this.connected = false
				if (wasConnected) {
					this.readyPromise = new Promise(resolve => { this.resolveReady = resolve })
					window.dispatchEvent(new CustomEvent('pore:recording-coordination-lost', {
						detail: { sessionId: this.sessionId, recordingId: this.recordingId },
					}))
				}
			})

			return this.readyPromise
		}

		async waitUntilReady(timeoutMs = DEFAULT_READY_TIMEOUT_MS) {
			if (this.source?.readyState === 1 && this.connected) return
			if (!this.readyPromise) throw new Error('PoRE recording coordination transport is not initialized')
			const ready = this.readyPromise
			let timer = null
			try {
				await Promise.race([
					ready,
					new Promise((_, reject) => {
						timer = window.setTimeout(() => reject(new Error('PoRE recording coordination transport did not become ready')), timeoutMs)
					}),
				])
			} finally {
				if (timer !== null) window.clearTimeout(timer)
			}
		}

		async publish(type) {
			if (!this.sessionId || !this.recordingId) throw new Error('PoRE recording coordination transport is not initialized')
			if (!type) throw new Error('PoRE recording coordination event type is required')
			const params = new URLSearchParams({
				sessionId: this.sessionId,
				recordingId: this.recordingId,
				eventType: type,
				requestId: window.crypto?.randomUUID ? window.crypto.randomUUID() : String(Date.now()) + '-' + String(Math.random()),
			})
			const response = await fetch(url(PUBLISH_API_VERSION), {
				method: 'POST',
				credentials: 'same-origin',
				headers: {
					Accept: 'application/json',
					'OCS-APIRequest': 'true',
					'Content-Type': 'application/x-www-form-urlencoded; charset=UTF-8',
					...(window.OC?.requestToken ? { requesttoken: window.OC.requestToken } : {}),
					...(getTalkSessionTabId() ? { [TALK_SESSION_TAB_ID]: getTalkSessionTabId() } : {}),
				},
				body: params,
			})
			let body = null
			try { body = await response.json() } catch (_) {}
			if (!response.ok || body?.ocs?.meta?.status !== 'ok') {
				const error = new Error(body?.ocs?.data?.error_code || 'PoRE coordination publish failed (' + response.status + ')')
				error.code = body?.ocs?.data?.error_code || null
				error.status = response.status
				throw error
			}
			return body.ocs.data
		}

		disconnect() {
			if (this.source) this.source.close()
			this.source = null
			this.sessionId = null
			this.recordingId = null
			this.connected = false
			this.readyPromise = null
			this.resolveReady = null
		}
	}

	window.PoRERecordingCoordinationChannel = PoRERecordingCoordinationChannel
	window.__poreRecordingCoordinationChannel = new PoRERecordingCoordinationChannel()
})()

