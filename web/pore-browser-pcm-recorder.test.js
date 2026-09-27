import '../js/pore-browser-pcm-recorder.js'

describe('Browser PCM recorder persistence recovery', () => {
	const Recorder = window.PoREBrowserPcmRecorder

	beforeEach(() => {
		jest.restoreAllMocks()
	})

	// TEST-01: transient persistence failure must not stop active capture.
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

	// TEST-02: safety cutoff is time-based below the hard backlog bound.
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

	// TEST-03: a recovering queue must stop immediately at its hard backlog bound.
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

		expect(recorder.getState()).toBe('recording')
		expect(safetyStop).toHaveBeenCalledTimes(1)
		expect(safetyStop.mock.calls[0][0].pendingBytes).toBe(6)

		recorder._acceptSamples(new Float32Array([0]))
		expect(recorder.capturedSamples).toBe(2)
		expect(recorder.pendingBytes).toBe(0)
		expect(recorder.persistenceQueue).toHaveLength(2)
	})
	// TEST-04: preservation follows the actual reported capture rate and the worklet receives one mono channel.
	it('binds the preservation context to the actual track rate and configures explicit mono downmix', async () => {
		const previousMediaStream = globalThis.MediaStream
		const previousAudioWorkletNode = globalThis.AudioWorkletNode

		class FakeMediaStream {
			constructor(tracks) { this.tracks = tracks }
			getTracks() { return this.tracks.slice() }
		}

		class FakeAudioContext {
			static instances = []
			constructor(options = {}) {
				this.options = options
				this.sampleRate = options.sampleRate || 48000
				this.state = 'running'
				this.audioWorklet = { addModule: jest.fn().mockResolvedValue(undefined) }
				this.destination = {}
				FakeAudioContext.instances.push(this)
			}
			createMediaStreamTrackSource = track => ({ track, connect: jest.fn(), disconnect: jest.fn() })
			createMediaStreamSource = stream => ({ stream, connect: jest.fn(), disconnect: jest.fn() })
			createMediaStreamDestination = () => ({})
			createGain = () => ({ gain: { value: 0 }, connect: jest.fn(), disconnect: jest.fn() })
			resume = jest.fn(async () => { this.state = 'running' })
			close = jest.fn(async () => { this.state = 'closed' })
		}

		class FakeAudioWorkletNode {
			static instances = []
			constructor(context, name, options) {
				this.context = context
				this.name = name
				this.options = options
				this.port = { onmessage: null }
				this.connect = jest.fn()
				this.disconnect = jest.fn()
				FakeAudioWorkletNode.instances.push(this)
			}
		}

		globalThis.MediaStream = FakeMediaStream
		globalThis.AudioWorkletNode = FakeAudioWorkletNode

		try {
			const store = {
				beginCapture: jest.fn().mockResolvedValue(undefined),
			}
			const recorder = new Recorder({
				AudioContextClass: FakeAudioContext,
				persistenceStoreFactory: () => store,
			})
			const track = {
				id: 'pore-rate-44100',
				kind: 'audio',
				readyState: 'live',
				getSettings: () => ({ sampleRate: 44100, channelCount: 2, echoCancellation: false, noiseSuppression: false, autoGainControl: false }),
			}

			await recorder.primeAudioContext()
			expect(FakeAudioContext.instances).toHaveLength(1)
			expect(FakeAudioContext.instances[0].sampleRate).toBe(48000)

			await recorder.start(track, {
				captureId: 'capture-rate-1',
				recordingSessionId: 'session-rate-1',
			})

			expect(FakeAudioContext.instances).toHaveLength(2)
			expect(FakeAudioContext.instances[0].close).toHaveBeenCalledTimes(1)
			expect(FakeAudioContext.instances[1].options).toEqual({ sampleRate: 44100 })
			expect(recorder.sampleRate).toBe(44100)
			expect(FakeAudioWorkletNode.instances[0].options).toEqual({
				numberOfInputs: 1,
				numberOfOutputs: 1,
				channelCount: 1,
				channelCountMode: 'explicit',
				channelInterpretation: 'speakers',
			})
		} finally {
			if (previousMediaStream) globalThis.MediaStream = previousMediaStream
			else delete globalThis.MediaStream
			if (previousAudioWorkletNode) globalThis.AudioWorkletNode = previousAudioWorkletNode
			else delete globalThis.AudioWorkletNode
		}
	})

	// TEST-05: a source replacement with a different known rate is rejected instead of silently resampled.
	it('rejects a source replacement with a different known sample rate', async () => {
		const previousMediaStream = globalThis.MediaStream
		const previousAudioWorkletNode = globalThis.AudioWorkletNode

		class FakeMediaStream {
			constructor(tracks) { this.tracks = tracks }
			getTracks() { return this.tracks.slice() }
		}
		class FakeAudioContext {
			constructor(options = {}) {
				this.sampleRate = options.sampleRate || 48000
				this.state = 'running'
				this.audioWorklet = { addModule: jest.fn().mockResolvedValue(undefined) }
				this.destination = {}
			}
			createMediaStreamTrackSource = track => ({ track, connect: jest.fn(), disconnect: jest.fn() })
			createMediaStreamSource = stream => ({ stream, connect: jest.fn(), disconnect: jest.fn() })
			createMediaStreamDestination = () => ({})
			createGain = () => ({ gain: { value: 0 }, connect: jest.fn(), disconnect: jest.fn() })
			resume = jest.fn(async () => {})
			close = jest.fn(async () => {})
		}
		class FakeAudioWorkletNode {
			constructor() {
				this.port = { onmessage: null }
				this.connect = jest.fn()
				this.disconnect = jest.fn()
			}
		}
		globalThis.MediaStream = FakeMediaStream
		globalThis.AudioWorkletNode = FakeAudioWorkletNode

		try {
			const store = { beginCapture: jest.fn().mockResolvedValue(undefined) }
			const recorder = new Recorder({ AudioContextClass: FakeAudioContext, persistenceStoreFactory: () => store })
			const firstTrack = {
				id: 'pore-rate-48000',
				kind: 'audio',
				readyState: 'live',
				getSettings: () => ({ sampleRate: 48000 }),
			}
			const secondTrack = {
				id: 'pore-rate-44100',
				kind: 'audio',
				readyState: 'live',
				getSettings: () => ({ sampleRate: 44100 }),
			}

			await recorder.start(firstTrack, { captureId: 'capture-rate-2', recordingSessionId: 'session-rate-2' })
			await expect(recorder.replaceTrack(secondTrack)).rejects.toThrow(/44100 Hz while preserving 48000 Hz/)
		} finally {
			if (previousMediaStream) globalThis.MediaStream = previousMediaStream
			else delete globalThis.MediaStream
			if (previousAudioWorkletNode) globalThis.AudioWorkletNode = previousAudioWorkletNode
			else delete globalThis.AudioWorkletNode
		}
	})

})
