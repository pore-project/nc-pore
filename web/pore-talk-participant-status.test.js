import '../js/pore-talk-recording-ui.js'
import '../js/pore-talk-participant-status.js'

describe('Talk participant recording status', () => {
	const Status = window.PoRETalkParticipantStatus

	// TEST-01: The participant indicator reuses the accepted recording-status semantics.
	it('uses the shared recording status semantics', () => {
		expect(Status.resolveStatus({ state: 'recording', ready: true })).toEqual(window.PoRETalkRecordingUi.STATUS.recording)
		expect(Status.resolveStatus({ state: 'ready', ready: true })).toEqual(window.PoRETalkRecordingUi.STATUS.ready)
		expect(Status.resolveStatus({ state: 'stopped', productionStatus: 'active' })).toEqual(window.PoRETalkRecordingUi.STATUS.stopped)
		expect(Status.resolveStatus({ state: 'completed', confirmed: true })).toEqual(window.PoRETalkRecordingUi.STATUS.confirmed)
	})

	// TEST-02: The small point remains semantically understandable without relying on color alone.
	it('provides an accessible label and a hover/focus detail for an active recording', () => {
		const indicator = Status.create({ state: 'recording', ready: true })
		document.body.appendChild(indicator)

		expect(indicator.getAttribute('role')).toBe('img')
		expect(indicator.tabIndex).toBe(0)
		expect(indicator.getAttribute('aria-label')).toBe('Aufnahme läuft')
		expect(indicator.title).toBe('Aufnahme läuft')

		indicator.dispatchEvent(new Event('mouseenter'))
		const popover = document.getElementById('pore-talk-participant-status-popover')
		expect(popover).not.toBeNull()
		expect(popover.textContent).toBe('Aufnahme läuft')

		Status.destroy(indicator)
		popover?.remove()
	})

	// TEST-03: Processing remains distinguishable from a finished/confirmed artifact.
	it('reports processing after stop without inventing a new domain state', () => {
		const indicator = Status.create({ state: 'stopped', productionStatus: 'active' })
		expect(indicator.getAttribute('aria-label')).toBe('Aufnahme beendet. Aufnahme beendet – Verarbeitung läuft.')
		expect(Status.getDetail({ state: 'stopped', productionStatus: 'active' })).toBe('Aufnahme beendet – Verarbeitung läuft.')
	})

	// TEST-04: A listener is still represented separately when a host sees the attendee.
	it('keeps non-recording attendees distinguishable from recording participants', () => {
		const indicator = Status.create({ state: 'recording', listener: true })
		expect(indicator.dataset.tone).toBe('listener')
		expect(indicator.getAttribute('aria-label')).toBe('Nicht beteiligt')
		Status.destroy(indicator)
	})
})
