/* NC-PoRE — durable browser completion job. */
(() => {
	'use strict'
	const V1_SAMPLE_RATES = [48000, 44100]

	class PoREBrowserCompletionJob {
		constructor({ persistenceStoreFactory = () => new window.PoREBrowserPcmPersistenceStore() } = {}) { this.persistenceStoreFactory = persistenceStoreFactory; this.persistenceStore = null }
		_store() { if (!this.persistenceStore) this.persistenceStore = this.persistenceStoreFactory(); return this.persistenceStore }
		async enqueue(handoff) {
			this._validateHandoff(handoff)
			const store = this._store()
			await store.finalizeCapture(handoff.captureId, { provenance: handoff.provenance || null, completionJob: { status: 'pending', enqueuedAt: new Date().toISOString() } })
			return this.prepare(handoff.captureId)
		}
		async prepare(captureId) {
			const store = this._store()
			let stored = await store.getCapture(captureId)
			if (!stored) throw new Error(`PoRE completion job capture not found: ${captureId}`)
			if (stored.manifest.status !== 'finalized') throw new Error(`PoRE completion job requires finalized capture: ${captureId}`)
			let job = stored.manifest.completionJob || {}
			if (job.status === 'completed') return null
			try {
				if (stored.manifest.storageFormat !== 'flac') {
					await this._ensureFinalizedFlac(stored)
					stored = await store.getCapture(captureId)
					if (!stored || stored.manifest.storageFormat !== 'flac') throw new Error(`PoRE FLAC finalization did not commit: ${captureId}`)
					job = stored.manifest.completionJob || job
				}
				const blob = new Blob(stored.chunks, { type: 'audio/flac' })
				if (blob.size <= 0 || blob.size !== stored.manifest.size) throw new Error('PoRE FLAC payload size does not match the durable manifest')
				const payloadSha256 = await sha256(blob)
				if (payloadSha256 !== stored.manifest.payloadSha256) throw new Error('PoRE FLAC payload integrity does not match the durable manifest')
				const sampleRate = stored.manifest.sampleRate
				const channels = stored.manifest.channels
				if (!V1_SAMPLE_RATES.includes(sampleRate) || channels !== 1) throw new Error('PoRE completion job has unsupported V1 audio properties')
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
					sampleRate, channels,
					size: blob.size,
					payloadSha256,
					chunkCount: stored.chunks.length,
					provenance: stored.manifest.provenance || null,
					manifest: stored.manifest,
					blob,
					completionJob: job,
				}
				await store.finalizeCapture(captureId, { completionJob: { ...job, status: ['authorized', 'remote_present', 'verified', 'transport_closed'].includes(job.status) ? job.status : 'prepared', preparedAt: job.preparedAt || new Date().toISOString(), payloadSha256, size: blob.size, payloadFormat: 'audio/flac' } })
				window.dispatchEvent(new CustomEvent('pore:recording-transport-ready', { detail: descriptor }))
				return descriptor
			} catch (error) {
				await store.finalizeCapture(captureId, { completionJob: { ...job, status: 'failed', failedAt: new Date().toISOString(), error: String(error?.message || error) } })
				throw error
			}
		}
		async _ensureFinalizedFlac(stored) {
			const store = this._store()
			const captureId = stored.manifest.captureId
			const sampleRate = stored.manifest.sampleRate
			const channels = stored.manifest.channels
			if (!V1_SAMPLE_RATES.includes(sampleRate)) throw new Error(`PoRE V1 FLAC finalization does not support ${sampleRate} Hz`)
			if (channels !== 1 || stored.manifest.encoding !== 'pcm_s24le') throw new Error('PoRE V1 FLAC finalization requires mono PCM24 preservation')
			const pcmSize = stored.chunks.reduce((total, chunk) => total + chunk.size, 0)
			if (pcmSize <= 0 || pcmSize % 3 !== 0) throw new Error('PoRE PCM preservation payload is not a complete PCM24 sample stream')
			const totalSamples = pcmSize / 3
			await store.clearFinalizedPayload(captureId)
			let encoder = null
			let nextIndex = 0
			try {
				encoder = await window.PoREBrowserFlacEncoder.create({ sampleRate, channels, totalSamples })
				for (const chunk of stored.chunks) {
					const bytes = new Uint8Array(await chunk.arrayBuffer())
					for (const payload of encoder.encodePcm24Bytes(bytes)) await store.appendFinalizedChunk(captureId, nextIndex++, new Blob([payload], { type: 'audio/flac' }))
				}
				for (const payload of encoder.finish()) await store.appendFinalizedChunk(captureId, nextIndex++, new Blob([payload], { type: 'audio/flac' }))
				encoder = null
				const finalized = await store.getFinalizedPayload(captureId)
				if (!finalized || finalized.chunks.length !== nextIndex) throw new Error(`PoRE FLAC staging is incomplete: ${captureId}`)
				const blob = new Blob(finalized.chunks, { type: 'audio/flac' })
				const header = new Uint8Array(await blob.slice(0, 4).arrayBuffer())
				if (String.fromCharCode(...header) !== 'fLaC') throw new Error('PoRE FLAC encoder did not produce a FLAC stream')
				const payloadSha256 = await sha256(blob)
				await store.commitFinalizedPayload(captureId, { size: blob.size, payloadSha256, chunkCount: finalized.chunks.length, lastChunkIndex: finalized.chunks.length - 1, chunkHashes: finalized.chunkHashes, chunkSizes: finalized.chunkSizes })
			} catch (error) {
				try { encoder?.free?.() } catch (_) {}
				await store.clearFinalizedPayload(captureId).catch(() => {})
				throw error
			}
		}
		async updateTransportState(captureId, patch) {
			const store = this._store()
			const stored = await store.getCapture(captureId)
			if (!stored) throw new Error(`PoRE transport capture not found: ${captureId}`)
			return store.finalizeCapture(captureId, { completionJob: { ...(stored.manifest.completionJob || {}), ...patch } })
		}
		async getTransportState(captureId) { const stored = await this._store().getCapture(captureId); return stored?.manifest?.completionJob || null }
		async markCompleted(captureId, details = {}) { return this.updateTransportState(captureId, { status: 'completed', coreCompletionStatus: 'pending', ...details }) }
		async removeCapture(captureId) { if (!captureId) throw new Error('PoRE completion job requires a capture id for cleanup'); return this._store().removeCapture(captureId) }
		async recover() {
			const store = this._store()
			const captures = await store.listRecoverableCaptures()
			for (const manifest of captures) {
				if (manifest.status !== 'finalized') continue
				const job = manifest.completionJob || {}
				if (job.status === 'completed') {
					if (job.coreCompletionStatus === 'completed') {
						try { await this.removeCapture(manifest.captureId) } catch (error) { window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } })) }
					} else if (job.coreCompletionStatus === 'pending' && job.artifactId) {
						window.dispatchEvent(new CustomEvent('pore:recording-transport-completed', { detail: { captureId: manifest.captureId, artifact_id: job.artifactId, file_id: job.fileId, path: job.path, size: job.size, sha256: job.sha256 } }))
					}
					continue
				}
				try { await this.prepare(manifest.captureId) } catch (error) { window.dispatchEvent(new CustomEvent('pore:recording-local-error', { detail: { error } })) }
			}
		}
		_validateHandoff(handoff) {
			const required = ['productionId', 'recordingId', 'captureId', 'recordingSessionId']
			if (!handoff || required.some(key => !handoff[key])) throw new Error('PoRE completion job requires authoritative and technical identities')
			if (handoff.captureId === handoff.productionId || handoff.captureId === handoff.recordingId || handoff.recordingSessionId === handoff.productionId || handoff.recordingSessionId === handoff.recordingId) throw new Error('PoRE completion job requires distinct technical identities')
		}
	}
	async function sha256(blob) {
		if (!window.crypto?.subtle) throw new Error('PoRE completion job requires Web Crypto')
		const digest = await window.crypto.subtle.digest('SHA-256', await blob.arrayBuffer())
		return [...new Uint8Array(digest)].map(byte => byte.toString(16).padStart(2, '0')).join('')
	}
	window.PoREBrowserCompletionJob = PoREBrowserCompletionJob
})()
