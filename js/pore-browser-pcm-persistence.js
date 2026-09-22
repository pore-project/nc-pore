/* NC-PoRe — durable browser PCM capture persistence. */
(() => {
	'use strict'

	const DB_NAME = 'nc-pore-recordings'
	const DB_VERSION = 2
	const MANIFEST_STORE = 'manifests'
	const CHUNK_STORE = 'chunks'

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
					if (db.objectStoreNames.contains(CHUNK_STORE)) {
						chunks = request.transaction.objectStore(CHUNK_STORE)
					} else {
						chunks = db.createObjectStore(CHUNK_STORE, { keyPath: ['captureId', 'index'] })
					}
					if (!chunks.indexNames.contains('captureId')) chunks.createIndex('captureId', 'captureId', { unique: false })
				}
				request.onsuccess = () => {
					const db = request.result
					if (!db.objectStoreNames.contains(MANIFEST_STORE) || !db.objectStoreNames.contains(CHUNK_STORE)) {
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
				chunkCount: 0,
				lastChunkIndex: -1,
				chunks: [],
				updatedAt: new Date().toISOString(),
			}
			const db = await this._database()
			await this._put(db, MANIFEST_STORE, record)
			return record
		}

		async appendChunk(captureId, index, payload) {
			if (!captureId) throw new Error('PoRE chunk requires captureId')
			if (!Number.isInteger(index) || index < 0) throw new Error('PoRE chunk requires a non-negative index')
			const blob = payload instanceof Blob ? payload : new Blob([payload], { type: 'application/octet-stream' })
			const sha256 = await this._sha256(blob)
			const db = await this._database()
			await this._transaction(db, [MANIFEST_STORE, CHUNK_STORE], 'readwrite', (transaction, abort) => {
				const manifestStore = transaction.objectStore(MANIFEST_STORE)
				const chunkStore = transaction.objectStore(CHUNK_STORE)
				const manifestRequest = manifestStore.get(captureId)

				manifestRequest.onsuccess = () => {
					const manifest = manifestRequest.result
					if (!manifest) {
						abort(new Error(`PoRE capture manifest not found: ${captureId}`))
						return
					}
					const lastChunkIndex = Number.isInteger(manifest.lastChunkIndex) ? manifest.lastChunkIndex : -1
					if (index > lastChunkIndex + 1) {
						abort(new Error(`PoRE capture chunk gap detected: ${captureId}/${index}`))
						return
					}

					// Chunk keys are unique and the recorder queues chunks serially. Using
					// put() makes retries idempotent without a second read transaction.
					chunkStore.put({ captureId, index, payload: blob, size: blob.size, sha256 })
					manifest.chunkCount = Math.max(Number.isInteger(manifest.chunkCount) ? manifest.chunkCount : 0, index + 1)
					manifest.lastChunkIndex = Math.max(lastChunkIndex, index)
					manifest.updatedAt = new Date().toISOString()
					manifestStore.put(manifest)
				}
			})
		}

		async finalizeCapture(captureId, patch = {}) {
			const db = await this._database()
			const manifest = await this._get(db, MANIFEST_STORE, captureId)
			if (!manifest) throw new Error(`PoRE capture manifest not found: ${captureId}`)
			const finalized = { ...manifest, ...patch, status: 'finalized', finalizedAt: manifest.finalizedAt || new Date().toISOString(), updatedAt: new Date().toISOString() }
			await this._put(db, MANIFEST_STORE, finalized)
			return finalized
		}

		async getCapture(captureId) {
			const db = await this._database()
			const manifest = await this._get(db, MANIFEST_STORE, captureId)
			if (!manifest) return null
			const chunks = await this._getChunks(db, captureId)
			const expectedIndexes = Array.from({ length: Math.max(0, manifest.lastChunkIndex + 1) }, (_, index) => index)
			const actualIndexes = chunks.map(chunk => chunk.index)
			if (actualIndexes.length !== expectedIndexes.length || actualIndexes.some((index, position) => index !== expectedIndexes[position])) {
				throw new Error(`PoRE capture chunk continuity check failed: ${captureId}`)
			}
			if (manifest.chunkCount !== chunks.length) throw new Error(`PoRE capture chunk count check failed: ${captureId}`)
			for (const chunk of chunks) {
				const actual = await this._sha256(chunk.payload)
				if (actual !== chunk.sha256) throw new Error(`PoRE capture chunk payload integrity check failed: ${captureId}/${chunk.index}`)
			}
			return { manifest, chunks: chunks.map(chunk => chunk.payload) }
		}

		async listRecoverableCaptures() {
			const db = await this._database()
			const manifests = await this._getAll(db, MANIFEST_STORE)
			return manifests.filter(manifest => manifest.status !== 'finalized' || manifest.completionJob?.status !== 'completed' || manifest.completionJob?.coreCompletionStatus === 'pending')
		}

		async removeCapture(captureId) {
			const db = await this._database()
			const chunks = await this._getChunks(db, captureId)
			await this._transaction(db, [MANIFEST_STORE, CHUNK_STORE], 'readwrite', transaction => {
				transaction.objectStore(MANIFEST_STORE).delete(captureId)
				for (const chunk of chunks) transaction.objectStore(CHUNK_STORE).delete([captureId, chunk.index])
			})
		}

		async _sha256(blob) {
			if (!window.crypto?.subtle) throw new Error('PoRE durable browser preservation requires Web Crypto')
			const digest = await window.crypto.subtle.digest('SHA-256', await blob.arrayBuffer())
			return [...new Uint8Array(digest)].map(byte => byte.toString(16).padStart(2, '0')).join('')
		}

		_put(db, storeName, value) { return this._request(db, storeName, 'readwrite', store => store.put(value)) }
		_get(db, storeName, key) { return this._request(db, storeName, 'readonly', store => store.get(key)) }
		_getAll(db, storeName) { return this._request(db, storeName, 'readonly', store => store.getAll()) }

		_getChunks(db, captureId) {
			return this._request(db, CHUNK_STORE, 'readonly', store => {
				const index = store.index('captureId')
				return index.getAll(this.keyRangeFactory.only(captureId))
			}).then(chunks => chunks.sort((a, b) => a.index - b.index))
		}

		_request(db, storeName, mode, operation) {
			return new Promise((resolve, reject) => {
				const transaction = db.transaction(storeName, mode)
				const request = operation(transaction.objectStore(storeName))
				request.onsuccess = () => resolve(request.result)
				request.onerror = () => reject(request.error || new Error(`PoRE IndexedDB request failed: ${storeName}`))
				transaction.onerror = () => reject(transaction.error || new Error(`PoRE IndexedDB transaction failed: ${storeName}`))
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
				configure(transaction, abort)
			})
		}
	}

	window.PoREBrowserPcmPersistenceStore = PoREBrowserPcmPersistenceStore
})()
