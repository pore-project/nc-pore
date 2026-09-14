import '../js/pore-talk-recording-state-bridge.js'
import '../js/pore-talk-audio-connector.js'
import '../js/pore-recording-controller.js'

describe('Talk recording state bridge', () => {
	const Bridge = window.PoRETalkRecordingStateBridge

	it('normalizes authoritative Core identity and state without creating a second state machine', () => {
		const bridge = new Bridge({ dispatchEvent: jest.fn() })
		const snapshot = bridge.publish({ productionId: 'production-42', recordingId: 'recording-17', role: 'host', state: 'recording', readyCount: 2, participantCount: 2, startedAt: '2026-09-04T10:00:00Z' })
		expect(snapshot.productionId).toBe('production-42')
		expect(snapshot.recordingId).toBe('recording-17')
		expect(snapshot.role).toBe('host')
		expect(snapshot.state).toBe('recording')
		expect(snapshot.readyCount).toBe(2)
		expect(snapshot.participantCount).toBe(2)
		expect(bridge.getSnapshot()).toBe(snapshot)
	})

	it('derives readiness from participant state when aggregate counts are absent', () => {
		const snapshot = window.PoRETalkRecordingStateNormalize({ role: 'participant', state: 'ready', participants: [{ ready: true }, { ready: false }] })
		expect(snapshot.readyCount).toBe(1)
		expect(snapshot.participantCount).toBe(2)
	})

	it('accepts the Core runtime phase field used by command snapshots', () => {
		const snapshot = window.PoRETalkRecordingStateNormalize({ role: 'host', phase: 'ready', participants: [{ ready: true }] })
		expect(snapshot.state).toBe('ready')
	})

	it('keeps listener semantics separate from recording state', () => {
		const snapshot = window.PoRETalkRecordingStateNormalize({ role: 'listener', state: 'recording' })
		expect(snapshot.listener).toBe(true)
		expect(snapshot.state).toBe('recording')
	})
})

describe('PoRE local audio capture', () => {
	it('opens its own getUserMedia stream for the Talk-selected device', async () => {
		const track = { kind: 'audio', id: 'pore-track', getSettings: () => ({ deviceId: 'mic-1', sampleRate: 48000 }) }
		const stream = { getAudioTracks: () => [track], getTracks: () => [track] }
		const getUserMedia = jest.fn().mockResolvedValue(stream)
		const capture = new window.PoRELocalAudioCapture({ mediaDevices: { getUserMedia } })
		const result = await capture.open('mic-1')
		expect(result).toBe(track)
		expect(getUserMedia).toHaveBeenCalledWith({ audio: expect.objectContaining({ deviceId: { exact: 'mic-1' }, echoCancellation: false, noiseSuppression: false, autoGainControl: false }) })
	})
})

describe('Talk microphone observer', () => {
	it('reports microphone identity changes without cloning or forwarding the Talk track', () => {
		const events = []
		const observer = new window.PoRETalkAudioCaptureConnector({ dispatchEvent: event => events.push(event) })
		const source = { connectTrackSink: jest.fn((input, sink) => { source.sink = sink }), disconnectTrackSink: jest.fn() }
		window.OCA = { Talk: { SimpleWebRTC: { webrtc: { _mediaDevicesSource: source } } } }
		const talkTrack = { id: 'talk-track-1', getSettings: () => ({ deviceId: 'mic-1' }) }
		const replacementTrack = { id: 'talk-track-2', getSettings: () => ({ deviceId: 'mic-2' }) }
		expect(observer.attachToTalk()).toBe(true)
		source.sink._onTrack(talkTrack)
		source.sink._onTrack(replacementTrack)
		expect(observer.getCurrentMicrophone().deviceId).toBe('mic-2')
		expect(events.map(event => event.type)).toEqual(['pore:talk-microphone', 'pore:talk-microphone'])
		expect(events[1].detail.previousDeviceId).toBe('mic-1')
		expect(talkTrack.clone).toBeUndefined()
		expect(replacementTrack.clone).toBeUndefined()
		observer.dispose()
		delete window.OCA
	})
})
