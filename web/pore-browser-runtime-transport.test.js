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
		blob: new Blob([new Uint8Array(44)], { type: 'audio/wav' }),
	}

	function completionJob() {
		let state = null
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
		const receipt = await transport.transfer(descriptor)

		expect(receipt.file_id).toBe(42)
		expect(job.markCompleted).toHaveBeenCalledTimes(1)
		expect(fetchMock.mock.calls[0][0]).toContain('/finalized-artifact/prepare')
		expect(fetchMock.mock.calls[1][1].method).toBe('PUT')
		expect(fetchMock.mock.calls[1][0]).toContain('/public.php/dav/files/share-token/Host.wav')
		expect(fetchMock.mock.calls[1][1].headers.Authorization).toContain('anonymous:secret')
		expect(fetchMock.mock.calls[2][0]).toContain('/finalized-artifact/verify')
		expect(fetchMock.mock.calls[3][0]).toContain('/finalized-artifact/close')
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
