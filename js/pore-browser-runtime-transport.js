/* NC-PoRE — browser-to-Nextcloud artifact transport adapter. */
(() => {
	'use strict'

	class PoREBrowserRuntimeTransport {
		constructor({ completionJob = window.__poreBrowserCompletionJob } = {}) {
			this.completionJob = completionJob
			this.active = new Set()
			window.addEventListener('pore:recording-transport-ready', event => {
				void this.submit(event.detail).catch(error => {
					window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } }))
				})
			})
		}

		async submit(descriptor) {
			if (!descriptor?.blob) throw new Error('PoRE transport requires a finalized payload')
			if (!descriptor.captureId || !descriptor.recordingSessionId || !descriptor.productionId || !descriptor.recordingId) {
				throw new Error('PoRE transport requires authoritative and technical identities')
			}
			if (this.active.has(descriptor.captureId)) return null
			this.active.add(descriptor.captureId)

			const requestId = window.crypto?.randomUUID
				? `runtime-${window.crypto.randomUUID()}`
				: `runtime-${Date.now()}-${Math.random().toString(16).slice(2)}`
			const metadata = {
				request_id: requestId,
				capture_id: descriptor.captureId,
				recording_session_id: descriptor.recordingSessionId,
				production_id: descriptor.productionId,
				recording_id: descriptor.recordingId,
				track_id: descriptor.trackId || 'browser-track',
				sample_rate_hz: descriptor.sampleRate,
				channels: descriptor.channels,
				format: descriptor.format || 'audio/wav',
				encoding: descriptor.encoding || 'pcm_s24le',
				payload_sha256: descriptor.payloadSha256 || null,
			}

			try {
				const form = new FormData()
				form.append('metadata', JSON.stringify(metadata))
				form.append('payload', descriptor.blob, `${descriptor.captureId}.wav`)

				const url = window.OC?.generateUrl
					? window.OC.generateUrl('/ocs/v2.php/apps/pore/v1/recordings/finalized-artifact')
					: '/ocs/v2.php/apps/pore/v1/recordings/finalized-artifact'
				const response = await fetch(url, {
					method: 'POST',
					headers: {
						Accept: 'application/json',
						'OCS-APIRequest': 'true',
					},
					credentials: 'same-origin',
					body: form,
				})
				const body = await response.json()
				if (!response.ok || body?.ocs?.meta?.status !== 'ok' || body?.ocs?.data?.status !== 'stored') {
					throw new Error(body?.ocs?.data?.error_code || `PoRE transport failed (${response.status})`)
				}

				if (this.completionJob?.markCompleted) {
					await this.completionJob.markCompleted(descriptor.captureId, {
						requestId,
						artifactId: body.ocs.data.artifact_id,
						fileId: body.ocs.data.file_id,
						path: body.ocs.data.path,
						size: body.ocs.data.size,
						sha256: body.ocs.data.sha256,
						completedAt: new Date().toISOString(),
					})
				}
				window.dispatchEvent(new CustomEvent('pore:recording-transport-completed', { detail: body.ocs.data }))
				return body.ocs.data
			} finally {
				this.active.delete(descriptor.captureId)
			}
		}
	}

	window.PoREBrowserRuntimeTransport = PoREBrowserRuntimeTransport
})()
