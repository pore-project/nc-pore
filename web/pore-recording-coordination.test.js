import '../js/pore-recording-coordination.js'

describe('PoRE recording coordination channel', () => {
	const Channel = window.PoRERecordingCoordinationChannel

	class FakeEventSource {
		static instances = []

		constructor(target) {
			this.target = target
			this.readyState = 0
			this.listeners = new Map()
			FakeEventSource.instances.push(this)
		}

		addEventListener(type, handler) {
			if (!this.listeners.has(type)) this.listeners.set(type, new Set())
			this.listeners.get(type).add(handler)
		}

		close() {
			this.readyState = 2
		}

		emit(type, event = {}) {
			for (const handler of this.listeners.get(type) || []) handler(event)
		}

		open() {
			this.readyState = 1
			this.emit('open')
		}

		fail() {
			this.readyState = 0
			this.emit('error')
		}
	}

	beforeEach(() => {
		FakeEventSource.instances = []
		window.EventSource = FakeEventSource
		jest.restoreAllMocks()
	})

	it('connects independently of Talk and forwards PoRE events', async () => {
		const channel = new Channel()
		const received = jest.fn()
		window.addEventListener('pore:recording-signal', received)

		const ready = channel.connect('session-1', 'recording-1')
		expect(FakeEventSource.instances).toHaveLength(1)
		FakeEventSource.instances[0].open()
		await ready

		FakeEventSource.instances[0].emit('pore-recording', {
			data: JSON.stringify({
				id: 1,
				version: 1,
				type: 'begin',
				sessionId: 'session-1',
				recordingId: 'recording-1',
				actorId: 'host-1',
			}),
		})

		expect(received).toHaveBeenCalledTimes(1)
		expect(received.mock.calls[0][0].detail.type).toBe('begin')
		expect(received.mock.calls[0][0].detail.from).toBe('host-1')
		window.removeEventListener('pore:recording-signal', received)
		channel.disconnect()
	})

	it('publishes lifecycle events through the PoRE endpoint, not a host transport', async () => {
		const fetchMock = jest.fn().mockResolvedValue({
			ok: true,
			status: 200,
			json: async () => ({ ocs: { meta: { status: 'ok' }, data: { status: 'published' } } }),
		})
		global.fetch = fetchMock

		const channel = new Channel()
		channel.connect('session-1', 'recording-1')
		await channel.publish('opening')

		expect(fetchMock).toHaveBeenCalledTimes(1)
		expect(fetchMock.mock.calls[0][0]).toContain('/recordings/coordination/publish')
		expect(fetchMock.mock.calls[0][1].method).toBe('POST')
		expect(String(fetchMock.mock.calls[0][1].body)).toContain('eventType=opening')
		channel.disconnect()
	})

	it('waits for a real reconnect after an established stream is lost', async () => {
		const channel = new Channel()
		const ready = channel.connect('session-1', 'recording-1')
		const source = FakeEventSource.instances[0]
		source.open()
		await ready

		source.fail()
		expect(channel.connected).toBe(false)
		const reconnectReady = channel.waitUntilReady(1000)

		source.open()
		await reconnectReady
		expect(channel.connected).toBe(true)
		channel.disconnect()
	})
})
