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

	it('prepares a finalized durable capture for transport without uploading it', async () => {
		const persisted = []
		const provenance = { schemaVersion: 1, capture: { sampleRate: 48000, sampleSize: 24, channelCount: 1, processing: { echoCancellation: false, noiseSuppression: false, autoGainControl: false } }, sourceSegments: [] }
		const store = {
			finalizeCapture: jest.fn(async (captureId, patch) => {
				persisted.push({ captureId, patch })
			}),
			getCapture: jest.fn(async captureId => ({
				manifest: {
					captureId,
					status: 'finalized',
					productionId: 'production-1',
					recordingId: 'recording-1',
					recordingSessionId: 'session-1',
					sampleRate: 48000,
					channels: 1,
					provenance,
				},
				chunks: [new Blob([new Uint8Array([0, 0, 0])])],
			})),
		}
		const job = new Job({ persistenceStoreFactory: () => store })
		const handler = jest.fn()
		window.addEventListener('pore:recording-transport-ready', handler)

		const descriptor = await job.enqueue({
			productionId: 'production-1',
			recordingId: 'recording-1',
			captureId: 'capture-1',
			recordingSessionId: 'session-1',
			provenance,
		})

		expect(store.finalizeCapture).toHaveBeenCalledTimes(2)
		expect(descriptor.captureId).toBe('capture-1')
		expect(descriptor.productionId).toBe('production-1')
		expect(descriptor.recordingId).toBe('recording-1')
		expect(descriptor.recordingSessionId).toBe('session-1')
		expect(descriptor.format).toBe('audio/wav')
		expect(descriptor.encoding).toBe('pcm_s24le')
		expect(descriptor.provenance).toEqual(provenance)
		expect(descriptor.blob).toBeInstanceOf(Blob)
		expect(handler).toHaveBeenCalledTimes(1)

		window.removeEventListener('pore:recording-transport-ready', handler)
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
