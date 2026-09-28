import '../js/pore-browser-pcm-persistence.js'

describe('Browser PCM persistence store transactions', () => {
	const Store = window.PoREBrowserPcmPersistenceStore

	function createFakeDb() {
		const request = {}
		const store = {
			put: jest.fn(() => request),
			get: jest.fn(() => request),
		}
		const transaction = {
			objectStore: jest.fn(() => store),
			oncomplete: null,
			onerror: null,
			onabort: null,
			error: null,
			abort: jest.fn(() => { transaction.onabort?.() }),
		}
		const db = {
			transaction: jest.fn(() => transaction),
		}
		return { db, store, request, transaction }
	}

	it('does not resolve a write until the IndexedDB transaction commits', async () => {
		const { db, store, request, transaction } = createFakeDb()
		const persistence = new Store()
		const promise = persistence._request(db, 'manifests', 'readwrite', target => target.put({ captureId: 'capture-1' }))

		expect(store.put).toHaveBeenCalledTimes(1)
		request.result = 'capture-1'
		request.onsuccess()

		let settled = false
		void promise.then(() => { settled = true })
		await Promise.resolve()
		expect(settled).toBe(false)

		transaction.oncomplete()
		const result = await promise
		expect(result).toBe('capture-1')
	})

	it('rejects when the transaction is aborted after the request succeeds', async () => {
		const { db, request, transaction } = createFakeDb()
		const persistence = new Store()
		const promise = persistence._request(db, 'manifests', 'readwrite', target => target.put({ captureId: 'capture-1' }))

		request.result = 'capture-1'
		request.onsuccess()
		transaction.error = new Error('QuotaExceededError')
		transaction.onabort()

		await expect(promise).rejects.toThrow('QuotaExceededError')
	})
})


describe('Browser FLAC finalization persistence', () => {
	const Store = window.PoREBrowserPcmPersistenceStore

	it('can read staged FLAC chunks before the atomic storage-format commit', async () => {
		const persistence = new Store()
		const payload = new Blob([new Uint8Array([1, 2, 3])])
		persistence._database = jest.fn(async () => ({}))
		persistence._get = jest.fn(async () => ({ captureId: 'capture-1', status: 'finalized', storageFormat: 'pcm', finalizedChunkCount: 1 }))
		persistence._getChunks = jest.fn(async () => [{ captureId: 'capture-1', index: 0, payload, size: payload.size, sha256: 'a'.repeat(64) }])
		persistence._sha256 = jest.fn(async () => 'a'.repeat(64))

		const finalized = await persistence.getFinalizedPayload('capture-1')

		expect(finalized.manifest.storageFormat).toBe('pcm')
		expect(finalized.chunks).toHaveLength(1)
	})

	it('atomically switches to FLAC and removes the old PCM chunks only after exact verification', async () => {
		const persistence = new Store()
		const first = new Blob([new Uint8Array([1, 2])])
		const second = new Blob([new Uint8Array([3, 4, 5])])
		const chunks = [
			{ captureId: 'capture-1', index: 0, payload: first, size: first.size, sha256: 'c'.repeat(64) },
			{ captureId: 'capture-1', index: 1, payload: second, size: second.size, sha256: 'c'.repeat(64) },
		]
		const manifest = { captureId: 'capture-1', status: 'finalized', storageFormat: 'pcm', finalizedChunkCount: 2 }
		const putCalls = []
		const deleteCalls = []
		persistence._database = jest.fn(async () => ({}))
		persistence._get = jest.fn(async () => manifest)
		persistence._getChunks = jest.fn(async () => chunks)
		persistence._sha256 = jest.fn(async blob => blob.size === 5 ? 'a'.repeat(64) : 'c'.repeat(64))
		persistence._transaction = jest.fn(async (_db, _stores, _mode, configure) => {
			const stores = new Map([
				['manifests', { put: value => putCalls.push(value) }],
				['chunks', { delete: key => deleteCalls.push(key) }],
				['finalizedChunks', { delete: () => {} }],
			])
			configure({ objectStore: name => stores.get(name) })
		})

		const result = await persistence.commitFinalizedPayload('capture-1', {
			size: 5,
			payloadSha256: 'a'.repeat(64),
			sampleCount: 3,
		})

		expect(result.storageFormat).toBe('flac')
		expect(result.format).toBe('audio/flac')
		expect(result.encoding).toBe('flac')
		expect(result.chunkCount).toBe(2)
		expect(putCalls).toHaveLength(1)
		expect(putCalls[0].sampleCount).toBe(3)
		expect(deleteCalls).toEqual([['capture-1', 0], ['capture-1', 1]])
	})

	it('clears staged FLAC chunks and resets their continuity counter before retry', async () => {
		const persistence = new Store()
		const chunks = [
			{ captureId: 'capture-1', index: 0, payload: new Blob([new Uint8Array([1])]), size: 1, sha256: 'a'.repeat(64) },
			{ captureId: 'capture-1', index: 1, payload: new Blob([new Uint8Array([2])]), size: 1, sha256: 'a'.repeat(64) },
		]
		const manifest = { captureId: 'capture-1', status: 'finalized', storageFormat: 'pcm', finalizedChunkCount: 2 }
		const deletes = []
		let manifestPut = null
		persistence._database = jest.fn(async () => ({}))
		persistence._getChunks = jest.fn(async () => chunks)
		persistence._transaction = jest.fn(async (_db, _stores, _mode, configure) => {
			const request = { result: manifest, onsuccess: null }
			const manifestStore = {
				get: () => request,
				put: value => { manifestPut = value },
			}
			configure({
				objectStore: name => name === 'manifests'
					? manifestStore
					: { delete: key => deletes.push(key) },
			})
			request.onsuccess?.()
		})

		await persistence.clearFinalizedPayload('capture-1')

		expect(manifestPut.finalizedChunkCount).toBe(0)
		expect(deletes).toEqual([['capture-1', 0], ['capture-1', 1]])
	})
})
