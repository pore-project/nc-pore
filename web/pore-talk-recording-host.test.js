import '../js/pore-talk-recording-host.js'

describe('Talk recording host adapter', () => {
	const Adapter = window.PoRETalkRecordingHostAdapter

	// TEST-01: Talk context JSON fields are normalized at the adapter boundary.
	it('normalizes Talk context snake_case fields before publishing coordinator state', async () => {
		const originalFetch = globalThis.fetch
		const originalLocation = window.location
		const originalCoordinator = window.__poreTalkRecordingCoordinator
		const originalChannel = window.__poreRecordingCoordinationChannel
		const contextEvents = []
		const onContext = event => contextEvents.push(event.detail)

		window.addEventListener('pore:recording-ui-context', onContext)
		window.location = { pathname: '/call/test-talk-token' }
		window.__poreTalkRecordingCoordinator = null
		window.__poreRecordingCoordinationChannel = {
			connect: jest.fn().mockResolvedValue(undefined),
			publish: jest.fn().mockResolvedValue(undefined),
		}

		const response = data => ({
			status: 200,
			ok: true,
			json: async () => ({ ocs: { meta: { status: 'ok' }, data } }),
		})

		globalThis.fetch = jest.fn(async target => {
			if (target.startsWith('/ocs/v2.php/apps/pore/v1/talk/context?')) {
				return response({
					protocol_version: 1,
					status: 'ok',
					actor_type: 'users',
					actor_id: 'user-1',
					display_name: 'Host',
					participant_type: 1,
					talk_session_id: 'talk-session-1',
					owner_id: 'user-1',
					guest: false,
					error_code: null,
				})
			}
			if (target === '/ocs/v2.php/apps/spreed/api/v4/room/test-talk-token') {
				return response({ displayName: 'Test room' })
			}
			if (target === '/ocs/v2.php/apps/spreed/api/v4/call/test-talk-token') {
				return response([
					{ actorType: 'users', actorId: 'user-1', participantType: 1, displayName: 'Host' },
				])
			}
			throw new Error(`Unexpected Talk adapter request: ${target}`)
		})

		try {
			await Adapter.bootstrap()

			expect(window.__poreTalkRecordingCoordinator).toEqual(expect.objectContaining({
				sessionId: 'test-talk-token',
				recordingId: 'recording-test-talk-token',
				actorId: 'user-1',
				recordingParticipantId: 'user-1',
				ownerId: 'user-1',
				participants: ['user-1'],
			}))
			expect(contextEvents).toHaveLength(1)
			expect(contextEvents[0]).toEqual(expect.objectContaining({
				productionId: 'test-talk-token',
				productionLabel: 'Test room',
				participantLabel: 'Host',
				role: 'host',
				guest: false,
				state: 'preparing',
			}))
			expect(window.__poreRecordingCoordinationChannel.connect).toHaveBeenCalledWith('test-talk-token', 'recording-test-talk-token')
		} finally {
			window.removeEventListener('pore:recording-ui-context', onContext)
			globalThis.fetch = originalFetch
			if (originalLocation === undefined) delete window.location
			else window.location = originalLocation
			if (originalCoordinator === undefined) delete window.__poreTalkRecordingCoordinator
			else window.__poreTalkRecordingCoordinator = originalCoordinator
			if (originalChannel === undefined) delete window.__poreRecordingCoordinationChannel
			else window.__poreRecordingCoordinationChannel = originalChannel
		}
	})
})
