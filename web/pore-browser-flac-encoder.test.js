import '../js/pore-browser-flac-encoder.js'

describe('Browser FLAC encoder boundary', () => {
	const Encoder = window.PoREBrowserFlacEncoder
	const previousFlac = window.Flac

	afterEach(() => {
		window.Flac = previousFlac
	})

	// TEST-01: packed signed PCM24 must become exact signed Int32 samples.
	it('decodes packed PCM24 samples exactly before encoding', () => {
		const calls = []
		window.Flac = {
			create_libflac_encoder: jest.fn(() => ({})),
			init_encoder_stream: jest.fn((encoder, write, metadata) => {
				encoder.write = write
				encoder.metadata = metadata
				return 0
			}),
			FLAC__stream_encoder_process_interleaved: jest.fn((encoder, samples) => {
				calls.push(samples)
				return true
			}),
			FLAC__stream_encoder_delete: jest.fn(),
		}
		const encoder = new Encoder({ sampleRate: 48000, channels: 1, totalSamples: 3 })
		encoder.encodePcm24Bytes(new Uint8Array([
			0x00, 0x00, 0x00,
			0xff, 0xff, 0x7f,
			0x00, 0x00, 0x80,
		]))
		expect(calls).toHaveLength(1)
		expect([...calls[0]]).toEqual([0, 8388607, -8388608])
		encoder.free()
	})

	// TEST-02: V1 FLAC conversion is deliberately mono only.
	it('rejects a non-mono encoder configuration', () => {
		window.Flac = {
			create_libflac_encoder: jest.fn(),
		}
		expect(() => new Encoder({ sampleRate: 48000, channels: 2, totalSamples: 3 })).toThrow('mono only')
		expect(window.Flac.create_libflac_encoder).not.toHaveBeenCalled()
	})

	// TEST-03: final STREAMINFO fields are patched at their documented offsets.
	it('patches final STREAMINFO metadata without corrupting the format boundary', () => {
		const encoder = Object.create(Encoder.prototype)
		encoder.headerBuffer = new Uint8Array(42)
		encoder.headerBuffer.set([0x66, 0x4c, 0x61, 0x43], 0)
		encoder.headerBuffer[21] = 0xa0
		encoder.totalSamples = 0x100000001
		encoder.streamInfo = {
			min_framesize: 0x010203,
			max_framesize: 0x040506,
			md5sum: '00112233445566778899aabbccddeeff',
		}
		encoder._patchStreamInfo()
		expect([...encoder.headerBuffer.slice(0, 4)]).toEqual([0x66, 0x4c, 0x61, 0x43])
		expect([...encoder.headerBuffer.slice(12, 18)]).toEqual([0x01, 0x02, 0x03, 0x04, 0x05, 0x06])
		expect((encoder.headerBuffer[21] & 0x0f)).toBe(1)
		expect([...encoder.headerBuffer.slice(22, 26)]).toEqual([0x00, 0x00, 0x00, 0x01])
		expect([...encoder.headerBuffer.slice(26, 42)]).toEqual([
			0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
			0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff,
		])
	})
})
