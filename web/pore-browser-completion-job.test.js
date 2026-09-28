import '../js/pore-browser-completion-job.js'

describe('Browser completion job', () => {
	const Job = window.PoREBrowserCompletionJob

	it('re-emits a completed transport handoff when Core completion is still pending', async () => {
		const store = {
			listRecoverableCaptures: jest.fn(async () => [{
				captureId: 'capture-1',
				status: 'finalized',
				completionJob: {
					status: 'completed',
					coreCompletionStatus: 'pending',
					artifactId: 'capture-1',
					fileId: 42,
					path: 'PoRE/example.wav',
					size: 123,
					sha256: 'abc',
				},
			}]),
		}
		const job = new Job({ persistenceStoreFactory: () => store })
		const events = []
		const handler = event => events.push(event.detail)
		window.addEventListener('pore:recording-transport-completed', handler)

		await job.recover()

		expect(events).toHaveLength(1)
		expect(events[0].artifact_id).toBe('capture-1')
		expect(events[0].file_id).toBe(42)
		window.removeEventListener('pore:recording-transport-completed', handler)
	})

	it('retries cleanup for a capture whose Core completion was already acknowledged', async () => {
		const store = {
			listRecoverableCaptures: jest.fn(async () => [{
				captureId: 'capture-2',
				status: 'finalized',
				completionJob: { status: 'completed', coreCompletionStatus: 'completed' },
			}]),
			removeCapture: jest.fn(async () => {}),
		}
		const job = new Job({ persistenceStoreFactory: () => store })

		await job.recover()

		expect(store.removeCapture).toHaveBeenCalledTimes(1)
		expect(store.removeCapture).toHaveBeenCalledWith('capture-2')
	})

	it('removes the local capture only through the completion job cleanup boundary', async () => {
		const store = {
			removeCapture: jest.fn(async () => {}),
		}
		const job = new Job({ persistenceStoreFactory: () => store })

		await job.removeCapture('capture-1')

		expect(store.removeCapture).toHaveBeenCalledTimes(1)
		expect(store.removeCapture).toHaveBeenCalledWith('capture-1')
	})

	it('finalizes durable PCM as FLAC before transport', async () => {
		const previousEncoder = window.PoREBrowserFlacEncoder
		const finalized = []
		let committed = false
		const provenance = { schemaVersion: 1, capture: { sampleRate: 48000, sampleSize: 24, channelCount: 1, processing: { echoCancellation: false, noiseSuppression: false, autoGainControl: false } }, sourceSegments: [] }
		const pcmChunk = new Blob([new Uint8Array([0, 0, 0])])
		const store = {
			finalizeCapture: jest.fn(async () => {}),
			getCapture: jest.fn(async captureId => committed
				? { manifest: { captureId, status: 'finalized', storageFormat: 'flac', format: 'audio/flac', encoding: 'flac', sampleRate: 48000, channels: 1, productionId: 'production-1', recordingId: 'recording-1', recordingSessionId: 'session-1', provenance }, chunks: finalized.slice() }
				: { manifest: { captureId, status: 'finalized', storageFormat: 'pcm_s24le', encoding: 'pcm_s24le', sampleRate: 48000, channels: 1, productionId: 'production-1', recordingId: 'recording-1', recordingSessionId: 'session-1', provenance }, chunks: [pcmChunk] }),
			clearFinalizedPayload: jest.fn(async () => { finalized.length = 0 }),
			appendFinalizedChunk: jest.fn(async (_captureId, index, payload) => { finalized[index] = payload }),
			getFinalizedPayload: jest.fn(async captureId => ({ manifest: { captureId, status: 'finalized', storageFormat: 'pcm_s24le' }, chunks: finalized.slice() })),
			commitFinalizedPayload: jest.fn(async captureId => {
				committed = true
				return { captureId, storageFormat: 'flac', status: 'finalized' }
			}),
		}
		window.PoREBrowserFlacEncoder = class {
			constructor(options) { this.options = options }
			encodePcm24Bytes(bytes) { return [new Uint8Array([0x66, 0x4c, 0x61, 0x43, bytes.length])] }
			finish() { return [new Uint8Array([0x80])] }
			free() {}
		}
		try {
			const job = new Job({ persistenceStoreFactory: () => store })
			const handler = jest.fn()
			window.addEventListener('pore:recording-transport-ready', handler)
			const descriptor = await job.enqueue({ productionId: 'production-1', recordingId: 'recording-1', captureId: 'capture-1', recordingSessionId: 'session-1', provenance })
			expect(store.appendFinalizedChunk).toHaveBeenCalled()
			expect(store.commitFinalizedPayload).toHaveBeenCalledWith('capture-1', expect.objectContaining({ payloadSha256: expect.any(String) }))
			expect(committed).toBe(true)
			expect(descriptor.format).toBe('audio/flac')
			expect(descriptor.encoding).toBe('flac')
			expect(descriptor.sampleRate).toBe(48000)
			expect(descriptor.channels).toBe(1)
			expect(descriptor.provenance).toEqual(provenance)
			expect(descriptor.blob).toBeInstanceOf(Blob)
			expect(handler).toHaveBeenCalledTimes(1)
			window.removeEventListener('pore:recording-transport-ready', handler)
		} finally {
			window.PoREBrowserFlacEncoder = previousEncoder
		}
	})

	it('does not discard PCM persistence when FLAC finalization fails', async () => {
		const previousEncoder = window.PoREBrowserFlacEncoder
		let committed = false
		const store = {
			finalizeCapture: jest.fn(async () => {}),
			getCapture: jest.fn(async () => ({ manifest: { captureId: 'capture-2', status: 'finalized', storageFormat: 'pcm_s24le', encoding: 'pcm_s24le', sampleRate: 48000, channels: 1 }, chunks: [new Blob([new Uint8Array([0, 0, 0])])] })),
			clearFinalizedPayload: jest.fn(async () => {}),
			appendFinalizedChunk: jest.fn(async () => {}),
			getFinalizedPayload: jest.fn(async () => ({ manifest: { captureId: 'capture-2', status: 'finalized' }, chunks: [] })),
			commitFinalizedPayload: jest.fn(async () => { committed = true }),
		}
		window.PoREBrowserFlacEncoder = class {
			constructor() {}
			encodePcm24Bytes() { throw new Error('synthetic FLAC encoder failure') }
			finish() { return [] }
			free() {}
		}
		try {
			const job = new Job({ persistenceStoreFactory: () => store })
			await expect(job.enqueue({ productionId: 'production-1', recordingId: 'recording-1', captureId: 'capture-2', recordingSessionId: 'session-1' })).rejects.toThrow('synthetic FLAC encoder failure')
			expect(store.commitFinalizedPayload).not.toHaveBeenCalled()
			expect(committed).toBe(false)
		} finally {
			window.PoREBrowserFlacEncoder = previousEncoder
		}
	})

	it('persists transport state without losing existing completion-job fields', async () => {
		const store = {
			finalizeCapture: jest.fn(async () => {}),
			getCapture: jest.fn(async () => ({
				manifest: {
					status: 'finalized',
					completionJob: {
						status: 'prepared',
						payloadSha256: 'abc',
						size: 123,
					},
				},
			})),
		}
		const job = new Job({ persistenceStoreFactory: () => store })

		await job.updateTransportState('capture-1', {
			status: 'authorized',
			transferId: 'opaque-transfer-handle',
		})

		expect(store.finalizeCapture).toHaveBeenCalledWith('capture-1', {
			completionJob: {
				status: 'authorized',
				payloadSha256: 'abc',
				size: 123,
				transferId: 'opaque-transfer-handle',
			},
		})
	})

	it('rejects a handoff with aliased or missing identities', async () => {
		const job = new Job({ persistenceStoreFactory: () => ({}) })

		await expect(job.enqueue({
			productionId: 'production-1',
			recordingId: 'recording-1',
			captureId: 'production-1',
			recordingSessionId: 'session-1',
		})).rejects.toThrow()
	})
})
