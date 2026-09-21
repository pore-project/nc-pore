import '../js/pore-recording-controller.js'

describe('Browser recording controller', () => {
	const Controller = window.PoREBrowserRecordingController

	const createTrack = (id = 'pore-track-1', deviceId = 'device-1') => ({
		kind: 'audio',
		id,
		label: 'PoRE microphone',
		readyState: 'live',
		getSettings: () => ({ deviceId, sampleRate: 48000, channelCount: 1 }),
	})

	it('preserves production, recording and technical identities in source metadata', async () => {
		const recorder = {
			start: jest.fn().mockResolvedValue(undefined),
			stop: jest.fn().mockResolvedValue({ kind: 'audio', format: 'audio/wav' }),
		}
		const controller = new Controller({ recorderFactory: () => recorder })
		const track = createTrack()

		const started = jest.fn()
		window.addEventListener('pore:recording-started', started)

		await controller.start(track, {
			productionId: 'conversation-42',
			recordingId: 'recording-17',
			captureId: 'capture-17',
			recordingSessionId: 'recorder-session-17',
		})
		const artifact = await controller.stop('host')

		expect(started).toHaveBeenCalledTimes(1)
		expect(started.mock.calls[0][0].detail.source.captureId).toBe('capture-17')
		expect(started.mock.calls[0][0].detail.startedAt).toBeTruthy()
		window.removeEventListener('pore:recording-started', started)

		expect(recorder.start).toHaveBeenCalledWith(track, {
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
				sampleRate: 48000, channels: 1,
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
		expect(handoff.blob).toBeUndefined()
		expect(handoff.format).toBe('audio/wav')
		expect(handoff.encoding).toBe('pcm_s24le')
	})

	it('replaces the microphone without ending the technical capture', async () => {
		const firstTrack = createTrack('pore-track-1', 'device-1')
		const secondTrack = createTrack('pore-track-2', 'device-2')
		const recorder = {
			start: jest.fn().mockResolvedValue(undefined),
			replaceTrack: jest.fn(track => Promise.resolve(track)),
			stop: jest.fn().mockResolvedValue({ kind: 'audio', format: 'audio/wav' }),
		}
		const controller = new Controller({ recorderFactory: () => recorder })

		await controller.start(firstTrack, { productionId: 'conversation-42', recordingId: 'recording-17' })
		const masterChanges = []
		window.addEventListener('pore:recording-master-track-changed', event => masterChanges.push(event.detail))
		controller.noteSourceChange(firstTrack, secondTrack, '2026-09-16T08:00:10.000Z', { from: { deviceId: 'device-1' }, to: { deviceId: 'device-2' } })
		await controller.replaceTrack(secondTrack)
		expect(masterChanges).toHaveLength(1)
		expect(masterChanges[0].previousTrack).toBe(firstTrack)
		expect(masterChanges[0].track).toBe(secondTrack)
		expect(masterChanges[0].trackId).toBe('pore-track-2')

		await controller.replaceTrack(firstTrack)
		expect(masterChanges).toHaveLength(2)
		expect(masterChanges[1].previousTrack).toBe(secondTrack)
		expect(masterChanges[1].track).toBe(firstTrack)
		expect(masterChanges[1].trackId).toBe('pore-track-1')

		expect(controller.getState()).toBe('recording')
		expect(recorder.replaceTrack).toHaveBeenCalledTimes(1)
		expect(recorder.stop).not.toHaveBeenCalled()
		expect(controller.sourceChanges).toHaveLength(1)
		expect(controller.sourceChanges[0].from.deviceId).toBe('device-1')
		expect(controller.sourceChanges[0].to.deviceId).toBe('device-2')
	})
})
