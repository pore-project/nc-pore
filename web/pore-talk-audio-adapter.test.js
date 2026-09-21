describe('Nextcloud Talk audio adapter', () => {
	const createTrack = (id, deviceId, clone = null) => {
		const track = {
			id,
			kind: 'audio',
			label: `PoRE microphone ${id}`,
			readyState: 'live',
			enabled: true,
			getSettings: jest.fn(() => ({ deviceId, sampleRate: 48000, channelCount: 1 })),
			addEventListener: jest.fn(),
			removeEventListener: jest.fn(),
			stop: jest.fn(),
		}
		track.clone = clone || jest.fn(() => createTrack(`${id}-clone`, deviceId))
		return track
	}

	const createTalkPipeline = initialTrack => {
		let outputTrack = initialTrack
		const source = {
			connectTrackSink: jest.fn((inputTrackId, sink) => {
				sink.connectTrackSource(inputTrackId, source, 'audio')
			}),
			disconnectTrackSink: jest.fn((inputTrackId, sink) => {
				sink.disconnectTrackSource(inputTrackId, source, 'audio')
			}),
			getOutputTrack: jest.fn(() => outputTrack),
		}
		const enabler = {
			connectTrackSource: jest.fn((_inputTrackId, poreSource) => {
				poreSource.on('outputTrackSet', () => {
					outputTrack = poreSource.getOutputTrack('audio')
				})
				outputTrack = poreSource.getOutputTrack('audio')
			}),
			disconnectTrackSource: jest.fn(),
		}
		return { source, enabler, setOutputTrack: track => { outputTrack = track } }
	}

	beforeEach(() => {
		window.OCA = undefined
		window.__poreLocalAudioCapture = undefined
	})

	it('keeps PoRE master ownership while replacing only Talk clones on a master change', async () => {
		const talkTrack = createTrack('talk-initial', 'device-a')
		const masterOne = createTrack('master-a', 'device-a')
		const masterTwo = createTrack('master-b', 'device-b')
		const pipeline = createTalkPipeline(talkTrack)

		window.OCA = {
			Talk: {
				SimpleWebRTC: {
					webrtc: {
						_mediaDevicesSource: pipeline.source,
						_audioTrackEnabler: pipeline.enabler,
					},
				},
			},
		}

		await import('../js/pore-talk-audio-adapter.js')
		const adapter = window.__poreTalkAudioAdapter
		expect(adapter.attachToTalk()).toBe(true)
		window.__poreLocalAudioCapture = { getCurrentTrack: () => masterOne }

		window.dispatchEvent(new CustomEvent('pore:recording-started'))
		const talkCloneOne = pipeline.source.getOutputTrack('audio')

		expect(talkCloneOne).not.toBe(masterOne)
		expect(talkCloneOne).not.toBe(talkTrack)
		expect(talkTrack.stop).toHaveBeenCalledTimes(1)
		expect(masterOne.stop).not.toHaveBeenCalled()
		expect(talkCloneOne.stop).toBeInstanceOf(Function)

		window.dispatchEvent(new CustomEvent('pore:recording-master-track-changed', {
			detail: { sequence: 1, previousTrack: masterOne, track: masterTwo, trackId: masterTwo.id, deviceId: 'device-b' },
		}))
		const talkCloneTwo = pipeline.source.getOutputTrack('audio')

		expect(talkCloneTwo).not.toBe(masterTwo)
		expect(talkCloneTwo).not.toBe(talkCloneOne)
		expect(talkCloneOne.stop).toHaveBeenCalledTimes(1)
		expect(masterOne.stop).not.toHaveBeenCalled()
		expect(masterTwo.stop).not.toHaveBeenCalled()

		adapter.clearMasterTrack()
		window.__poreLocalAudioCapture = undefined
		window.OCA = undefined
	})
})
