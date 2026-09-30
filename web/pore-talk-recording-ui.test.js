import '../js/pore-talk-recording-ui.js'

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
		expect(Ui.resolveStatus({ state: 'stopped', productionStatus: 'active' })).toEqual(Ui.STATUS.stopped)
		expect(Ui.resolveStatus({ state: 'completed', confirmed: true })).toEqual(Ui.STATUS.confirmed)
		expect(Ui.resolveStatus({ state: 'stopped', confirmed: true, productionStatus: 'completed' })).toEqual(Ui.STATUS.productionClosed)
		expect(Ui.resolveStatus({ state: 'error' })).toEqual(Ui.STATUS.error)
	})

	it('maps technical errors to non-technical user feedback', () => {
		expect(Ui.toUserFacingError({ name: 'NotAllowedError' })).toBe('Der Zugriff auf das Mikrofon wurde nicht freigegeben.')
		expect(Ui.toUserFacingError({ code: 'device_not_found' })).toBe('Es wurde kein verwendbares Mikrofon gefunden.')
		expect(Ui.toUserFacingError({ code: 'runtime_unavailable' })).toBe('Bei der Aufnahme ist ein Problem aufgetreten. Bitte den Vorgang erneut versuchen.')
	})

	it('formats elapsed recording time without inventing precision', () => {
		expect(Ui.formatElapsed(0)).toBe('00:00')
		expect(Ui.formatElapsed(65)).toBe('01:05')
		expect(Ui.formatElapsed(3661)).toBe('61:01')
	})

	it('derives live elapsed time from the technical start timestamp', () => {
		expect(Ui.elapsedSecondsFromStartedAt('2026-09-14T15:00:00.000Z', Date.parse('2026-09-14T15:01:05.900Z'))).toBe(65.9)
		expect(Ui.elapsedSecondsFromStartedAt(null, Date.now(), 12)).toBe(12)
	})

	it('exposes the compact Talk-like control and host recording action', () => {
		const participant = Ui.create({ role: 'participant', state: 'recording', ready: true, elapsedSeconds: 12 })
		const host = Ui.create({
			role: 'host',
			state: 'recording',
			ready: true,
			participantCount: 2,
			readyCount: 2,
			onStop: jest.fn(),
		})

		expect(participant.querySelector('[aria-label="NC-PoRE: Aufnahme läuft"]')).not.toBeNull()
		expect(participant.textContent).toContain('Aufnahme')
		expect(participant.textContent).not.toContain('Aufnahme beenden')
		expect(host.querySelector('[aria-label="NC-PoRE: Aufnahme läuft"]')).not.toBeNull()
		expect(host.querySelector('[aria-label="NC-PoRE öffnen"]')).not.toBeNull()
		expect(host.textContent).toContain('Aufnahme beenden')
	})

	it('shows the host start action only while preparing', () => {
		const host = Ui.create({ role: 'host', state: 'preparing', participantCount: 1, readyCount: 0, onStart: jest.fn() })
		expect(host.textContent).toContain('Aufnahme starten')
	})

	it('does not expose recording information to a non-participant', () => {
		const listener = Ui.create({ role: 'listener', state: 'recording', ready: false })
		expect(listener.hidden).toBe(true)
		expect(listener.getAttribute('aria-hidden')).toBe('true')
		expect(listener.textContent).not.toContain('Aufnahme läuft')
	})

	it('shows processing feedback after recording stop without inventing a stopping state', () => {
		const stopped = Ui.create({ role: 'participant', state: 'stopped', productionStatus: 'active' })
		expect(stopped.textContent).toContain('Aufnahme beendet')
		expect(stopped.textContent).toContain('Verarbeitung läuft')
	})

	it('shows user-facing error feedback without requiring a technical state transition', () => {
		const root = Ui.create({ role: 'participant', state: 'recording', errorMessage: 'Bei der Aufnahme ist ein Problem aufgetreten. Bitte den Vorgang erneut versuchen.' })
		const error = root.querySelector('[role="alert"]')
		expect(error).not.toBeNull()
		expect(error.textContent).toContain('Bei der Aufnahme ist ein Problem aufgetreten')
	})

	it('does not expose a recording action to a non-host member', () => {
		const participant = Ui.create({ role: 'participant', state: 'preparing', participantCount: 1, onStart: jest.fn() })
		expect(participant.textContent).not.toContain('Aufnahme starten')
		expect(participant.textContent).not.toContain('Aufnahme beenden')
	})
})
