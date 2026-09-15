import '../js/pore-recording-controller.js'
import '../js/pore-browser-completion-job.js'

describe('Browser recording lifecycle', () => {
	const Controller = window.PoREBrowserRecordingController
	const Job = window.PoREBrowserCompletionJob

	const createTrack = () => ({
		kind: 'audio',
		id: 'pore-track-1',
		label: 'PoRE microphone',
		readyState: 'live',
		getSettings: () => ({ deviceId: 'device-1', sampleRate: 48000, channelCount: 1 }),
	})

	it('hands a stopped recording through persistence to transport-ready without touching the transport', async () => {
		const capture = {
			manifest: {
				captureId: 'capture-17',
				productionId: 'production-1',
				recordingId: 'recording-1',
				recordingSessionId: 'session-1',
				sampleRate: 48000,
				channels: 1,
				status: 'finalized',
			},
			chunks: [new Blob([new Uint8Array([0, 0, 0, 1, 2, 3])])],
		}
		const store = {
			finalizeCapture: jest.fn(async (captureId, patch) => {
				capture.manifest = { ...capture.manifest, ...patch }
				return capture.manifest
			}),
			getCapture: jest.fn(async () => capture),
		}
		const completionJob = new Job({ persistenceStoreFactory: () => store })
		const recorder = {
			start: jest.fn().mockResolvedValue(undefined),
			stop: jest.fn().mockResolvedValue({
				kind: 'audio',
				format: 'audio/wav',
				encoding: 'pcm_s24le',
				size: 50,
			}),
		}
		const controller = new Controller({ recorderFactory: () => recorder })
		const transport = { transfer: jest.fn() }
		const handoffs = []
		const ready = []

		const handoffHandler = event => {
			const handoff = controller.createPersistenceHandoff(event.detail)
			handoffs.push(handoff)
			void completionJob.enqueue(handoff)
		}
		const readyHandler = event => ready.push(event.detail)
		window.addEventListener('pore:recording-local-finalized', handoffHandler)
		window.addEventListener('pore:recording-transport-ready', readyHandler)

		try {
			await controller.start(createTrack(), {
				productionId: 'production-1',
				recordingId: 'recording-1',
				captureId: 'capture-17',
				recordingSessionId: 'session-1',
			})
			await controller.stop('host')
			await new Promise(resolve => setTimeout(resolve, 0))
			await new Promise(resolve => setTimeout(resolve, 0))

			expect(handoffs).toHaveLength(1)
			expect(handoffs[0].blob).toBeUndefined()
			expect(store.finalizeCapture).toHaveBeenCalledWith('capture-17', expect.objectContaining({ completionJob: expect.objectContaining({ status: 'pending' }) }))
			expect(ready).toHaveLength(1)
			expect(ready[0].captureId).toBe('capture-17')
			expect(ready[0].productionId).toBe('production-1')
			expect(ready[0].recordingId).toBe('recording-1')
			expect(ready[0].recordingSessionId).toBe('session-1')
			expect(ready[0].blob).toBeInstanceOf(Blob)
			expect(ready[0].payloadSha256).toMatch(/^[0-9a-f]{64}$/)
			expect(transport.transfer).not.toHaveBeenCalled()
		} finally {
			window.removeEventListener('pore:recording-local-finalized', handoffHandler)
			window.removeEventListener('pore:recording-transport-ready', readyHandler)
		}
	})
})
