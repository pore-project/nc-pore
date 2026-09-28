import { createRequire } from 'node:module'
import '../js/pore-browser-flac-encoder.js'

const require = createRequire(import.meta.url)
const RealFlac = require('../js/vendor/libflac.js')

describe('Browser FLAC encoder real-library smoke test', () => {
	const Encoder = window.PoREBrowserFlacEncoder

	it('produces a real V1 FLAC stream with the expected STREAMINFO profile', () => {
		if (typeof RealFlac?.isReady === 'function' && !RealFlac.isReady()) throw new Error('Vendored libFLAC is not ready')
		const previous = window.Flac
		try {
			window.Flac = RealFlac
			const pcm = new Uint8Array([
				0x00, 0x00, 0x00,
				0xff, 0xff, 0x7f,
				0x00, 0x00, 0x80,
			])
			const encoder = new Encoder({ sampleRate: 48000, channels: 1, totalSamples: 3, compression: 5 })
			const output = [...encoder.encodePcm24Bytes(pcm), ...encoder.finish()]
			const bytes = new Uint8Array(output.reduce((total, chunk) => total + chunk.length, 0))
			let offset = 0
			for (const chunk of output) {
				bytes.set(chunk, offset)
				offset += chunk.length
			}

			expect([...bytes.slice(0, 4)]).toEqual([0x66, 0x4c, 0x61, 0x43])
			expect([...bytes.slice(4, 8)]).toEqual([0x00, 0x00, 0x00, 0x22])

			const streamInfo = bytes.slice(8, 42)
			const sampleRate = (streamInfo[10] << 12) | (streamInfo[11] << 4) | (streamInfo[12] >> 4)
			const channels = ((streamInfo[12] >> 1) & 0x07) + 1
			const bitsPerSample = (((streamInfo[12] & 0x01) << 4) | (streamInfo[13] >> 4)) + 1
			const totalSamples = ((streamInfo[13] & 0x0f) * 0x100000000) + ((streamInfo[14] << 24) >>> 0) + (streamInfo[15] << 16) + (streamInfo[16] << 8) + streamInfo[17]

			expect(sampleRate).toBe(48000)
			expect(channels).toBe(1)
			expect(bitsPerSample).toBe(24)
			expect(totalSamples).toBe(3)
			expect(bytes.length).toBeGreaterThan(42)
	})
})
