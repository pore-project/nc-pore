import '../js/pore-talk-audio-connector.js'

describe('Nextcloud Talk microphone observer', () => {
	const Connector = window.PoRETalkAudioCaptureConnector
	const eventName = window.PoRETalkMicrophoneEvent

	const createTrack = ({ id, deviceId, stop = jest.fn() }) => ({
		id,
		kind: 'audio',
		label: `Microphone ${id}`,
		readyState: 'live',
		getSettings: jest.fn(() => ({ deviceId })),
		stop,
	})

	const createSource = track => {
		const listeners = new Map()
		const source = {
			connectTrackSink: jest.fn((inputTrackId, sink) => {
				sink.connectTrackSource(inputTrackId, source, 'audio')
			}),
			disconnectTrackSink: jest.fn((inputTrackId, sink) => {
				sink.disconnectTrackSource(inputTrackId, source, 'audio')
			}),
			getOutputTrack: jest.fn(() => track),
			on: jest.fn((event, handler) => listeners.set(event, handler)),
			off: jest.fn((event, handler) => {
				if (listeners.get(event) === handler) listeners.delete(event)
			}),
			emitTrack: nextTrack => {
				source.getOutputTrack.mockReturnValue(nextTrack)
				listeners.get('outputTrackSet')?.(source, 'audio', nextTrack)
			},
		}
		return source
	}

	const installTalk = (source, conversationId = null) => {
		window.OCA = {
			Talk: {
				SimpleWebRTC: {
					webrtc: {
						_mediaDevicesSource: source,
						signaling: { currentRoomToken: conversationId },
					},
				},
			},
		}
	}

	beforeEach(() => {
		window.OCA = undefined
	})

	it('observes the current Talk microphone without owning the Talk track', () => {
		const talkTrack = createTrack({ id: 'talk-a', deviceId: 'device-a' })
		const source = createSource(talkTrack)
		const events = []
		installTalk(source)

		const connector = new Connector({
			dispatchEvent: event => events.push(event),
		})

		expect(connector.attachToTalk()).toBe(true)
		expect(connector.getCurrentMicrophone().deviceId).toBe('device-a')
		expect(events).toHaveLength(1)
		expect(events[0].type).toBe(eventName)
		expect(events[0].detail.previousDeviceId).toBeNull()
		expect(events[0].detail.deviceId).toBe('device-a')
		expect(talkTrack.stop).not.toHaveBeenCalled()
		expect(talkTrack.clone).toBeUndefined()
	})

	it('reports microphone changes and keeps both Talk tracks untouched', () => {
		const firstTrack = createTrack({ id: 'talk-a', deviceId: 'device-a' })
		const secondTrack = createTrack({ id: 'talk-b', deviceId: 'device-b' })
		const source = createSource(firstTrack)
		const events = []
		installTalk(source)

		const connector = new Connector({
			dispatchEvent: event => events.push(event),
		})

		expect(connector.attachToTalk()).toBe(true)
		source.emitTrack(secondTrack)

		expect(connector.getCurrentMicrophone()).toEqual(expect.objectContaining({
			deviceId: 'device-b',
		}))
		expect(events).toHaveLength(2)
		expect(events[1].detail.previousDeviceId).toBe('device-a')
		expect(events[1].detail.deviceId).toBe('device-b')
		expect(events[1].detail.changed).toBe(true)
		expect(firstTrack.stop).not.toHaveBeenCalled()
		expect(secondTrack.stop).not.toHaveBeenCalled()
	})

	it('does not emit duplicate selection events for the same microphone', () => {
		const talkTrack = createTrack({ id: 'talk-a', deviceId: 'device-a' })
		const source = createSource(talkTrack)
		const events = []
		installTalk(source)

		const connector = new Connector({
			dispatchEvent: event => events.push(event),
		})

		expect(connector.attachToTalk()).toBe(true)
		source.emitTrack(talkTrack)

		expect(events).toHaveLength(1)
	})

	it('publishes the provider-native Talk conversation identity', () => {
		const talkTrack = createTrack({ id: 'talk-a', deviceId: 'device-a' })
		const source = createSource(talkTrack)
		const events = []
		installTalk(source, 'conversation-42')

		const connector = new Connector({
			dispatchEvent: event => events.push(event),
		})

		expect(connector.getCurrentMicrophone()).toBeNull()
		expect(connector.attachToTalk()).toBe(true)
		expect(connector.getCurrentMicrophone().deviceId).toBe('device-a')
		expect(events.some(event => event.type === 'pore:talk-production-identity')).toBe(false)
	})
})