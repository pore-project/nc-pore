import '../js/pore-talk-audio-connector.js'

describe('Nextcloud Talk audio connector', () => {
	const Connector = window.PoRETalkAudioCaptureConnector
	const eventName = window.PoRETalkAudioTrackEvent

	const createTrack = ({ id, deviceId = id, stop = jest.fn() }) => {
		const listeners = new Map()
		const track = {
			id,
			kind: 'audio',
			label: `Microphone ${id}`,
			readyState: 'live',
			getSettings: jest.fn(() => ({ deviceId })),
			stop,
			addEventListener: jest.fn((type, listener) => listeners.set(type, listener)),
			removeEventListener: jest.fn((type, listener) => {
				if (listeners.get(type) === listener) listeners.delete(type)
			}),
		}
		return { track, end: () => listeners.get('ended')?.() }
	}

	const createStream = track => ({
		getAudioTracks: jest.fn(() => [track]),
		getTracks: jest.fn(() => [track]),
	})

	const createTrackEnabler = track => {
		const listeners = new Map()
		const enabler = {
			connectTrackSink: jest.fn((inputTrackId, sink) => {
				sink.connectTrackSource(inputTrackId, enabler, 'default')
			}),
			disconnectTrackSink: jest.fn((inputTrackId, sink) => {
				sink.disconnectTrackSource(inputTrackId, enabler, 'default')
			}),
			getOutputTrack: jest.fn(() => track),
			on: jest.fn((event, handler) => listeners.set(event, handler)),
			off: jest.fn((event, handler) => {
				if (listeners.get(event) === handler) listeners.delete(event)
			}),
			emitTrack: nextTrack => {
				enabler.getOutputTrack.mockReturnValue(nextTrack)
				listeners.get('outputTrackSet')?.(enabler, 'default', nextTrack)
			},
		}
		return enabler
	}

	const installTalk = trackEnabler => {
		window.OCA = { Talk: { SimpleWebRTC: { webrtc: { _audioTrackEnabler: trackEnabler } } } }
	}

	beforeEach(() => { window.OCA = undefined })

	it('opens an independent capture for the Talk-selected microphone', async () => {
		const talk = createTrack({ id: 'talk-a', deviceId: 'device-a' })
		const capture = createTrack({ id: 'pore-a', deviceId: 'device-a' })
		const getUserMedia = jest.fn(async constraints => {
			expect(constraints).toEqual({ audio: {
				echoCancellation: false,
				noiseSuppression: false,
				autoGainControl: false,
				deviceId: { exact: 'device-a' },
			} })
			return createStream(capture.track)
		})
		const enabler = createTrackEnabler(talk.track)
		const events = []
		installTalk(enabler)
		const connector = new Connector({ dispatchEvent: event => events.push(event), getUserMedia })

		expect(connector.attachToTalk()).toBe(true)
		await Promise.resolve()

		expect(getUserMedia).toHaveBeenCalledTimes(1)
		expect(talk.track.stop).not.toHaveBeenCalled()
		expect(connector.getCurrentSourceTrack()).toBe(talk.track)
		expect(connector.getCurrentCaptureTrack()).toBe(capture.track)
		expect(events).toHaveLength(1)
		expect(events[0].type).toBe(eventName)
		expect(events[0].detail.track).toBe(capture.track)
		expect(events[0].detail.sourceTrack).toBe(talk.track)
	})

	it('does not open a second capture for the same Talk track', async () => {
		const talk = createTrack({ id: 'talk-a', deviceId: 'device-a' })
		const capture = createTrack({ id: 'pore-a', deviceId: 'device-a' })
		const getUserMedia = jest.fn(async () => createStream(capture.track))
		const enabler = createTrackEnabler(talk.track)
		installTalk(enabler)
		const connector = new Connector({ dispatchEvent: jest.fn(), getUserMedia })

		connector.attachToTalk()
		connector.attachToTalk()
		await Promise.resolve()

		expect(getUserMedia).toHaveBeenCalledTimes(1)
	})

	it('reopens independent capture when Talk replaces the microphone', async () => {
		const firstTalk = createTrack({ id: 'talk-a', deviceId: 'device-a' })
		const secondTalk = createTrack({ id: 'talk-b', deviceId: 'device-b' })
		const firstCapture = createTrack({ id: 'pore-a', deviceId: 'device-a' })
		const secondCapture = createTrack({ id: 'pore-b', deviceId: 'device-b' })
		const getUserMedia = jest.fn()
			.mockResolvedValueOnce(createStream(firstCapture.track))
			.mockResolvedValueOnce(createStream(secondCapture.track))
		const enabler = createTrackEnabler(firstTalk.track)
		installTalk(enabler)
		const connector = new Connector({ dispatchEvent: jest.fn(), getUserMedia })

		connector.attachToTalk()
		await Promise.resolve()
		enabler.emitTrack(secondTalk.track)
		await Promise.resolve()

		expect(getUserMedia).toHaveBeenCalledTimes(2)
		expect(getUserMedia.mock.calls[1][0].audio.deviceId).toEqual({ exact: 'device-b' })
		expect(firstCapture.track.stop).toHaveBeenCalledTimes(1)
		expect(connector.getCurrentCaptureTrack()).toBe(secondCapture.track)
	})

	it('ignores a stale capture result after a microphone replacement', async () => {
		const firstTalk = createTrack({ id: 'talk-a', deviceId: 'device-a' })
		const secondTalk = createTrack({ id: 'talk-b', deviceId: 'device-b' })
		const staleCapture = createTrack({ id: 'pore-a', deviceId: 'device-a' })
		const currentCapture = createTrack({ id: 'pore-b', deviceId: 'device-b' })
		let resolveFirst
		const firstPromise = new Promise(resolve => { resolveFirst = resolve })
		const getUserMedia = jest.fn()
			.mockReturnValueOnce(firstPromise)
			.mockResolvedValueOnce(createStream(currentCapture.track))
		const enabler = createTrackEnabler(firstTalk.track)
		installTalk(enabler)
		const connector = new Connector({ dispatchEvent: jest.fn(), getUserMedia })

		connector.attachToTalk()
		enabler.emitTrack(secondTalk.track)
		await Promise.resolve()
		resolveFirst(createStream(staleCapture.track))
		await Promise.resolve()

		expect(staleCapture.track.stop).toHaveBeenCalledTimes(1)
		expect(connector.getCurrentCaptureTrack()).toBe(currentCapture.track)
	})

	it('does not stop the Talk-owned source when detached', async () => {
		const talk = createTrack({ id: 'talk-a', deviceId: 'device-a' })
		const capture = createTrack({ id: 'pore-a', deviceId: 'device-a' })
		const getUserMedia = jest.fn(async () => createStream(capture.track))
		const enabler = createTrackEnabler(talk.track)
		installTalk(enabler)
		const connector = new Connector({ dispatchEvent: jest.fn(), getUserMedia })

		connector.attachToTalk()
		await Promise.resolve()
		connector.detachFromTalk()

		expect(capture.track.stop).toHaveBeenCalledTimes(1)
		expect(talk.track.stop).not.toHaveBeenCalled()
	})
})
