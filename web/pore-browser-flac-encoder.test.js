import '../js/pore-browser-flac-encoder.js'

describe('Browser FLAC encoder', () => {
	const Encoder = window.PoREBrowserFlacEncoder

	it('packs signed PCM24 samples and finalizes a FLAC stream through libFLAC', async () => {
		const previous = window.Flac
		const calls = { processed: [], deleted: 0 }
		const header = new Uint8Array(42)
		header.set([0x66, 0x4c, 0x61, 0x43, 0x80, 0x00, 0x00, 0x22])
		window.Flac = {
			isReady: () => true,
			create_libflac_encoder: jest.fn(() => 17),
			init_encoder_stream: jest.fn((_encoder, write, metadata) => { write(header); metadata({ min_framesize: 11, max_framesize: 29, md5sum: '00112233445566778899aabbccddeeff' }); return 0 }),
			FLAC__stream_encoder_process_interleaved: jest.fn((_encoder, samples, count) => { calls.processed.push({ samples, count }); return 1 }),
			FLAC__stream_encoder_finish: jest.fn(() => 1),
			FLAC__stream_encoder_delete: jest.fn(() => { calls.deleted += 1 }),
		}
		try {
			const encoder = await Encoder.create({ sampleRate: 48000, channels: 1, totalSamples: 4, compression: 5 })
			const live = encoder.encodePcm24Bytes(new Uint8Array([0, 0, 0, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f, 0, 0, 0x80]))
			const final = encoder.finish()
			const output = [...live, ...final].flatMap(chunk => [...chunk])
			expect(calls.processed).toHaveLength(1)
			expect(calls.processed[0].count).toBe(4)
			expect(calls.processed[0].samples).toEqual(new Int32Array([0, -1, 8388607, -8388608]))
			expect(output.slice(0, 4)).toEqual([0x66, 0x4c, 0x61, 0x43])
			expect(output.slice(26, 42)).toEqual([...Buffer.from('00112233445566778899aabbccddeeff', 'hex')])
			expect(calls.deleted).toBe(1)
		} finally { window.Flac = previous }
	})

	it('rejects non-sample-aligned packed PCM24 input before invoking libFLAC', async () => {
		const previous = window.Flac
		const process = jest.fn(() => 1)
		window.Flac = { isReady: () => true, create_libflac_encoder: jest.fn(() => 18), init_encoder_stream: jest.fn(() => 0), FLAC__stream_encoder_process_interleaved: process, FLAC__stream_encoder_delete: jest.fn(), FLAC__stream_encoder_finish: jest.fn(() => 1) }
		try {
			const encoder = await Encoder.create({ sampleRate: 48000, channels: 1, totalSamples: 1 })
			await expect(Promise.resolve().then(() => encoder.encodePcm24Bytes(new Uint8Array([1, 2])))).rejects.toThrow(/sample aligned/)
			expect(process).toHaveBeenCalledTimes(0)
			encoder.free()
		} finally { window.Flac = previous }
	})

	it('rejects unsupported channel counts at the FLAC boundary', async () => {
		const previous = window.Flac
		window.Flac = { isReady: () => true }
		try { await expect(Encoder.create({ sampleRate: 48000, channels: 2, totalSamples: 1 })).rejects.toThrow(/mono only/) }
		finally { window.Flac = previous }
	})
})
