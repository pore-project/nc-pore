import './pore-talk-recording-ui.js'

describe('Talk recording UI', () => {
	const Ui = window.PoRETalkRecordingUi

	it('keeps listener separate from the recording state machine', () => {
		expect(Ui.resolveStatus({ state: 'recording', listener: true })).toEqual(Ui.STATUS.listener)
	})

	it('uses the accepted visual semantics for the lifecycle', () => {
		expect(Ui.resolveStatus({ state: 'preparing' })).toEqual(Ui.STATUS.preparing)
		expect(Ui.resolveStatus({ state: 'ready', ready: true })).toEqual(Ui.STATUS.ready)
		expect(Ui.resolveStatus({ state: 'opening', ready: true })).toEqual(Ui.STATUS.opening)
		expect(Ui.resolveStatus({ state: 'recording', ready: true })).toEqual(Ui.STATUS.recording)
		expect(Ui.resolveStatus({ state: 'stopping' })).toEqual(Ui.STATUS.stopping)
		expect(Ui.resolveStatus({ state: 'done', confirmed: true })).toEqual(Ui.STATUS.confirmed)
		expect(Ui.resolveStatus({ state: 'error' })).toEqual(Ui.STATUS.error)
	})

	it('formats elapsed recording time without inventing precision', () => {
		expect(Ui.formatElapsed(0)).toBe('00:00')
		expect(Ui.formatElapsed(65)).toBe('01:05')
		expect(Ui.formatElapsed(3661)).toBe('61:01')
	})

	it('only exposes start/stop controls to the host', () => {
		const participant = Ui.create({ role: 'participant', state: 'recording', ready: true, elapsedSeconds: 12 })
		const host = Ui.create({
			role: 'host',
			state: 'recording',
			ready: true,
			participantCount: 2,
			readyCount: 2,
			onStop: jest.fn(),
		})

		expect(participant.querySelector('button')).toBeNull()
		expect(host.querySelector('button')?.textContent).toBe('Aufnahme beenden')
	})

	it('does not expose a recording UI to a non-recording member', () => {
		const none = Ui.create({ role: 'none', listener: true, state: 'recording' })
		expect(none.querySelector('button')).toBeNull()
		expect(none.textContent).toContain('Listener')
	})
})
