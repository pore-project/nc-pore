/* NC-PoRe — browser PCM preservation capture. */
(() => {
	'use strict'

	class PoREBrowserPcmRecorder {
		constructor({ AudioContextClass = window.AudioContext || window.webkitAudioContext, workletUrl = 'pore-browser-pcm-worklet.js', persistenceStoreFactory = () => new window.PoREBrowserPcmPersistenceStore(), persistenceChunkBytes = 128 * 1024 } = {}) {
			this.AudioContextClass = AudioContextClass; this.workletUrl = workletUrl; this.persistenceStoreFactory = persistenceStoreFactory; this.persistenceChunkBytes = persistenceChunkBytes
			this.state = 'idle'; this.context = null; this.source = null; this.worklet = null; this.stream = null; this.sampleRate = null; this.channels = 1; this.startedAt = null; this.stoppedAt = null; this.sequence = 0
			this.captureId = null; this.recordingSessionId = null; this.productionId = null; this.productionLabel = null; this.recordingId = null; this.persistenceStore = null; this.persistenceChain = Promise.resolve(); this.pendingParts = []; this.pendingBytes = 0; this.persistedChunkIndex = 0
		}
		getState() { return this.state }
		isRecording() { return this.state === 'recording' }

		async start(track, metadata = {}) {
			if (this.isRecording()) throw new Error('PoRE PCM recording is already active')
			if (!track || track.kind !== 'audio') throw new Error('PoRE requires an owned audio MediaStreamTrack')
			if (track.readyState !== 'live') throw new Error('PoRE cannot start from an ended audio track')
			if (!this.AudioContextClass) throw new Error('Web Audio is not available in this browser')
			if (!window.PoREBrowserPcmPersistenceStore && !metadata.persistenceStore) throw new Error('PoRE durable browser preservation is not available')

			this.startedAt = new Date().toISOString(); this.stoppedAt = null; this.sequence += 1; this.stream = new MediaStream([track]); this.captureId = metadata.captureId || technicalId('browser-capture'); this.recordingSessionId = metadata.recordingSessionId || technicalId('browser-session'); this.productionId = metadata.productionId || null; this.productionLabel = metadata.productionLabel || this.productionId; this.recordingId = metadata.recordingId || null; this.pendingParts = []; this.pendingBytes = 0; this.persistedChunkIndex = 0; this.persistenceChain = Promise.resolve(); this.persistenceStore = metadata.persistenceStore || this.persistenceStoreFactory()
			try {
				this.context = new this.AudioContextClass(); await this.context.audioWorklet.addModule(this.workletUrl); this.sampleRate = this.context.sampleRate
				await this.persistenceStore.beginCapture({ captureId: this.captureId, recordingSessionId: this.recordingSessionId, productionId: this.productionId, productionLabel: this.productionLabel, recordingId: this.recordingId, sequence: this.sequence, startedAt: this.startedAt, sampleRate: this.sampleRate, channels: this.channels, encoding: 'pcm_s24le', format: 'audio/wav' })
				this.source = this.context.createMediaStreamSource(this.stream); this.worklet = new AudioWorkletNode(this.context, 'pore-pcm-processor', { numberOfInputs: 1, numberOfOutputs: 1, channelCount: 1, channelCountMode: 'explicit', channelInterpretation: 'speakers' })
				this.worklet.port.onmessage = event => { if (this.state === 'recording' && event.data instanceof Float32Array) this._acceptSamples(event.data) }
				this.source.connect(this.worklet); const sink = this.context.createGain(); sink.gain.value = 0; this.worklet.connect(sink); sink.connect(this.context.destination); if (this.context.state === 'suspended') await this.context.resume(); this.state = 'recording'
				window.dispatchEvent(new CustomEvent('pore:recording-started', { detail: { sequence: this.sequence, startedAt: this.startedAt, sampleRate: this.sampleRate, channels: this.channels, format: 'pcm_s24le', trackId: track.id, trackLabel: track.label, captureId: this.captureId, recordingSessionId: this.recordingSessionId } }))
			} catch (error) { this.state = 'error'; await this._cleanup(); throw error }
		}

		_acceptSamples(samples) {
			const chunk = float32ToPcm24(samples); this.pendingParts.push(chunk); this.pendingBytes += chunk.length
			if (this.pendingBytes >= this.persistenceChunkBytes) this._queuePersistenceChunk()
		}

		_queuePersistenceChunk() {
			if (!this.pendingBytes) return
			const parts = this.pendingParts; const size = this.pendingBytes; const index = this.persistedChunkIndex; this.pendingParts = []; this.pendingBytes = 0; this.persistedChunkIndex += 1
			this.persistenceChain = this.persistenceChain.then(() => this.persistenceStore.appendChunk(this.captureId, index, new Blob(parts, { type: 'application/octet-stream' })))
			this.persistenceChain = this.persistenceChain.catch(error => { this.state = 'error'; window.dispatchEvent(new CustomEvent('pore:recording-error', { detail: { error } })); throw error })
			void size
		}

		async stop(reason = 'host') {
			if (!this.isRecording()) return null; this.state = 'stopping'; this.stoppedAt = new Date().toISOString()
			try {
				this.worklet?.disconnect(); this.source?.disconnect(); if (this.context?.state !== 'closed') await this.context?.close(); this._queuePersistenceChunk(); await this.persistenceChain
				const stored = await this.persistenceStore.getCapture(this.captureId); if (!stored || !stored.chunks.length) throw new Error('PoRE durable capture contains no persisted audio chunks')
				const pcm = new Blob(stored.chunks, { type: 'application/octet-stream' }); const blob = new Blob([createWavHeader(pcm.size, this.sampleRate, this.channels), pcm], { type: 'audio/wav' })
				const artifact = { kind: 'audio', format: 'audio/wav', encoding: 'pcm_s24le', size: blob.size, sequence: this.sequence, captureId: this.captureId, recordingSessionId: this.recordingSessionId, productionId: this.productionId, recordingId: this.recordingId, startedAt: this.startedAt, stoppedAt: this.stoppedAt, stopReason: reason, sampleRate: this.sampleRate, channels: this.channels, blob, source: { productionId: this.productionId, productionLabel: this.productionLabel, recordingId: this.recordingId, captureId: this.captureId, recordingSessionId: this.recordingSessionId, trackId: this.stream?.getAudioTracks?.()[0]?.id || null, startedAt: this.startedAt } }
				await this.persistenceStore.finalizeCapture(this.captureId, { stoppedAt: this.stoppedAt, stopReason: reason, size: blob.size, chunkCount: stored.chunks.length })
				await this._cleanup(); this.state = 'idle'; window.dispatchEvent(new CustomEvent('pore:recording-finalized', { detail: artifact })); return artifact
			} catch (error) { this.state = 'error'; await this._cleanup(); window.dispatchEvent(new CustomEvent('pore:recording-error', { detail: { error } })); throw error }
		}

		async _cleanup() { if (this.stream) this.stream.getTracks().forEach(track => track.stop()); this.stream = null; this.source = null; this.worklet = null; this.context = null; this.pendingParts = []; this.pendingBytes = 0 }
	}

	function technicalId(prefix) { if (window.crypto?.randomUUID) return `${prefix}-${window.crypto.randomUUID()}`; return `${prefix}-${Date.now()}-${Math.random().toString(16).slice(2)}` }
	function float32ToPcm24(samples) { const output = new Uint8Array(samples.length * 3); for (let i = 0; i < samples.length; i += 1) { const value = Math.max(-1, Math.min(1, samples[i])); const integer = Math.round(value * (value < 0 ? 8388608 : 8388607)); const offset = i * 3; output[offset] = integer & 0xff; output[offset + 1] = (integer >> 8) & 0xff; output[offset + 2] = (integer >> 16) & 0xff } return output }
	function createWavHeader(dataLength, sampleRate, channels) { const header = new ArrayBuffer(44); const view = new DataView(header); const write = (offset, text) => [...text].forEach((char, index) => view.setUint8(offset + index, char.charCodeAt(0))); write(0, 'RIFF'); view.setUint32(4, 36 + dataLength, true); write(8, 'WAVE'); write(12, 'fmt '); view.setUint32(16, 16, true); view.setUint16(20, 1, true); view.setUint16(22, channels, true); view.setUint32(24, sampleRate, true); view.setUint32(28, sampleRate * channels * 3, true); view.setUint16(32, channels * 3, true); view.setUint16(34, 24, true); write(36, 'data'); view.setUint32(40, dataLength, true); return header }

	window.PoREBrowserPcmRecorder = PoREBrowserPcmRecorder
})()
