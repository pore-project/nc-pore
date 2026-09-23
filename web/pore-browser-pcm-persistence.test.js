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
		await expect(promise).resolves.toBe('capture-1')
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
