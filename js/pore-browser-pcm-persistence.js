/* NC-PoRE — durable browser PCM capture persistence. */
(() => {
	'use strict'

	const DB_NAME = 'nc-pore-recordings'
	const DB_VERSION = 3
	const MANIFEST_STORE = 'manifests'
	const CHUNK_STORE = 'chunks'
	const FINALIZED_CHUNK_STORE = 'finalizedChunks'

	class PoREBrowserPcmPersistenceStore {
		constructor({ indexedDBFactory = window.indexedDB, dbName = DB_NAME, keyRangeFactory = window.IDBKeyRange } = {}) {
			this.indexedDBFactory = indexedDBFactory
			this.dbName = dbName
			this.keyRangeFactory = keyRangeFactory
			this.db = null
		}

		async _database() {
			if (this.db) return this.db
			if (!this.indexedDBFactory) throw new Error('PoRE durable browser preservation requires IndexedDB')
			this.db = await new Promise((resolve, reject) => {
				const request = this.indexedDBFactory.open(this.dbName, DB_VERSION)
				request.onupgradeneeded = () => {
					const db = request.result
					if (!db.objectStoreNames.contains(MANIFEST_STORE)) db.createObjectStore(MANIFEST_STORE, { keyPath: 'captureId' })
					let chunks
					if (db.objectStoreNames.contains(CHUNK_STORE)) chunks = request.transaction.objectStore(CHUNK_STORE)
					else chunks = db.createObjectStore(CHUNK_STORE, { keyPath: ['captureId', 'index'] })
					if (!chunks.indexNames.contains('captureId')) chunks.createIndex('captureId', 'captureId', { unique: false })
					if (!db.objectStoreNames.contains(FINALIZED_CHUNK_STORE)) {
						const finalized = db.createObjectStore(FINALIZED_CHUNK_STORE, { keyPath: ['captureId', 'index'] })
						finalized.createIndex('captureId', 'captureId', { unique: false })
					}
				}
				request.onsuccess = () => {
					const db = request.result
					if (![MANIFEST_STORE, CHUNK_STORE, FINALIZED_CHUNK_STORE].every(name => db.objectStoreNames.contains(name))) {
						db.close()
						reject(new Error('PoRE IndexedDB schema is incomplete'))
						return
					}
					resolve(db)
				}
				request.onerror = () => reject(request.error || new Error('Unable to open PoRE IndexedDB database'))
			})
			this.db.onversionchange = () => { this.db.close(); this.db = null }
			return this.db
		}

		async beginCapture(manifest) {
			if (!manifest?.captureId) throw new Error('PoRE capture manifest requires captureId')
			const record = {
				...manifest,
				status: 'capturing',
				storageFormat: 'pcm_s24le',
				chunkCount: 0,
				lastChunkIndex: -1,
				chunks: [],
				updatedAt: new Date().toISOString(),
			}
			await this._put(await this._database(), MANIFEST_STORE, record)
			return record
		}

		appendChunk(captureId, index, payload) {
			return this._appendChunkToStore(captureId, index, payload, CHUNK_STORE, 'PoRE capture')
		}

		appendFinalizedChunk(captureId, index, payload) {
			return this._appendChunkToStore(captureId, index, payload, FINALIZED_CHUNK_STORE, 'PoRE FLAC finalization')
		}

		async _appendChunkToStore(captureId, index, payload, storeName, label) {
			if (!captureId) throw new Error('PoRE chunk requires captureId')
			if (!Number.isInteger(index) || index < 0) throw new Error('PoRE chunk requires a non-negative index')
			const blob = payload instanceof Blob ? payload : new Blob([payload], { type: 'application/octet-stream' })
			const sha256 = await this._sha256(blob)
			const db = await this._database()
			await this._transaction(db, [MANIFEST_STORE, storeName], 'readwrite', (transaction, abort) => {
				const manifestStore = transaction.objectStore(MANIFEST_STORE)
				const chunkStore = transaction.objectStore(storeName)
				const manifestRequest = manifestStore.get(captureId)
				manifestRequest.onsuccess = () => {
					const manifest = manifestRequest.result
					if (!manifest) { abort(new Error(label + ' manifest not found: ' + captureId)); return }
					if (storeName === FINALIZED_CHUNK_STORE && manifest.storageFormat === 'flac') {
						abort(new Error('PoRE FLAC finalization is already committed: ' + captureId))
						return
					}
					if (storeName === CHUNK_STORE && !['capturing', 'finalized'].includes(manifest.status)) {
						abort(new Error('PoRE capture is not writable: ' + captureId))
						return
					}
					const lastChunkIndex = storeName === FINALIZED_CHUNK_STORE
						? (Number.isInteger(manifest.flacStagedLastChunkIndex) ? manifest.flacStagedLastChunkIndex : -1)
						: (Number.isInteger(manifest.lastChunkIndex) ? manifest.lastChunkIndex : -1)
					if (index > lastChunkIndex + 1) {
						abort(new Error(label + ' chunk gap detected: ' + captureId + '/' + index))
						return
					}
					const existingRequest = chunkStore.get([captureId, index])
					existingRequest.onsuccess = () => {
						const existing = existingRequest.result
						if (existing) {
							if (existing.size !== blob.size || String(existing.sha256).toLowerCase() !== sha256) {
								abort(new Error('PoRE chunk retry differs from persisted payload: ' + captureId + '/' + index))
							}
							return
						}
						chunkStore.put({ captureId, index, payload: blob, size: blob.size, sha256 })
						if (storeName === CHUNK_STORE) {
							manifest.chunkCount = Math.max(Number.isInteger(manifest.chunkCount) ? manifest.chunkCount : 0, index + 1)
							manifest.lastChunkIndex = Math.max(lastChunkIndex, index)
						} else {
							manifest.flacStagedChunkCount = Math.max(Number.isInteger(manifest.flacStagedChunkCount) ? manifest.flacStagedChunkCount : 0, index + 1)
							manifest.flacStagedLastChunkIndex = Math.max(lastChunkIndex, index)
						}
						manifest.updatedAt = new Date().toISOString()
						manifestStore.put(manifest)
					}
				}
			})
		}

		async finalizeCapture(captureId, patch = {}) {
			const db = await this._database()
			const manifest = await this._get(db, MANIFEST_STORE, captureId)
			if (!manifest) throw new Error('PoRE capture manifest not found: ' + captureId)
			const finalized = { ...manifest, ...patch, status: 'finalized', finalizedAt: manifest.finalizedAt || new Date().toISOString(), updatedAt: new Date().toISOString() }
			await this._put(db, MANIFEST_STORE, finalized)
			return finalized
		}

		async getCapture(captureId) {
			const db = await this._database()
			const manifest = await this._get(db, MANIFEST_STORE, captureId)
			if (!manifest) return null
			if (manifest.storageFormat === 'flac') return this.getFinalizedPayload(captureId)
			return this._validatedPayload(db, manifest, CHUNK_STORE)
		}

		async getFinalizedPayload(captureId) {
			const db = await this._database()
			const manifest = await this._get(db, MANIFEST_STORE, captureId)
			if (!manifest) return null
			const validated = await this._validatedPayload(db, manifest, FINALIZED_CHUNK_STORE)
			return {
				...validated,
				chunkHashes: validated.records.map(chunk => chunk.sha256),
				chunkSizes: validated.records.map(chunk => chunk.size),
			}
		}

		async clearFinalizedPayload(captureId) {
			const db = await this._database()
			const chunks = await this._getChunks(db, captureId, FINALIZED_CHUNK_STORE)
			await this._transaction(db, [MANIFEST_STORE, FINALIZED_CHUNK_STORE], 'readwrite', (transaction, abort) => {
				const manifestStore = transaction.objectStore(MANIFEST_STORE)
				const finalizedStore = transaction.objectStore(FINALIZED_CHUNK_STORE)
				manifestStore.get(captureId).onsuccess = event => {
					const manifest = event.target.result
					if (!manifest || manifest.storageFormat === 'flac') return
					delete manifest.flacStagedChunkCount
					delete manifest.flacStagedLastChunkIndex
					manifest.updatedAt = new Date().toISOString()
					manifestStore.put(manifest)
					for (const chunk of chunks) finalizedStore.delete([captureId, chunk.index])
				}
			})
		}

		async commitFinalizedPayload(captureId, { size, payloadSha256, chunkCount, lastChunkIndex, chunkHashes = [], chunkSizes = [] } = {}) {
			if (!Number.isInteger(size) || size <= 0 || !Number.isInteger(chunkCount) || chunkCount <= 0 || !Number.isInteger(lastChunkIndex) || lastChunkIndex !== chunkCount - 1) throw new Error('PoRE FLAC commit metadata is invalid')
			if (!/^[a-f0-9]{64}$/i.test(String(payloadSha256 || ''))) throw new Error('PoRE FLAC commit requires a SHA-256 digest')
			if (!Array.isArray(chunkHashes) || chunkHashes.length !== chunkCount || chunkHashes.some(hash => !/^[a-f0-9]{64}$/i.test(String(hash)))) throw new Error('PoRE FLAC commit requires per-chunk SHA-256 digests')
			if (!Array.isArray(chunkSizes) || chunkSizes.length !== chunkCount || chunkSizes.some(value => !Number.isInteger(value) || value < 0)) throw new Error('PoRE FLAC commit requires per-chunk sizes')
			const db = await this._database()
			await this._transaction(db, [MANIFEST_STORE, CHUNK_STORE, FINALIZED_CHUNK_STORE], 'readwrite', (transaction, abort) => {
				const manifestStore = transaction.objectStore(MANIFEST_STORE)
				const rawStore = transaction.objectStore(CHUNK_STORE)
				const finalizedStore = transaction.objectStore(FINALIZED_CHUNK_STORE)
				manifestStore.get(captureId).onsuccess = event => {
					const manifest = event.target.result
					if (!manifest) { abort(new Error('PoRE capture manifest not found: ' + captureId)); return }
					if (manifest.storageFormat === 'flac') {
						if (manifest.size === size && String(manifest.payloadSha256).toLowerCase() === String(payloadSha256).toLowerCase()) return
						abort(new Error('PoRE FLAC finalization conflicts with committed payload: ' + captureId)); return
					}
					if (manifest.status !== 'finalized') { abort(new Error('PoRE FLAC finalization requires a finalized capture: ' + captureId)); return }
					finalizedStore.index('captureId').getAll(this.keyRangeFactory.only(captureId)).onsuccess = event2 => {
						const chunks = (event2.target.result || []).sort((a, b) => a.index - b.index)
						if (chunks.length !== chunkCount || chunks.some((chunk, index) => chunk.index !== index || chunk.size !== chunkSizes[index] || String(chunk.sha256).toLowerCase() !== String(chunkHashes[index]).toLowerCase())) {
							abort(new Error('PoRE FLAC staged payload changed before commit: ' + captureId))
							return
						}
						if (chunks.reduce((total, chunk) => total + chunk.size, 0) !== size) { abort(new Error('PoRE FLAC staged payload size changed: ' + captureId)); return }
						rawStore.index('captureId').getAll(this.keyRangeFactory.only(captureId)).onsuccess = event3 => {
							manifest.storageFormat = 'flac'
							manifest.format = 'audio/flac'
							manifest.encoding = 'flac'
							manifest.size = size
							manifest.payloadSha256 = String(payloadSha256).toLowerCase()
							manifest.chunkCount = chunkCount
							manifest.lastChunkIndex = lastChunkIndex
							delete manifest.flacStagedChunkCount
							delete manifest.flacStagedLastChunkIndex
							manifest.flacCommittedAt = new Date().toISOString()
							manifest.updatedAt = new Date().toISOString()
							manifestStore.put(manifest)
							for (const chunk of event3.target.result || []) rawStore.delete([captureId, chunk.index])
						}
					}
				}
			})
		}

		async listRecoverableCaptures() {
			const db = await this._database()
			const manifests = await this._getAll(db, MANIFEST_STORE)
			return manifests.filter(manifest => manifest.status !== 'finalized' || manifest.completionJob?.status !== 'completed' || manifest.completionJob?.coreCompletionStatus === 'pending')
		}

		async removeCapture(captureId) {
			const db = await this._database()
			const raw = await this._getChunks(db, captureId, CHUNK_STORE)
			const finalized = await this._getChunks(db, captureId, FINALIZED_CHUNK_STORE)
			await this._transaction(db, [MANIFEST_STORE, CHUNK_STORE, FINALIZED_CHUNK_STORE], 'readwrite', transaction => {
				transaction.objectStore(MANIFEST_STORE).delete(captureId)
				const rawStore = transaction.objectStore(CHUNK_STORE)
				const finalizedStore = transaction.objectStore(FINALIZED_CHUNK_STORE)
				for (const chunk of raw) rawStore.delete([captureId, chunk.index])
				for (const chunk of finalized) finalizedStore.delete([captureId, chunk.index])
			})
		}

		async _validatedPayload(db, manifest, storeName) {
			const records = await this._getChunks(db, manifest.captureId, storeName)
			const expectedIndexes = Array.from({ length: Math.max(0, Number.isInteger(manifest.lastChunkIndex) ? manifest.lastChunkIndex + 1 : 0) }, (_, index) => index)
			const actualIndexes = records.map(chunk => chunk.index)
			if (actualIndexes.length !== expectedIndexes.length || actualIndexes.some((index, position) => index !== expectedIndexes[position])) throw new Error('PoRE capture chunk continuity check failed: ' + manifest.captureId)
			if (manifest.chunkCount !== records.length) throw new Error('PoRE capture chunk count check failed: ' + manifest.captureId)
			for (const chunk of records) {
				const actual = await this._sha256(chunk.payload)
				if (actual !== String(chunk.sha256).toLowerCase()) throw new Error('PoRE capture chunk payload integrity check failed: ' + manifest.captureId + '/' + chunk.index)
			}
			return { manifest, records, chunks: records.map(chunk => chunk.payload) }
		}

		async _sha256(blob) {
			if (!window.crypto?.subtle) throw new Error('PoRE durable browser preservation requires Web Crypto')
			const digest = await window.crypto.subtle.digest('SHA-256', await blob.arrayBuffer())
			return [...new Uint8Array(digest)].map(byte => byte.toString(16).padStart(2, '0')).join('')
		}

		_put(db, storeName, value) { return this._request(db, storeName, 'readwrite', store => store.put(value)) }
		_get(db, storeName, key) { return this._request(db, storeName, 'readonly', store => store.get(key)) }
		_getAll(db, storeName) { return this._request(db, storeName, 'readonly', store => store.getAll()) }

		_getChunks(db, captureId, storeName = CHUNK_STORE) {
			return this._request(db, storeName, 'readonly', store => store.index('captureId').getAll(this.keyRangeFactory.only(captureId))).then(chunks => chunks.sort((a, b) => a.index - b.index))
		}

		_request(db, storeName, mode, operation) {
			return new Promise((resolve, reject) => {
				const transaction = db.transaction(storeName, mode)
				let requestResult
				let settled = false
				const rejectOnce = error => { if (settled) return; settled = true; reject(error) }
				transaction.oncomplete = () => { if (settled) return; settled = true; resolve(requestResult) }
				transaction.onerror = () => rejectOnce(transaction.error || new Error('PoRE IndexedDB transaction failed: ' + storeName))
				transaction.onabort = () => rejectOnce(transaction.error || new Error('PoRE IndexedDB transaction aborted: ' + storeName))
				let request
				try { request = operation(transaction.objectStore(storeName)) } catch (error) { rejectOnce(error); try { transaction.abort() } catch (_) {}; return }
				request.onsuccess = () => { requestResult = request.result }
				request.onerror = () => rejectOnce(request.error || new Error('PoRE IndexedDB request failed: ' + storeName))
			})
		}

		_transaction(db, stores, mode, configure) {
			return new Promise((resolve, reject) => {
				const transaction = db.transaction(stores, mode)
				let abortError = null
				const abort = error => { abortError = error; transaction.abort() }
				transaction.oncomplete = () => resolve()
				transaction.onerror = () => reject(abortError || transaction.error || new Error('PoRE IndexedDB transaction failed'))
				transaction.onabort = () => reject(abortError || transaction.error || new Error('PoRE IndexedDB transaction aborted'))
				try { configure(transaction, abort) } catch (error) { abort(error) }
			})
		}
	}

	window.PoREBrowserPcmPersistenceStore = PoREBrowserPcmPersistenceStore
})()
