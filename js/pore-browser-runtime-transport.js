/* NC-PoRE — browser-side Nextcloud transport adapter. */
(() => {
	'use strict'

	class PoREBrowserRuntimeTransport {
		constructor({ completionJob = window.__poreBrowserCompletionJob } = {}) {
			this.completionJob = completionJob
			this.active = new Set()
			window.addEventListener('pore:recording-transport-ready', event => {
				void this.transfer(event.detail).catch(error => {
					window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } }))
				})
			})
		}

		async transfer(descriptor) {
			if (!descriptor?.blob) throw new Error('PoRE transport requires a finalized payload')
			if (!descriptor.captureId || !descriptor.recordingSessionId || !descriptor.productionId || !descriptor.recordingId || !descriptor.startedAt) {
				throw new Error('PoRE transport requires authoritative identity and recording start time')
			}
			if (this.active.has(descriptor.captureId)) return null
			this.active.add(descriptor.captureId)

			try {
				let state = await this.completionJob?.getTransportState?.(descriptor.captureId)

				if (!state || state.status === 'failed' || state.status === 'pending' || (state.status === 'prepared' && !state.transferId)) {
					await this.prepare(descriptor)
					state = await this.completionJob.getTransportState(descriptor.captureId)
				}

				if (state.status === 'prepared') {
					await this.completionJob.updateTransportState(descriptor.captureId, {
						status: 'authorized',
						authorizedAt: new Date().toISOString(),
					})
					state = await this.completionJob.getTransportState(descriptor.captureId)
				}

				if (state.status === 'authorized') {
					if (!state.transferId || !state.uploadUrl || !state.uploadUsername || !state.uploadPassword || !state.filename) {
						throw new Error('PoRE transport authorization is incomplete')
					}
					await this.upload(state.uploadUrl, state.uploadUsername, state.uploadPassword, state.filename, descriptor.blob)
					await this.completionJob.updateTransportState(descriptor.captureId, {
						status: 'remote_present',
						remotePresentAt: new Date().toISOString(),
					})
				}

				state = await this.completionJob.getTransportState(descriptor.captureId)
				if (!state?.transferId) throw new Error('PoRE transport transfer handle is missing')

				if (state.status === 'remote_present') {
					const receipt = await this.verify(state.transferId)
					await this.completionJob.updateTransportState(descriptor.captureId, {
						status: 'verified',
						verifiedAt: new Date().toISOString(),
						receipt,
					})
				}

				state = await this.completionJob.getTransportState(descriptor.captureId)
				if (state.status === 'verified') {
					await this.close(state.transferId)
					await this.completionJob.updateTransportState(descriptor.captureId, {
						status: 'transport_closed',
						transportClosedAt: new Date().toISOString(),
					})
				}

				state = await this.completionJob.getTransportState(descriptor.captureId)
				if (state.status !== 'transport_closed' && state.status !== 'completed') {
					throw new Error('PoRE transport did not reach a closed verified state')
				}

				const details = state.receipt || {}
				await this.completionJob.markCompleted(descriptor.captureId, {
					completedAt: new Date().toISOString(),
					artifactId: details.artifact_id,
					fileId: details.file_id,
					path: details.path,
					size: details.size,
					sha256: details.sha256,
				})
				window.dispatchEvent(new CustomEvent('pore:recording-transport-completed', { detail: details }))
				return details
			} catch (error) {
				await this.completionJob?.updateTransportState?.(descriptor.captureId, {
					lastErrorAt: new Date().toISOString(),
					lastError: String(error?.message || error),
				}).catch(() => {})
				throw error
			} finally {
				this.active.delete(descriptor.captureId)
			}
		}

		async prepare(descriptor) {
			const form = new URLSearchParams()
			form.set('production_id', descriptor.productionId)
			form.set('production_label', descriptor.productionLabel || descriptor.productionId)
			form.set('recording_id', descriptor.recordingId)
			form.set('capture_id', descriptor.captureId)
			form.set('started_at', descriptor.startedAt)
			form.set('participant_label', descriptor.participantLabel || '')
			form.set('size', String(descriptor.size))
			form.set('payload_sha256', descriptor.payloadSha256)
			const body = await this.control('/ocs/v2.php/apps/pore/v1/recordings/finalized-artifact/prepare', form)
			if (body?.status !== 'prepared') throw new Error(body?.error_code || 'PoRE transport preparation failed')
			await this.completionJob.updateTransportState(descriptor.captureId, {
				status: 'prepared',
				transferId: body.transfer_id,
				uploadUrl: body.upload_url,
				uploadUsername: body.upload_username,
				uploadPassword: body.upload_password,
				filename: body.filename,
				preparedAt: new Date().toISOString(),
			})
			return body
		}

		async verify(transferId) {
			const form = new URLSearchParams()
			form.set('transfer_id', transferId)
			const body = await this.control('/ocs/v2.php/apps/pore/v1/recordings/finalized-artifact/verify', form)
			if (body?.status !== 'verified') throw new Error(body?.error_code || 'PoRE transport verification failed')
			return body
		}

		async close(transferId) {
			const form = new URLSearchParams()
			form.set('transfer_id', transferId)
			const body = await this.control('/ocs/v2.php/apps/pore/v1/recordings/finalized-artifact/close', form)
			if (body?.status !== 'closed') throw new Error(body?.error_code || 'PoRE transport close failed')
			return body
		}

		async upload(uploadUrl, username, password, filename, blob) {
			const url = window.OC?.generateUrl ? window.OC.generateUrl(uploadUrl) : uploadUrl
			const response = await fetch(`${url.replace(/\/$/, '')}/${encodeURIComponent(filename)}`, {
				method: 'PUT',
				headers: {
					Accept: '*/*',
					'X-Requested-With': 'XMLHttpRequest',
					Authorization: `Basic ${btoa(`${username}:${password}`)}`,
				},
				credentials: 'same-origin',
				body: blob,
			})
			if (!response.ok) throw new Error(`PoRE Nextcloud upload failed (${response.status})`)
		}

		async control(path, form) {
			const url = window.OC?.generateUrl ? window.OC.generateUrl(path) : path
			const response = await fetch(url, {
				method: 'POST',
				headers: {
					Accept: 'application/json',
					'OCS-APIRequest': 'true',
					'Content-Type': 'application/x-www-form-urlencoded;charset=UTF-8',
				},
				credentials: 'same-origin',
				body: form.toString(),
			})
			const envelope = await response.json()
			const body = envelope?.ocs?.data || envelope?.data || envelope
			if (!response.ok) throw new Error(body?.error_code || `PoRE transport control failed (${response.status})`)
			return body
		}
	}

	window.PoREBrowserRuntimeTransport = PoREBrowserRuntimeTransport
})()
