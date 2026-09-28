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
				provenance: handoff.provenance || null,
				completionJob: {
					status: 'pending',
					enqueuedAt: new Date().toISOString(),
				},
			})
			return this.prepare(handoff.captureId)
		}

		async prepare(captureId) {
			const store = this._store()
			let stored = await store.getCapture(captureId)
			if (!stored) throw new Error(`PoRE completion job capture not found: ${captureId}`)
			if (stored.manifest.status !== 'finalized') throw new Error(`PoRE completion job requires finalized capture: ${captureId}`)
			const job = stored.manifest.completionJob || {}
			if (job.status === 'completed') return null

			try {
				if (stored.manifest.storageFormat !== 'flac') {
					await this._ensureFinalizedFlac(stored)
					stored = await store.getCapture(captureId)
				}
				if (!stored || stored.manifest.storageFormat !== 'flac') throw new Error('PoRE completion job did not produce finalized FLAC persistence')

				const sampleRate = stored.manifest.sampleRate
				const channels = stored.manifest.channels
				if (!Number.isFinite(sampleRate) || !Number.isFinite(channels)) throw new Error('PoRE completion job requires sample rate and channel count')
				if (channels !== 1) throw new Error('PoRE V1 FLAC transport requires mono audio')

				const blob = new Blob(stored.chunks, { type: 'audio/flac' })
				const payloadSha256 = await sha256(blob)
				const descriptor = {
					captureId: stored.manifest.captureId,
					recordingSessionId: stored.manifest.recordingSessionId,
					productionId: stored.manifest.productionId,
					productionLabel: stored.manifest.productionLabel || stored.manifest.productionTitle || stored.manifest.productionId,
					participantLabel: stored.manifest.participantLabel || null,
					recordingId: stored.manifest.recordingId,
					startedAt: stored.manifest.startedAt || stored.manifest.createdAt || new Date().toISOString(),
					format: 'audio/flac',
					encoding: 'flac',
					sampleRate,
					channels,
					size: blob.size,
					payloadSha256,
					chunkCount: stored.chunks.length,
					provenance: stored.manifest.provenance || null,
					manifest: stored.manifest,
					blob,
					completionJob: job,
				}
				await store.finalizeCapture(captureId, {
					completionJob: {
						...job,
						status: job.status === 'authorized' || job.status === 'remote_present' || job.status === 'verified' || job.status === 'transport_closed' ? job.status : 'prepared',
						preparedAt: job.preparedAt || new Date().toISOString(),
						payloadSha256,
						size: blob.size,
					},
					payloadSha256,
					size: blob.size,
					format: 'audio/flac',
					encoding: 'flac',
				})
				window.dispatchEvent(new CustomEvent('pore:recording-transport-ready', { detail: descriptor }))
				return descriptor
			} catch (error) {
				await store.finalizeCapture(captureId, {
					completionJob: {
						...job,
						status: 'failed',
						failedAt: new Date().toISOString(),
						error: String(error?.message || error),
					},
				})
				throw error
			}
		}

		async _ensureFinalizedFlac(stored) {
			const store = this._store()
			const captureId = stored.manifest.captureId
			const sampleRate = stored.manifest.sampleRate
			const channels = stored.manifest.channels
			if (!Number.isFinite(sampleRate) || !Number.isFinite(channels)) throw new Error('PoRE FLAC conversion requires sample rate and channel count')
			if (channels !== 1) throw new Error('PoRE V1 FLAC conversion requires mono audio')
			if (stored.manifest.encoding && stored.manifest.encoding !== 'pcm_s24le' && stored.manifest.storageFormat !== 'flac') throw new Error('PoRE FLAC conversion requires packed PCM24 input')
			const pcmSize = stored.chunks.reduce((total, chunk) => total + chunk.size, 0)
			if (pcmSize === 0 || pcmSize % 3 !== 0) throw new Error('PoRE FLAC conversion requires sample-aligned PCM24 data')
			await store.clearFinalizedPayload(captureId)
			const encoder = new window.PoREBrowserFlacEncoder({
				sampleRate,
				channels,
				totalSamples: pcmSize / 3,
				compression: 5,
			})
			let index = 0
			try {
				for (const chunk of stored.chunks) {
					const bytes = new Uint8Array(await chunk.arrayBuffer())
					for (const output of encoder.encodePcm24Bytes(bytes)) {
						if (output.length) await store.appendFinalizedChunk(captureId, index++, output)
					}
				}
				for (const output of encoder.finish()) {
					if (output.length) await store.appendFinalizedChunk(captureId, index++, output)
				}
			} finally {
				encoder.free?.()
			}

			const finalized = await store.getFinalizedPayload(captureId)
			if (!finalized || !Array.isArray(finalized.chunks) || finalized.chunks.length === 0) throw new Error('PoRE FLAC conversion produced no payload')
			const blob = new Blob(finalized.chunks, { type: 'audio/flac' })
			const payloadSha256 = await sha256(blob)
			await store.commitFinalizedPayload(captureId, {
				size: blob.size,
				payloadSha256,
				sampleCount: pcmSize / 3,
			})
		}

		async updateTransportState(captureId, patch) {
			const store = this._store()
			const stored = await store.getCapture(captureId)
			if (!stored) throw new Error(`PoRE transport capture not found: ${captureId}`)
			return store.finalizeCapture(captureId, {
				completionJob: {
					...(stored.manifest.completionJob || {}),
					...patch,
				},
			})
		}

		async getTransportState(captureId) {
			const stored = await this._store().getCapture(captureId)
			return stored?.manifest?.completionJob || null
		}

		async markCompleted(captureId, details = {}) {
			return this.updateTransportState(captureId, {
				status: 'completed',
				coreCompletionStatus: 'pending',
				...details,
			})
		}

		async removeCapture(captureId) {
			if (!captureId) throw new Error('PoRE completion job requires a capture id for cleanup')
			return this._store().removeCapture(captureId)
		}

		async recover() {
			const store = this._store()
			const captures = await store.listRecoverableCaptures()
			for (const manifest of captures) {
				if (manifest.status !== 'finalized') continue
				const job = manifest.completionJob || {}
				if (job.status === 'completed') {
					if (job.coreCompletionStatus === 'completed') {
						try { await this.removeCapture(manifest.captureId) } catch (error) {
							window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } }))
						}
					} else if (job.coreCompletionStatus === 'pending' && job.artifactId) {
						window.dispatchEvent(new CustomEvent('pore:recording-transport-completed', {
							detail: {
								captureId: manifest.captureId,
								artifact_id: job.artifactId,
								file_id: job.fileId,
								path: job.path,
								size: job.size,
								sha256: job.sha256,
							},
						}))
					}
					continue
				}
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
