import '../js/pore-recording-controller.js'

describe('Recording controller persistence safety stop', () => {
	const Controller = window.PoREBrowserRecordingController

	it('routes recorder persistence safety stops through the existing local-stop boundary', async () => {
		let safetyStop = null
		const recorder = {
			start: jest.fn().mockResolvedValue(undefined),
			stop: jest.fn().mockResolvedValue(null),
		}
		const controller = new Controller({
			recorderFactory: options => {
				safetyStop = options.onPersistenceSafetyStop
				return recorder
			},
		})
		const handler = jest.fn()
		window.addEventListener('pore:recording-ui-stop-local', handler)

		await controller.start({ kind: 'audio', id: 'track-1', label: 'Mic', readyState: 'live', getSettings: () => ({}) })
		safetyStop({ captureId: 'capture-1', pendingChunks: 8, pendingBytes: 1048576 })

		expect(handler).toHaveBeenCalledTimes(1)
		expect(handler.mock.calls[0][0].detail.reason).toBe('persistence-safety-stop')
		expect(handler.mock.calls[0][0].detail.persistence.pendingChunks).toBe(8)

		window.removeEventListener('pore:recording-ui-stop-local', handler)
	})
})
