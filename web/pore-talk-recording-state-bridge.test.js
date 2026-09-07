import '../js/pore-talk-recording-state-bridge.js'

describe('Talk recording state bridge', () => {
	const Bridge = window.PoRETalkRecordingStateBridge

	it('normalizes authoritative Core identity and state without creating a second state machine', () => {
		const bridge = new Bridge({ dispatchEvent: jest.fn() })
		const snapshot = bridge.publish({
			productionId: 'production-42',
			recordingId: 'recording-17',
			role: 'host',
			state: 'recording',
			readyCount: 2,
			participantCount: 2,
			startedAt: '2026-09-04T10:00:00Z',
		})

		expect(snapshot.productionId).toBe('production-42')
		expect(snapshot.recordingId).toBe('recording-17')
		expect(snapshot.role).toBe('host')
		expect(snapshot.state).toBe('recording')
		expect(snapshot.readyCount).toBe(2)
		expect(snapshot.participantCount).toBe(2)
		expect(bridge.getSnapshot()).toBe(snapshot)
	})

	it('derives readiness from participant state when aggregate counts are absent', () => {
		const snapshot = window.PoRETalkRecordingStateNormalize({
			role: 'participant',
			state: 'ready',
			participants: [{ ready: true }, { ready: false }],
		})

		expect(snapshot.readyCount).toBe(1)
		expect(snapshot.participantCount).toBe(2)
	})

	it('keeps listener semantics separate from recording state', () => {
		const snapshot = window.PoRETalkRecordingStateNormalize({
			role: 'listener',
			state: 'recording',
		})

		expect(snapshot.listener).toBe(true)
		expect(snapshot.state).toBe('recording')
	})
})
