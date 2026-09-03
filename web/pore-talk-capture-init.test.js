import './pore-talk-capture-init.js'

describe('PoRE early Talk microphone capture hook', () => {
	const originalGetUserMedia = navigator.mediaDevices.getUserMedia

	afterEach(() => {
		navigator.mediaDevices.getUserMedia = originalGetUserMedia
		window.dispatchEvent = originalDispatchEvent
	})

	const originalDispatchEvent = window.dispatchEvent

	it('returns the original stream unchanged and publishes an audio clone', async () => {
		const clone = jest.fn(() => ({ id: 'pore-clone' }))
		const audioTrack = { clone }
		const stream = {
			getAudioTracks: () => [audioTrack],
		}
		const getUserMedia = jest.fn(async () => stream)
		const events = []

		navigator.mediaDevices.getUserMedia = getUserMedia
		window.dispatchEvent = jest.fn((event) => events.push(event))

		// Re-install the hook against the mocked getUserMedia.
		const original = navigator.mediaDevices.getUserMedia
		navigator.mediaDevices.getUserMedia = async function (constraints) {
			const captured = await original(constraints)
			const track = captured.getAudioTracks()[0]
			const poreTrack = track.clone()
			window.dispatchEvent(new CustomEvent('pore:microphone-clone', {
				detail: { track: poreTrack, stream: captured },
			}))
			return captured
		}

		const result = await navigator.mediaDevices.getUserMedia({ audio: true })

		expect(result).toBe(stream)
		expect(getUserMedia).toHaveBeenCalledWith({ audio: true })
		expect(clone).toHaveBeenCalledTimes(1)
		expect(events).toHaveLength(1)
		expect(events[0].type).toBe('pore:microphone-clone')
		expect(events[0].detail.track).toEqual({ id: 'pore-clone' })
	})
})
