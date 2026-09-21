import '../js/pore-browser-pcm-recorder.js'

describe('Browser PCM recorder persistence recovery', () => {
	const Recorder = window.PoREBrowserPcmRecorder

	beforeEach(() => {
		jest.restoreAllMocks()
	})

	it('keeps realtime capture active when a persistence chunk temporarily fails', async () => {
		let attempts = 0
		const store = {
			appendChunk: jest.fn(async () => {
				attempts += 1
				if (attempts === 1) throw new Error('temporary persistence failure')
			}),
		}
		const recorder = new Recorder({
			AudioContextClass: null,
			persistenceStoreFactory: () => store,
			persistenceChunkBytes: 3,
			maxPersistenceQueueBytes: 12,
		})
		recorder.state = 'recording'
		recorder.captureId = 'capture-1'
	recorder.recordingSessionId = 'session-1'
	recorder.sampleRate = 48000
	recorder.persistenceStore = store

	recorder._acceptSamples(new Float32Array([0]))
	await Promise.resolve()
	await Promise.resolve()
	recorder._acceptSamples(new Float32Array([0]))

	expect(recorder.getState()).toBe('recording')
	expect(recorder.persistenceQueue).toHaveLength(2)
	expect(recorder.persistenceState).toBe('recovering')

	if (recorder.persistenceDrainPromise) await recorder.persistenceDrainPromise
	recorder._clearPersistenceRetry()
	await recorder._drainPersistenceQueue()

	expect(recorder.getState()).toBe('recording')
	expect(recorder.persistenceQueue).toHaveLength(0)
	expect(store.appendChunk).toHaveBeenCalledTimes(3)
	})

	it('waits for sustained persistence failure before safety stop below the hard backlog bound', async () => {
		const originalDateNow = Date.now
		let now = 1000
		Date.now = () => now
		try {
			const safetyStop = jest.fn()
			const store = { appendChunk: jest.fn(async () => { throw new Error('persistence unavailable') }) }
			const recorder = new Recorder({
				AudioContextClass: null,
				persistenceStoreFactory: () => store,
				persistenceChunkBytes: 3,
				maxPersistenceQueueBytes: 12,
				onPersistenceSafetyStop: safetyStop,
			})
			recorder.state = 'recording'
			recorder.captureId = 'capture-1'
			recorder.recordingSessionId = 'session-1'
			recorder.sampleRate = 48000
			recorder.persistenceStore = store

			recorder._acceptSamples(new Float32Array([0]))
			await Promise.resolve()
			await Promise.resolve()
			expect(safetyStop).toHaveBeenCalledTimes(0)

			now += 2 * 60 * 1000
			recorder._clearPersistenceRetry()
			await recorder._drainPersistenceQueue()

			expect(recorder.getState()).toBe('recording')
			expect(safetyStop).toHaveBeenCalledTimes(1)
			expect(safetyStop.mock.calls[0][0].pendingBytes).toBe(3)
		} finally {
			Date.now = originalDateNow
		}
	})

	it('requests a controlled safety stop when persistence backlog reaches its bound', async () => {
		const safetyStop = jest.fn()
		const store = { appendChunk: jest.fn(async () => { throw new Error('persistence unavailable') }) }
		const recorder = new Recorder({
			AudioContextClass: null,
			persistenceStoreFactory: () => store,
			persistenceChunkBytes: 3,
			maxPersistenceQueueBytes: 6,
			onPersistenceSafetyStop: safetyStop,
		})
		recorder.state = 'recording'
		recorder.captureId = 'capture-1'
		recorder.recordingSessionId = 'session-1'
		recorder.sampleRate = 48000

		recorder._acceptSamples(new Float32Array([0]))
		await Promise.resolve()
		await Promise.resolve()
		recorder._acceptSamples(new Float32Array([0]))
		await Promise.resolve()
		await Promise.resolve()

		expect(recorder.getState()).toBe('recording')
		expect(safetyStop).toHaveBeenCalledTimes(1)
		expect(safetyStop.mock.calls[0][0].pendingBytes).toBe(6)

		recorder._acceptSamples(new Float32Array([0]))
		expect(recorder.capturedSamples).toBe(2)
		expect(recorder.pendingBytes).toBe(0)
		expect(recorder.persistenceQueue).toHaveLength(2)
	})
})
