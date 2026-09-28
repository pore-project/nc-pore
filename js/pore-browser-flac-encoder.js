/* NC-PoRe — exact 24-bit PCM to FLAC encoder boundary. */
(() => {
	'use strict'

	const MAX_24 = 8388607
	const MIN_24 = -8388608
	const HEADER_BYTES = 42

	class PoREBrowserFlacEncoder {
		constructor({ sampleRate, channels = 1, totalSamples = 0, compression = 5 } = {}) {
			if (!Number.isFinite(sampleRate) || sampleRate <= 0) throw new Error('PoRE FLAC encoder requires a valid sample rate')
			if (channels !== 1) throw new Error('PoRE V1 FLAC encoder supports mono only')
			if (!window.Flac) throw new Error('PoRE FLAC encoder library is not available')
			this.Flac = window.Flac
			this.sampleRate = sampleRate
			this.channels = channels
			this.totalSamples = totalSamples
			this.compression = compression
			this.encoder = this.Flac.create_libflac_encoder(sampleRate, channels, 24, compression, totalSamples, false, 0)
			if (!this.encoder) throw new Error('PoRE FLAC encoder creation failed')
			this.pendingOutput = []
			this.headerBuffer = new Uint8Array(0)
			this.headerReady = false
			this.streamInfo = null
			this.finished = false
			const status = this.Flac.init_encoder_stream(
				this.encoder,
				data => this._write(data),
				summary => { this.streamInfo = summary },
				0,
			)
			if (status !== 0) {
				this._destroy()
				throw new Error('PoRE FLAC encoder initialization failed: ' + status)
			}
		}

		_encodeInterleaved(samples) {
			if (this.finished) throw new Error('PoRE FLAC encoder is already finished')
			if (!(samples instanceof Int32Array)) throw new Error('PoRE FLAC encoder requires Int32Array samples')
			if (samples.length === 0) return []
			if (!this.Flac.FLAC__stream_encoder_process_interleaved(this.encoder, samples, samples.length)) {
				throw new Error('PoRE FLAC encoding failed')
			}
			return this._drain(false)
		}

		encodePcm24Bytes(bytes) {
			const input = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes)
			if (input.length % 3 !== 0) throw new Error('PoRE packed PCM24 payload is not sample aligned')
			const samples = new Int32Array(input.length / 3)
			for (let i = 0, o = 0; i < input.length; i += 3, o += 1) {
				let value = input[i] | (input[i + 1] << 8) | (input[i + 2] << 16)
				if (value & 0x800000) value |= 0xff000000
				if (value < MIN_24 || value > MAX_24) throw new Error('PoRE packed PCM24 sample is outside the supported range')
				samples[o] = value
			}
			return this._encodeInterleaved(samples)
		}

		finish() {
			if (this.finished) return []
			this.finished = true
			if (!this.Flac.FLAC__stream_encoder_finish(this.encoder)) {
				this._destroy()
				throw new Error('PoRE FLAC encoder finalization failed')
			}
			const output = this._drain(true)
			this._destroy()
			return output
		}

		free() {
			this.finished = true
			this._destroy()
		}

		_write(data) {
			const copy = new Uint8Array(data.length)
			copy.set(data)
			this.pendingOutput.push(copy)
		}

		_drain(final) {
			let output = this.pendingOutput
			this.pendingOutput = []

			if (!this.headerReady) {
				while (output.length && this.headerBuffer.length < HEADER_BYTES) {
					const next = output.shift()
					const combined = new Uint8Array(this.headerBuffer.length + next.length)
					combined.set(this.headerBuffer)
					combined.set(next, this.headerBuffer.length)
					this.headerBuffer = combined
				}
				if (this.headerBuffer.length < HEADER_BYTES) {
					if (final) throw new Error('PoRE FLAC encoder did not produce a complete STREAMINFO header')
					return []
				}
				const header = this.headerBuffer.slice(0, HEADER_BYTES)
				const remainder = this.headerBuffer.slice(HEADER_BYTES)
				this.headerBuffer = header
				this.headerReady = true
				if (remainder.length) output.unshift(remainder)
			}

			if (final) {
				if (!this.streamInfo) throw new Error('PoRE FLAC encoder did not provide final STREAMINFO')
				this._patchStreamInfo()
				return [this.headerBuffer, ...output]
			}
			return output
		}

		_patchStreamInfo() {
			const info = this.streamInfo
			const data = this.headerBuffer
			const view = new DataView(data.buffer, data.byteOffset + 8)
			view.setUint16(0, info.min_blocksize)
			view.setUint16(2, info.max_blocksize)
			view.setUint16(4, info.min_framesize >>> 8)
			view.setUint8(6, info.min_framesize & 0xff)
			view.setUint16(7, info.max_framesize >>> 8)
			view.setUint8(9, info.max_framesize & 0xff)
			view.setUint8(13, (view.getUint8(13) & 0xf0) | (Math.floor(this.totalSamples / 0x100000000) & 0x0f))
			view.setUint32(14, this.totalSamples >>> 0)
			for (let i = 0; i < 16; i += 1) view.setUint8(18 + i, parseInt(info.md5sum.slice(i * 2, i * 2 + 2), 16) || 0)
		}

		_destroy() {
			if (!this.encoder) return
			try { this.Flac.FLAC__stream_encoder_delete(this.encoder) } catch (_) {}
			this.encoder = null
		}
	}

	window.PoREBrowserFlacEncoder = PoREBrowserFlacEncoder
})()
