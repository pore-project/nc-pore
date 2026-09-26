import '../js/pore-browser-runtime-transport.js'

describe('Browser runtime transport', () => {
	const Transport = window.PoREBrowserRuntimeTransport

	const descriptor = {
		captureId: 'capture-1',
		recordingSessionId: 'session-1',
		productionId: 'production-1',
		recordingId: 'recording-1',
		productionLabel: 'Interview',
		participantLabel: 'Host',
		startedAt: '2026-09-14T15:00:00+02:00',
		size: 44,
		payloadSha256: 'a'.repeat(64),
		provenance: { schemaVersion: 1, capture: { sampleRate: 48000, sampleSize: 24, channelCount: 1, processing: { echoCancellation: false, noiseSuppression: false, autoGainControl: false } }, sourceSegments: [] },
		blob: new Blob([new Uint8Array(44)], { type: 'audio/wav' }),
	}

	function completionJob(initialState = null) {
		let state = initialState
		return {
			getTransportState: jest.fn(async () => state),
			updateTransportState: jest.fn(async (_captureId, patch) => {
				state = { ...(state || {}), ...patch }
			}),
			markCompleted: jest.fn(async (_captureId, patch) => {
				state = { ...(state || {}), ...patch, status: 'completed' }
			}),
		}
	}

	beforeEach(() => {
		jest.restoreAllMocks()
	})

	it('marks completion only after prepare, upload, verify and close', async () => {
		const job = completionJob()
		const fetchMock = jest.fn()
		fetchMock
			.mockResolvedValueOnce({ ok: true, status: 200, json: async () => ({ ocs: { data: {
				status: 'prepared',
				transfer_id: 'opaque-handle',
				upload_url: '/public.php/dav/files/share-token',
				upload_username: 'anonymous',
				upload_password: 'secret',
				filename: 'Host.wav',
			} } }) })
			.mockResolvedValueOnce({ ok: true, status: 201, json: async () => ({}) })
			.mockResolvedValueOnce({ ok: true, status: 200, json: async () => ({ ocs: { data: {
				status: 'verified', artifact_id: 'capture-1', file_id: 42, path: 'audio/2026/09/14 - 15:00 Interview - production-1/Host.wav', size: 44, sha256: 'a'.repeat(64),
			} } }) })
			.mockResolvedValueOnce({ ok: true, status: 200, json: async () => ({ ocs: { data: { status: 'closed' } } }) })
		global.fetch = fetchMock

		const transport = new Transport({ completionJob: job })
		const completedEvent = jest.fn()
		window.addEventListener('pore:recording-transport-completed', completedEvent)
		const receipt = await transport.transfer(descriptor)

		expect(receipt.file_id).toBe(42)
		expect(completedEvent).toHaveBeenCalledTimes(1)
		expect(completedEvent.mock.calls[0][0].detail.captureId).toBe('capture-1')
		window.removeEventListener('pore:recording-transport-completed', completedEvent)
		expect(job.markCompleted).toHaveBeenCalledTimes(1)
		expect(fetchMock.mock.calls[0][0]).toContain('/finalized-artifact/prepare')
		expect(fetchMock.mock.calls[0][1].body).toContain('recording_session_id=session-1')
		expect(fetchMock.mock.calls[0][1].body).toContain('capture_provenance=')
		expect(fetchMock.mock.calls[1][1].method).toBe('PUT')
		expect(fetchMock.mock.calls[1][0]).toContain('/public.php/dav/files/share-token/Host.wav')
		expect(fetchMock.mock.calls[1][1].headers.Authorization).toBe(`Basic ${btoa('anonymous:secret')}`)
		expect(fetchMock.mock.calls[1][1].headers['If-None-Match']).toBe('*')
		expect(fetchMock.mock.calls[2][0]).toContain('/finalized-artifact/verify')
		expect(fetchMock.mock.calls[3][0]).toContain('/finalized-artifact/close')
	})

	it('reuses an existing identical artifact without uploading', async () => {
		const job = completionJob()
		const fetchMock = jest.fn()
		fetchMock
			.mockResolvedValueOnce({ ok: true, status: 200, json: async () => ({ ocs: { data: {
				status: 'prepared', transfer_id: 'existing-handle', upload_url: '', upload_username: '', upload_password: '', filename: 'Host.wav', upload_required: false,
			} } }) })
			.mockResolvedValueOnce({ ok: true, status: 200, json: async () => ({ ocs: { data: {
				status: 'verified', artifact_id: 'capture-1', file_id: 17, path: 'audio/2026/09/.../Host.wav', size: 44, sha256: 'a'.repeat(64),
			} } }) })
			.mockResolvedValueOnce({ ok: true, status: 200, json: async () => ({ ocs: { data: { status: 'closed' } } }) })
		global.fetch = fetchMock

		const transport = new Transport({ completionJob: job })
		const receipt = await transport.transfer(descriptor)

		expect(receipt.file_id).toBe(17)
		expect(fetchMock).toHaveBeenCalledTimes(3)
		expect(fetchMock.mock.calls[0][0]).toContain('/finalized-artifact/prepare')
		expect(fetchMock.mock.calls[1][0]).toContain('/finalized-artifact/verify')
		expect(fetchMock.mock.calls[2][0]).toContain('/finalized-artifact/close')
	})

	it('reprepares after a concurrent upload collision instead of overwriting', async () => {
		const job = completionJob()
		const fetchMock = jest.fn()
		fetchMock
			.mockResolvedValueOnce({ ok: true, status: 200, json: async () => ({ ocs: { data: {
				status: 'prepared', transfer_id: 'first-handle', upload_url: '/public.php/dav/files/first-token', upload_username: 'anonymous', upload_password: 'secret-1', filename: 'Host.wav', upload_required: true,
			} } }) })
			.mockResolvedValueOnce({ ok: false, status: 412, json: async () => ({}) })
			.mockResolvedValueOnce({ ok: true, status: 200, json: async () => ({ ocs: { data: { status: 'closed' } } }) })
			.mockResolvedValueOnce({ ok: true, status: 200, json: async () => ({ ocs: { data: {
				status: 'prepared', transfer_id: 'second-handle', upload_url: '/public.php/dav/files/second-token', upload_username: 'anonymous', upload_password: 'secret-2', filename: 'Host (2).wav', upload_required: true,
			} } }) })
			.mockResolvedValueOnce({ ok: true, status: 201, json: async () => ({}) })
			.mockResolvedValueOnce({ ok: true, status: 200, json: async () => ({ ocs: { data: {
				status: 'verified', artifact_id: 'capture-1', file_id: 43, path: 'audio/2026/09/.../Host (2).wav', size: 44, sha256: 'a'.repeat(64),
			} } }) })
			.mockResolvedValueOnce({ ok: true, status: 200, json: async () => ({ ocs: { data: { status: 'closed' } } }) })
		global.fetch = fetchMock

		const transport = new Transport({ completionJob: job })
		const receipt = await transport.transfer(descriptor)

		expect(receipt.file_id).toBe(43)
		expect(fetchMock).toHaveBeenCalledTimes(7)
		expect(fetchMock.mock.calls[1][1].headers['If-None-Match']).toBe('*')
		expect(fetchMock.mock.calls[3][0]).toContain('/finalized-artifact/prepare')
		expect(fetchMock.mock.calls[4][0]).toContain('/second-token/Host%20(2).wav')
	})

	it('returns the stored receipt without repeating work after completion', async () => {
		const receipt = { status: 'verified', artifact_id: 'capture-1', file_id: 42, size: 44, sha256: 'a'.repeat(64) }
		const job = completionJob({ status: 'completed', receipt })
		const fetchMock = jest.fn()
		global.fetch = fetchMock
		const completedEvent = jest.fn()
		window.addEventListener('pore:recording-transport-completed', completedEvent)

		const transport = new Transport({ completionJob: job })
		await expect(transport.transfer(descriptor)).resolves.toEqual(receipt)

		expect(fetchMock).not.toHaveBeenCalled()
		expect(job.updateTransportState).not.toHaveBeenCalled()
		expect(job.markCompleted).not.toHaveBeenCalled()
		expect(completedEvent).not.toHaveBeenCalled()
		window.removeEventListener('pore:recording-transport-completed', completedEvent)
	})

	it('does not close or complete when remote verification fails', async () => {
		const job = completionJob()
		const fetchMock = jest.fn()
		fetchMock
			.mockResolvedValueOnce({ ok: true, status: 200, json: async () => ({ ocs: { data: {
				status: 'prepared', transfer_id: 'opaque-handle', upload_url: '/public.php/dav/files/share-token', upload_username: 'anonymous', upload_password: 'secret', filename: 'Host.wav',
			} } }) })
			.mockResolvedValueOnce({ ok: true, status: 201, json: async () => ({}) })
			.mockResolvedValueOnce({ ok: false, status: 500, json: async () => ({ ocs: { data: { status: 'rejected', error_code: 'nextcloud_transport_failed' } } }) })
		global.fetch = fetchMock

		const transport = new Transport({ completionJob: job })
		await expect(transport.transfer(descriptor)).rejects.toThrow('nextcloud_transport_failed')
		expect(job.markCompleted).not.toHaveBeenCalled()
		expect(fetchMock).toHaveBeenCalledTimes(3)
	})
})
