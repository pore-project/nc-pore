import { TalkAudioCaptureConnector, PORE_TALK_AUDIO_TRACK_EVENT } from './pore-talk-audio-connector.js'

describe('Nextcloud Talk audio connector', () => {
	const createTrack = ({ id, cloneTrackId }) => {
		const listeners = new Map()
		const cloneStop = jest.fn()
		const cloneTrack = { id: cloneTrackId, stop: cloneStop }
		const track = {
			id,
			clone: jest.fn(() => cloneTrack),
			addEventListener: jest.fn((type, listener) => listeners.set(type, listener)),
			removeEventListener: jest.fn((type, listener) => {
				if (listeners.get(type) === listener) {
					listeners.delete(type)
				}
			}),
		}

		return {
			track,
			cloneTrack,
			cloneStop,
			end: () => listeners.get('ended')?.(),
		}
	}

	it('clones the current audio track and leaves the Talk stream unchanged', () => {
		const first = createTrack({ id: 'talk-a', cloneTrackId: 'pore-a' })
		const stream = { getAudioTracks: () => [first.track] }
		const events = []
		const connector = new TalkAudioCaptureConnector({
			dispatchEvent: (event) => events.push(event),
		})

		const clone = connector.acceptStream(stream, { audio: true, video: true })

		expect(clone).toBe(first.cloneTrack)
		expect(first.track.clone).toHaveBeenCalledTimes(1)
		expect(connector.getCurrentSourceTrack()).toBe(first.track)
		expect(events).toHaveLength(1)
		expect(events[0].type).toBe(PORE_TALK_AUDIO_TRACK_EVENT)
		expect(events[0].detail.track).toBe(first.cloneTrack)
		expect(events[0].detail.sourceTrack).toBe(first.track)
	})

	it('does not clone the same Talk track twice', () => {
		const first = createTrack({ id: 'talk-a', cloneTrackId: 'pore-a' })
		const connector = new TalkAudioCaptureConnector({ dispatchEvent: jest.fn() })
		const stream = { getAudioTracks: () => [first.track] }

		const firstClone = connector.acceptStream(stream, { audio: true })
		const secondClone = connector.acceptStream(stream, { audio: true })

		expect(firstClone).toBe(secondClone)
		expect(first.track.clone).toHaveBeenCalledTimes(1)
	})

	it('stops the previous PoRE clone when Talk supplies a replacement track', () => {
		const first = createTrack({ id: 'talk-a', cloneTrackId: 'pore-a' })
		const second = createTrack({ id: 'talk-b', cloneTrackId: 'pore-b' })
		const connector = new TalkAudioCaptureConnector({ dispatchEvent: jest.fn() })

		connector.acceptStream({ getAudioTracks: () => [first.track] }, { audio: true })
		const replacement = connector.acceptStream({ getAudioTracks: () => [second.track] }, { audio: true })

		expect(first.cloneStop).toHaveBeenCalledTimes(1)
		expect(replacement).toBe(second.cloneTrack)
		expect(connector.getCurrentSourceTrack()).toBe(second.track)
	})

	it('stops its clone when the Talk source track ends', () => {
		const first = createTrack({ id: 'talk-a', cloneTrackId: 'pore-a' })
		const connector = new TalkAudioCaptureConnector({ dispatchEvent: jest.fn() })

		connector.acceptStream({ getAudioTracks: () => [first.track] }, { audio: true })
		first.end()

		expect(first.cloneStop).toHaveBeenCalledTimes(1)
		expect(connector.getCurrentSourceTrack()).toBeNull()
		expect(connector.getCurrentCloneTrack()).toBeNull()
	})

	it('ignores video-only capture requests', () => {
		const first = createTrack({ id: 'talk-a', cloneTrackId: 'pore-a' })
		const connector = new TalkAudioCaptureConnector({ dispatchEvent: jest.fn() })

		expect(connector.acceptStream(
			{ getAudioTracks: () => [first.track] },
			{ video: true },
		)).toBeNull()

		expect(first.track.clone).not.toHaveBeenCalled()
	})

	it('disposes the current clone', () => {
		const first = createTrack({ id: 'talk-a', cloneTrackId: 'pore-a' })
		const connector = new TalkAudioCaptureConnector({ dispatchEvent: jest.fn() })

		connector.acceptStream({ getAudioTracks: () => [first.track] }, { audio: true })
		connector.dispose()

		expect(first.cloneStop).toHaveBeenCalledTimes(1)
		expect(connector.getCurrentCloneTrack()).toBeNull()
	})
})
