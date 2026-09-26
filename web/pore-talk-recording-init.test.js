describe('Talk recording opening coordination', () => {
	// TEST-04: the host must advance from ready to opening from its authoritative
	// ready response because Talk sendToAll does not deliver the message to self.
	it('host triggers opening after its own ready response reaches all participants', async () => {
		const commands = []
		let mountedContext = null
		let recording = false

		const track = {
			id: 'pore-track-host',
			kind: 'audio',
			readyState: 'live',
			getSettings: () => ({ deviceId: 'device-host', sampleRate: 48000 }),
		}

		const state = ({ phase = 'preparing', ready = false, openingTriggered = false, openingConfirmed = false } = {}) => ({
			productionId: 'room-42',
			productionStatus: 'active',
			recordingId: 'recording-room-42',
			role: 'host',
			state: phase,
			listener: false,
			confirmed: openingConfirmed,
			ready,
			openingConfirmed: openingConfirmed,
			readyCount: ready ? 1 : 0,
			openingConfirmedCount: openingConfirmed ? 1 : 0,
			participantCount: 1,
			openingTriggered,
			participants: [{ id: 'host-1', ready, opening_confirmed: openingConfirmed }],
		})

		window.location = { pathname: '/call/room-42' }
		window.OC = { currentUser: { uid: 'host-1' } }
		window.PoRETalkRecordingStateNormalize = snapshot => snapshot

		window.PoRETalkAudioCaptureConnector = class {
			attachToTalk() { return true }
		}
		window.PoRELocalAudioCapture = class {
			getCurrentTrack() { return track }
			getCurrentDeviceId() { return 'device-host' }
			open() { return Promise.resolve(track) }
			stop() {}
		}
		window.PoREBrowserRecordingController = class {
			isRecording() { return recording }
			start() { recording = true; return Promise.resolve() }
			markOpeningSignet() {}
			waitForOpeningSignet() { return Promise.resolve() }
		}
		window.PoRETalkRecordingStateBridge = class {
			publish() {}
		}
		window.PoREBrowserCompletionJob = class {}
		window.PoREBrowserRuntimeTransport = class {}
		window.PoRETalkRecordingUi = {
			mount: context => { mountedContext = context },
		}
		window.PoRETalkRecordingHostAdapter = {
			bootstrap: async () => {
				window.__poreTalkRecordingCoordinator = {
					sessionId: 'room-42',
					recordingId: 'recording-room-42',
					actorId: 'host-1',
					command: async name => {
						commands.push(name)
						switch (name) {
						case 'snapshot':
							return { state: state() }
						case 'begin':
							return { state: state({ phase: 'recording' }) }
						case 'ready':
							return { state: state({ phase: 'recording', ready: true }) }
						case 'trigger_opening':
							return { state: state({ phase: 'opening', ready: true, openingTriggered: true }) }
						case 'confirm_opening':
							return { state: state({ phase: 'opening', ready: true, openingTriggered: true, openingConfirmed: true }) }
						default:
							return { state: state() }
						}
					},
				}
			},
			attachSignaling: () => true,
		}

		await import('../js/init.js')
		await Promise.resolve()
		await Promise.resolve()

		window.dispatchEvent(new CustomEvent('pore:talk-production-identity', {
			detail: { conversationId: 'room-42', productionLabel: 'Test room' },
		}))
		window.dispatchEvent(new CustomEvent('pore:recording-ui-context', {
			detail: {
				productionId: 'room-42',
				productionLabel: 'Test room',
				recordingId: 'recording-room-42',
				role: 'host',
				state: 'preparing',
				participantCount: 1,
				participants: [{ id: 'host-1', ready: false }],
			},
		}))

		expect(mountedContext?.onStart).toBeTruthy()
		await mountedContext.onStart()

		expect(commands.slice(-4)).toEqual(['begin', 'ready', 'trigger_opening', 'confirm_opening'])
		expect(recording).toBe(true)
	})
})
