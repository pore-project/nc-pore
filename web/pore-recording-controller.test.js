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

	it('preserves production identity in the recording source metadata', async () => {
		const recorder = {
			start: jest.fn().mockResolvedValue(undefined),
			stop: jest.fn().mockResolvedValue({ kind: 'audio', format: 'audio/wav' }),
		}
		const controller = new Controller({ recorderFactory: () => recorder })

		await controller.start(createTrack(), { productionId: 'conversation-42' })
		const artifact = await controller.stop('host')

		expect(recorder.start).toHaveBeenCalledTimes(1)
		expect(artifact.source.productionId).toBe('conversation-42')
	})
})
