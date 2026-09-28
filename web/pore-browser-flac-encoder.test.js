import '../js/pore-browser-flac-encoder.js'

describe('Browser FLAC encoder boundary', () => {
	const Encoder = window.PoREBrowserFlacEncoder

	it('decodes packed PCM24 samples exactly before encoding', () => {
		const previous = window.Flac
		const calls = []
		try {
			window.Flac = {
				create_libflac_encoder: jest.fn(() => ({})),
				init_encoder_stream: jest.fn((encoder, write, metadata) => { encoder.write = write; encoder.metadata = metadata; return 0 }),
				FLAC__stream_encoder_process_interleaved: jest.fn((encoder, samples) => { calls.push([...samples]); return true }),
				FLAC__stream_encoder_finish: jest.fn(() => true),
				FLAC__stream_encoder_delete: jest.fn(),
			}
			const encoder = new Encoder({ sampleRate: 48000, channels: 1, totalSamples: 3 })
			encoder.encodePcm24Bytes(new Uint8Array([0x00,0x00,0x00,0xff,0xff,0x7f,0x00,0x00,0x80]))
			expect(calls).toHaveLength(1)
			expect(calls[0]).toEqual([0, 8388607, -8388608])
			encoder.free()
		} finally { window.Flac = previous }
	})

	it('rejects unsupported V1 sample rates and non-mono encoding', () => {
		const previous = window.Flac
		try {
			window.Flac = { create_libflac_encoder: jest.fn() }

			let sampleRateError = null
			try { new Encoder({ sampleRate: 32000, channels: 1, totalSamples: 1 }) } catch (error) { sampleRateError = error }
			expect(sampleRateError).toBeTruthy()
			expect(String(sampleRateError.message)).toContain('sample rate')

			let channelError = null
			try { new Encoder({ sampleRate: 48000, channels: 2, totalSamples: 1 }) } catch (error) { channelError = error }
			expect(channelError).toBeTruthy()
			expect(String(channelError.message)).toContain('mono only')

			expect(window.Flac.create_libflac_encoder).not.toHaveBeenCalled()
		} finally { window.Flac = previous }
	})

	it('patches final STREAMINFO fields without corrupting the format boundary', () => {
		const encoder = Object.create(Encoder.prototype)
		encoder.headerBuffer = new Uint8Array(42)
		encoder.headerBuffer.set([0x66, 0x4c, 0x61, 0x43], 0)
		encoder.headerBuffer[21] = 0xa0
		encoder.totalSamples = 0x100000001
		encoder.streamInfo = { min_framesize: 0x010203, max_framesize: 0x040506, md5sum: '00112233445566778899aabbccddeeff' }
		encoder._patchStreamInfo()
		expect([...encoder.headerBuffer.slice(0,4)]).toEqual([0x66,0x4c,0x61,0x43])
		expect([...encoder.headerBuffer.slice(12,18)]).toEqual([1,2,3,4,5,6])
		expect(encoder.headerBuffer[21] & 0x0f).toBe(1)
		expect([...encoder.headerBuffer.slice(22,26)]).toEqual([0,0,0,1])
		expect([...encoder.headerBuffer.slice(26,42)]).toEqual([0,17,34,51,68,85,102,119,136,153,170,187,204,221,238,255])
	})
	it('emits the FLAC header before frame output and exposes its final patched copy', () => {
		const encoder = Object.create(Encoder.prototype)
		encoder.pendingOutput = [
			new Uint8Array([0x66,0x4c,0x61,0x43, ...new Uint8Array(38)]),
			new Uint8Array([0xff,0xf8,0x00]),
		]
		encoder.headerBuffer = new Uint8Array(0)
		encoder.headerReady = false
		encoder.headerEmitted = false
		encoder.streamInfo = null
		const output = encoder._drain(false)
		expect([...output[0].slice(0,4)]).toEqual([0x66,0x4c,0x61,0x43])
		expect([...output[1]]).toEqual([0xff,0xf8,0x00])
		expect(encoder.headerEmitted).toBe(true)
		encoder.finished = true
		encoder.streamInfo = { min_framesize: 1, max_framesize: 3, md5sum: '00112233445566778899aabbccddeeff' }
		encoder._patchStreamInfo()
		const finalHeader = encoder.getFinalizedHeader()
		expect([...finalHeader.slice(0,4)]).toEqual([0x66,0x4c,0x61,0x43])
		expect(finalHeader).not.toBe(output[0])
	})

})
