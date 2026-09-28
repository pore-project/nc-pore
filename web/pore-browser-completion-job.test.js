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

	it('prepares an already-finalized FLAC capture for transport without uploading it', async () => {
		const persisted = []
		const provenance = { schemaVersion: 1, capture: { sampleRate: 48000, sampleSize: 24, channelCount: 1, processing: { echoCancellation: false, noiseSuppression: false, autoGainControl: false } }, sourceSegments: [] }
		const payload = new Blob([new Uint8Array([0x66, 0x4c, 0x61, 0x43, 0x00])], { type: 'audio/flac' })
		const store = {
			finalizeCapture: jest.fn(async (captureId, patch) => { persisted.push({ captureId, patch }) }),
			getCapture: jest.fn(async captureId => ({
				manifest: {
					captureId,
					status: 'finalized',
					storageFormat: 'flac',
					productionId: 'production-1',
					recordingId: 'recording-1',
					recordingSessionId: 'session-1',
					sampleRate: 48000,
					channels: 1,
					payloadSha256: 'a'.repeat(64),
					size: payload.size,
					provenance,
				},
				chunks: [payload],
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
		expect(descriptor.format).toBe('audio/flac')
		expect(descriptor.encoding).toBe('flac')
		expect(descriptor.sampleRate).toBe(48000)
		expect(descriptor.channels).toBe(1)
		expect(descriptor.blob).toBeInstanceOf(Blob)
		expect(handler).toHaveBeenCalledTimes(1)

		window.removeEventListener('pore:recording-transport-ready', handler)
	})

	it('finalizes packed PCM24 into a durable FLAC payload before transport', async () => {
		const committed = []
		const finalized = []
		const pcm = new Blob([new Uint8Array([0, 0, 0, 0xff, 0xff, 0x7f, 0, 0, 0x80])], { type: 'application/octet-stream' })
		const flac = new Blob([new Uint8Array([0x66, 0x4c, 0x61, 0x43, 0x01])], { type: 'audio/flac' })
		let isFlac = false
		const awaitedOutput = [0x66, 0x4c, 0x61, 0x43, 0x01]
		window.PoREBrowserFlacEncoder = jest.fn(() => ({
			encodePcm24Bytes: jest.fn(() => [new Uint8Array([])]),
			finish: jest.fn(() => [new Uint8Array(awaitedOutput)]),
			free: jest.fn(),
		}))
		const store = {
			finalizeCapture: jest.fn(async () => {}),
			getCapture: jest.fn(async captureId => isFlac ? ({
				manifest: {
					captureId, status: 'finalized', storageFormat: 'flac', productionId: 'production-1',
					recordingId: 'recording-1', recordingSessionId: 'session-1', sampleRate: 48000, channels: 1,
					payloadSha256: 'a'.repeat(64), size: flac.size,
				},
				chunks: [flac],
			}) : ({
				manifest: {
					captureId, status: 'finalized', storageFormat: 'pcm', productionId: 'production-1',
					recordingId: 'recording-1', recordingSessionId: 'session-1', sampleRate: 48000, channels: 1,
					encoding: 'pcm_s24le',
				},
				chunks: [pcm],
			})),
			clearFinalizedPayload: jest.fn(async () => finalized.splice(0)),
			appendFinalizedChunk: jest.fn(async (_captureId, index, payload) => finalized[index] = payload),
			getFinalizedPayload: jest.fn(async () => ({ manifest: { status: 'finalized', storageFormat: 'pcm' }, chunks: finalized.filter(Boolean) })),
			commitFinalizedPayload: jest.fn(async (captureId, patch) => {
				committed.push({ captureId, patch })
				isFlac = true
			}),
		}
		const job = new Job({ persistenceStoreFactory: () => store })
		const descriptor = await job.prepare('capture-1')

		expect(window.PoREBrowserFlacEncoder).toHaveBeenCalledWith({ sampleRate: 48000, channels: 1, totalSamples: 3, compression: 5 })
		expect(store.clearFinalizedPayload).toHaveBeenCalledWith('capture-1')
		expect(store.appendFinalizedChunk).toHaveBeenCalledTimes(1)
		const appended = store.appendFinalizedChunk.mock.calls[0]
		expect(appended[0]).toBe('capture-1')
		expect(appended[1]).toBe(0)
		expect(appended[2]).toBeInstanceOf(Uint8Array)
		expect(store.commitFinalizedPayload).toHaveBeenCalledTimes(1)
		expect(committed[0].patch.size).toBe(5)
		expect(committed[0].patch.sampleCount).toBe(3)
		expect(typeof committed[0].patch.payloadSha256).toBe('string')
		expect(descriptor.format).toBe('audio/flac')
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
