import '../js/pore-talk-audio-connector.js'

describe('Nextcloud Talk audio connector', () => {
	const Connector = window.PoRETalkAudioCaptureConnector
	const eventName = window.PoRETalkAudioTrackEvent

	const createTrack = ({ id, cloneTrackId }) => {
		const listeners = new Map()
		const cloneStop = jest.fn()
		const cloneTrack = { id: cloneTrackId, kind: 'audio', readyState: 'live', stop: cloneStop }
		const track = {
			id, kind: 'audio', readyState: 'live',
			clone: jest.fn(() => cloneTrack),
			addEventListener: jest.fn((type, listener) => listeners.set(type, listener)),
			removeEventListener: jest.fn((type, listener) => {
				if (listeners.get(type) === listener) listeners.delete(type)
			}),
		}
		return { track, cloneTrack, cloneStop, end: () => listeners.get('ended')?.() }
	}

	const createTrackEnabler = (track) => {
		const listeners = new Map()
		const sink = { current: null }
		const enabler = {
			connectTrackSink: jest.fn((inputTrackId, trackSink) => {
				sink.current = trackSink
				trackSink.connectTrackSource(inputTrackId, enabler, 'default')
			}),
			disconnectTrackSink: jest.fn((inputTrackId, trackSink) => {
				trackSink.disconnectTrackSource(inputTrackId, enabler, 'default')
				sink.current = null
			}),
			getOutputTrack: jest.fn(() => track),
			on: jest.fn((event, handler) => listeners.set(event, handler)),
			off: jest.fn((event, handler) => {
				if (listeners.get(event) === handler) listeners.delete(event)
			}),
			emitTrack: (nextTrack) => {
				enabler.getOutputTrack.mockReturnValue(nextTrack)
				listeners.get('outputTrackSet')?.(enabler, 'default', nextTrack)
			},
			_sink: sink,
		}
		return enabler
	}

	const installTalk = trackEnabler => {
		window.OCA = { Talk: { SimpleWebRTC: { webrtc: { _audioTrackEnabler: trackEnabler } } } }
	}

	beforeEach(() => { window.OCA = undefined })

	it('clones the current TrackEnabler audio output', () => {
		const first = createTrack({ id: 'talk-a', cloneTrackId: 'pore-a' })
		const enabler = createTrackEnabler(first.track)
		const events = []
		installTalk(enabler)
		const connector = new Connector({ dispatchEvent: event => events.push(event) })

		expect(connector.attachToTalk()).toBe(true)
		expect(first.track.clone).toHaveBeenCalledTimes(1)
		expect(connector.getCurrentSourceTrack()).toBe(first.track)
		expect(connector.getCurrentCloneTrack()).toBe(first.cloneTrack)
		expect(events).toHaveLength(1)
		expect(events[0].type).toBe(eventName)
	})

	it('does not clone the same Talk track twice', () => {
		const first = createTrack({ id: 'talk-a', cloneTrackId: 'pore-a' })
		const enabler = createTrackEnabler(first.track)
		installTalk(enabler)
		const connector = new Connector({ dispatchEvent: jest.fn() })
		connector.attachToTalk()
		connector.attachToTalk()
		expect(first.track.clone).toHaveBeenCalledTimes(1)
	})

	it('stops the previous PoRE clone when Talk replaces the output track', () => {
		const first = createTrack({ id: 'talk-a', cloneTrackId: 'pore-a' })
		const second = createTrack({ id: 'talk-b', cloneTrackId: 'pore-b' })
		const enabler = createTrackEnabler(first.track)
		installTalk(enabler)
		const connector = new Connector({ dispatchEvent: jest.fn() })
		connector.attachToTalk()
		enabler.emitTrack(second.track)
		expect(first.cloneStop).toHaveBeenCalledTimes(1)
		expect(connector.getCurrentSourceTrack()).toBe(second.track)
		expect(connector.getCurrentCloneTrack()).toBe(second.cloneTrack)
	})

	it('stops its clone when the Talk source track ends', () => {
		const first = createTrack({ id: 'talk-a', cloneTrackId: 'pore-a' })
		const enabler = createTrackEnabler(first.track)
		installTalk(enabler)
		const connector = new Connector({ dispatchEvent: jest.fn() })
		connector.attachToTalk()
		first.end()
		expect(first.cloneStop).toHaveBeenCalledTimes(1)
		expect(connector.getCurrentSourceTrack()).toBeNull()
		expect(connector.getCurrentCloneTrack()).toBeNull()
	})

	it('disconnects cleanly without stopping the Talk source track', () => {
		const first = createTrack({ id: 'talk-a', cloneTrackId: 'pore-a' })
		const enabler = createTrackEnabler(first.track)
		installTalk(enabler)
		const connector = new Connector({ dispatchEvent: jest.fn() })
		connector.attachToTalk()
		connector.detachFromTalk()
		expect(enabler.disconnectTrackSink).toHaveBeenCalledTimes(1)
		expect(first.cloneStop).toHaveBeenCalledTimes(1)
		expect(connector.getCurrentSourceTrack()).toBeNull()
	})

	it('disposes the current clone', () => {
		const first = createTrack({ id: 'talk-a', cloneTrackId: 'pore-a' })
		const enabler = createTrackEnabler(first.track)
		installTalk(enabler)
		const connector = new Connector({ dispatchEvent: jest.fn() })
		connector.attachToTalk()
		connector.dispose()
		expect(first.cloneStop).toHaveBeenCalledTimes(1)
		expect(connector.getCurrentCloneTrack()).toBeNull()
	})
})
