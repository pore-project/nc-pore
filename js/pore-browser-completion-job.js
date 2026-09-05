/* NC-PoRE — durable browser completion job. */
(() => {
	'use strict'

	class PoREBrowserCompletionJob {
		constructor({ persistenceStoreFactory = () => new window.PoREBrowserPcmPersistenceStore() } = {}) {
			this.persistenceStoreFactory = persistenceStoreFactory
			this.persistenceStore = null
		}

		_store() {
			if (!this.persistenceStore) this.persistenceStore = this.persistenceStoreFactory()
			return this.persistenceStore
		}

		async enqueue(handoff) {
			this._validateHandoff(handoff)
			const store = this._store()
			await store.finalizeCapture(handoff.captureId, {
				completionJob: {
					status: 'pending',
					enqueuedAt: new Date().toISOString(),
				},
			})
			return this.prepare(handoff.captureId)
		}

		async prepare(captureId) {
			const store = this._store()
			const stored = await store.getCapture(captureId)
			if (!stored) throw new Error(`PoRE completion job capture not found: ${captureId}`)
			if (stored.manifest.status !== 'finalized') throw new Error(`PoRE completion job requires finalized capture: ${captureId}`)
			const job = stored.manifest.completionJob
			if (job?.status === 'completed') return null

			try {
				const pcm = new Blob(stored.chunks, { type: 'application/octet-stream' })
				const sampleRate = stored.manifest.sampleRate
				const channels = stored.manifest.channels
				if (!Number.isFinite(sampleRate) || !Number.isFinite(channels)) throw new Error('PoRE completion job requires sample rate and channel count')
				const blob = new Blob([createWavHeader(pcm.size, sampleRate, channels), pcm], { type: 'audio/wav' })
				const payloadSha256 = await sha256(blob)
				const descriptor = {
					captureId: stored.manifest.captureId,
					recordingSessionId: stored.manifest.recordingSessionId,
					productionId: stored.manifest.productionId,
					recordingId: stored.manifest.recordingId,
					format: 'audio/wav',
					encoding: 'pcm_s24le',
					sampleRate,
					channels,
					size: blob.size,
					payloadSha256,
					chunkCount: stored.chunks.length,
					manifest: stored.manifest,
					blob,
				}
				await store.finalizeCapture(captureId, {
					completionJob: {
						status: 'prepared',
						preparedAt: new Date().toISOString(),
						payloadSha256,
						size: blob.size,
					},
				})
				window.dispatchEvent(new CustomEvent('pore:recording-transport-ready', { detail: descriptor }))
				return descriptor
			} catch (error) {
				await store.finalizeCapture(captureId, {
					completionJob: {
						status: 'failed',
						failedAt: new Date().toISOString(),
						error: String(error?.message || error),
					},
				})
				throw error
			}
		}

		async markCompleted(captureId, details = {}) {
			const store = this._store()
			return store.finalizeCapture(captureId, {
				completionJob: {
					status: 'completed',
					...details,
				},
			})
		}

		async recover() {
			const store = this._store()
			const captures = await store.listRecoverableCaptures()
			for (const manifest of captures) {
				if (manifest.status !== 'finalized') continue
				if (manifest.completionJob?.status === 'completed') continue
				try {
					await this.prepare(manifest.captureId)
				} catch (error) {
					window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } }))
				}
			}
		}

		_validateHandoff(handoff) {
			const required = ['productionId', 'recordingId', 'captureId', 'recordingSessionId']
			if (!handoff || required.some(key => !handoff[key])) throw new Error('PoRE completion job requires authoritative and technical identities')
			if (handoff.captureId === handoff.productionId || handoff.captureId === handoff.recordingId || handoff.recordingSessionId === handoff.productionId || handoff.recordingSessionId === handoff.recordingId) {
				throw new Error('PoRE completion job requires distinct technical identities')
			}
		}
	}

	async function sha256(blob) {
		if (!window.crypto?.subtle) throw new Error('PoRE completion job requires Web Crypto')
		const digest = await window.crypto.subtle.digest('SHA-256', await blob.arrayBuffer())
		return [...new Uint8Array(digest)].map(byte => byte.toString(16).padStart(2, '0')).join('')
	}

	function createWavHeader(dataLength, sampleRate, channels) {
		const header = new ArrayBuffer(44)
		const view = new DataView(header)
		const write = (offset, text) => [...text].forEach((char, index) => view.setUint8(offset + index, char.charCodeAt(0)))
		write(0, 'RIFF'); view.setUint32(4, 36 + dataLength, true); write(8, 'WAVE'); write(12, 'fmt ')
		view.setUint32(16, 16, true); view.setUint16(20, 1, true); view.setUint16(22, channels, true)
		view.setUint32(24, sampleRate, true); view.setUint32(28, sampleRate * channels * 3, true)
		view.setUint16(32, channels * 3, true); view.setUint16(34, 24, true); write(36, 'data'); view.setUint32(40, dataLength, true)
		return header
	}

	window.PoREBrowserCompletionJob = PoREBrowserCompletionJob
})()
