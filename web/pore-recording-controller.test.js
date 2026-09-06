import '../js/pore-recording-controller.js'

describe('Browser recording controller', () => {
	const Controller = window.PoREBrowserRecordingController

	const createTrack = () => ({
		kind: 'audio',
		id: 'pore-track-1',
		label: 'PoRE microphone',
		readyState: 'live',
		getSettings: () => ({ deviceId: 'device-1', sampleRate: 48000, channelCount: 1 }),
	})

	it('preserves production, recording and technical identities in source metadata', async () => {
		const recorder = {
			start: jest.fn().mockResolvedValue(undefined),
			stop: jest.fn().mockResolvedValue({ kind: 'audio', format: 'audio/wav' }),
		}
		const controller = new Controller({ recorderFactory: () => recorder })

		await controller.start(createTrack(), {
			productionId: 'conversation-42',
			recordingId: 'recording-17',
			captureId: 'capture-17',
			recordingSessionId: 'recorder-session-17',
		})
		const artifact = await controller.stop('host')

		expect(recorder.start).toHaveBeenCalledWith(createTrack(), {
			productionId: 'conversation-42',
			recordingId: 'recording-17',
			captureId: 'capture-17',
			recordingSessionId: 'recorder-session-17',
		})
		expect(recorder.start).toHaveBeenCalledTimes(1)
		expect(artifact.source.productionId).toBe('conversation-42')
		expect(artifact.source.recordingId).toBe('recording-17')
		expect(artifact.source.captureId).toBe('capture-17')
		expect(artifact.source.recordingSessionId).toBe('recorder-session-17')
	})

	it('generates distinct technical identities when the host does not provide them', async () => {
		const recorder = {
			start: jest.fn().mockResolvedValue(undefined),
			stop: jest.fn().mockResolvedValue({ kind: 'audio', format: 'audio/wav' }),
		}
		const controller = new Controller({ recorderFactory: () => recorder })

		await controller.start(createTrack(), {
			productionId: 'conversation-42',
			recordingId: 'recording-17',
		})
		const artifact = await controller.stop('host')

		expect(artifact.source.captureId).toBeTruthy()
		expect(artifact.source.recordingSessionId).toBeTruthy()
		expect(artifact.source.captureId).not.toBe('conversation-42')
		expect(artifact.source.captureId).not.toBe('recording-17')
		expect(artifact.source.recordingSessionId).not.toBe('conversation-42')
		expect(artifact.source.recordingSessionId).not.toBe('recording-17')
	})

	it('publishes the enriched artifact at the local finalization boundary', async () => {
		const recorder = {
			start: jest.fn().mockResolvedValue(undefined),
			stop: jest.fn().mockResolvedValue({ kind: 'audio', format: 'audio/wav' }),
		}
		const controller = new Controller({ recorderFactory: () => recorder })
		const handler = jest.fn()
		window.addEventListener('pore:recording-local-finalized', handler)

		await controller.start(createTrack(), {
			productionId: 'conversation-42',
			recordingId: 'recording-17',
		})
		const artifact = await controller.stop('host')

		expect(handler).toHaveBeenCalledTimes(1)
		expect(handler.mock.calls[0][0].detail).toEqual(artifact)
		expect(handler.mock.calls[0][0].detail.source.productionId).toBe('conversation-42')
		expect(handler.mock.calls[0][0].detail.source.recordingId).toBe('recording-17')

		window.removeEventListener('pore:recording-local-finalized', handler)
	})

	it('builds a persistence handoff without coupling the browser to a transport', async () => {
		const recorder = {
			start: jest.fn().mockResolvedValue(undefined),
			stop: jest.fn().mockResolvedValue({
				kind: 'audio', format: 'audio/wav', encoding: 'pcm_s24le', size: 12,
				sampleRate: 48000, channels: 1, blob: new Blob(['payload'], { type: 'audio/wav' }),
			}),
		}
		const controller = new Controller({ recorderFactory: () => recorder })

		await controller.start(createTrack(), {
			productionId: 'conversation-42',
			recordingId: 'recording-17',
			captureId: 'capture-17',
			recordingSessionId: 'recorder-session-17',
		})
		const artifact = await controller.stop('host')
		const handoff = controller.createPersistenceHandoff(artifact)

		expect(handoff.productionId).toBe('conversation-42')
		expect(handoff.recordingId).toBe('recording-17')
		expect(handoff.captureId).toBe('capture-17')
		expect(handoff.recordingSessionId).toBe('recorder-session-17')
		expect(handoff.blob).toBe(artifact.blob)
		expect(handoff.format).toBe('audio/wav')
		expect(handoff.encoding).toBe('pcm_s24le')
	})
})
