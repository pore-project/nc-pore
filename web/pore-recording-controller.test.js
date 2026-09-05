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

	it('preserves production and recording identity in the recording source metadata', async () => {
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

		expect(recorder.start).toHaveBeenCalledTimes(1)
		expect(artifact.source.productionId).toBe('conversation-42')
		expect(artifact.source.recordingId).toBe('recording-17')
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
})
