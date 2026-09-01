/* NC-PoRe — browser PCM recording boundary. */
(() => {
	'use strict'
	class PoREBrowserPcmRecorder {
		constructor({ AudioContextClass = window.AudioContext || window.webkitAudioContext } = {}) {
			this.AudioContextClass = AudioContextClass
			this.state = 'idle'
			this.context = null
			this.source = null
			this.processor = null
			this.stream = null
			this.parts = []
			this.sampleRate = null
			this.channels = 1
			this.startedAt = null
			this.stoppedAt = null
			this.sequence = 0
		}
		getState() { return this.state }
		isRecording() { return this.state === 'recording' }
		async start(track) {
			if (this.isRecording()) throw new Error('PoRE PCM recording is already active')
			if (!track || track.kind !== 'audio') throw new Error('PoRE requires an owned audio MediaStreamTrack')
			if (track.readyState !== 'live') throw new Error('PoRE cannot start from an ended audio track')
			if (!this.AudioContextClass) throw new Error('Web Audio is not available in this browser')
			this.parts = []
			this.startedAt = new Date().toISOString()
			this.stoppedAt = null
			this.sequence += 1
			this.stream = new MediaStream([track])
			try {
				this.context = new this.AudioContextClass()
				this.sampleRate = this.context.sampleRate
				this.source = this.context.createMediaStreamSource(this.stream)
				this.processor = this.context.createScriptProcessor(4096, 1, 1)
				this.processor.onaudioprocess = event => this.parts.push(float32ToPcm24(event.inputBuffer.getChannelData(0)))
				this.source.connect(this.processor)
				this.processor.connect(this.context.destination)
				if (this.context.state === 'suspended') await this.context.resume()
				this.state = 'recording'
				window.dispatchEvent(new CustomEvent('pore:recording-started', { detail: {
					sequence: this.sequence, startedAt: this.startedAt, sampleRate: this.sampleRate,
					channels: this.channels, format: 'pcm_s24le', trackId: track.id, trackLabel: track.label,
				} }))
			} catch (error) {
				this.state = 'error'
				await this._cleanup()
				throw error
			}
		}
		async stop(reason = 'host') {
			if (!this.isRecording()) return null
			this.state = 'stopping'
			this.stoppedAt = new Date().toISOString()
			try {
				this.processor?.disconnect()
				this.source?.disconnect()
				if (this.context?.state !== 'closed') await this.context?.close()
				const pcm = concatUint8Arrays(this.parts)
				const blob = new Blob([createWavHeader(pcm.length, this.sampleRate, this.channels), pcm], { type: 'audio/wav' })
				const artifact = { kind: 'audio', format: 'audio/wav', encoding: 'pcm_s24le', size: blob.size,
					sequence: this.sequence, startedAt: this.startedAt, stoppedAt: this.stoppedAt, stopReason: reason,
					sampleRate: this.sampleRate, channels: this.channels, blob }
				await this._cleanup()
				this.state = 'idle'
				window.dispatchEvent(new CustomEvent('pore:recording-finalized', { detail: artifact }))
				return artifact
			} catch (error) {
				this.state = 'error'
				await this._cleanup()
				window.dispatchEvent(new CustomEvent('pore:recording-error', { detail: { error } }))
				throw error
			}
		}
		async _cleanup() {
			this.processor = null
			this.source = null
			if (this.stream) this.stream.getTracks().forEach(track => track.stop())
			this.stream = null
			this.context = null
			this.parts = []
		}
	}
	function float32ToPcm24(samples) {
		const output = new Uint8Array(samples.length * 3)
		for (let i = 0; i < samples.length; i += 1) {
			const value = Math.max(-1, Math.min(1, samples[i]))
			const integer = Math.round(value * (value < 0 ? 8388608 : 8388607))
			const offset = i * 3
			output[offset] = integer & 0xff
			output[offset + 1] = (integer >> 8) & 0xff
			output[offset + 2] = (integer >> 16) & 0xff
		}
		return output
	}
	function concatUint8Arrays(parts) {
		const length = parts.reduce((sum, part) => sum + part.length, 0)
		const output = new Uint8Array(length)
		let offset = 0
		for (const part of parts) { output.set(part, offset); offset += part.length }
		return output
	}
	function createWavHeader(dataLength, sampleRate, channels) {
		const header = new ArrayBuffer(44)
		const view = new DataView(header)
		const write = (offset, text) => [...text].forEach((char, index) => view.setUint8(offset + index, char.charCodeAt(0)))
		write(0, 'RIFF'); view.setUint32(4, 36 + dataLength, true); write(8, 'WAVE'); write(12, 'fmt ')
		view.setUint32(16, 16, true); view.setUint16(20, 1, true); view.setUint16(22, channels, true)
		view.setUint32(24, sampleRate, true); view.setUint32(28, sampleRate * channels * 3, true)
		view.setUint16(32, channels * 3, true); view.setUint16(34, 24, true); write(36, 'data'); view.setUint32(40, dataLength, true)
		return header
	}
	window.PoREBrowserPcmRecorder = PoREBrowserPcmRecorder
})()
