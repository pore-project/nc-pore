/* NC-PoRe — browser PCM preservation capture. */
(() => {
	'use strict'

	const OPENING_TEST_TONE_HZ = 1000
	const OPENING_TEST_TONE_MS = 100
	const OPENING_TEST_TONE_AMPLITUDE = 0.2
	const PERSISTENCE_RETRY_INITIAL_MS = 250
	const PERSISTENCE_RETRY_MAX_MS = 5000
	const PERSISTENCE_RETRY_BUFFER_CHUNKS = 8
	const PERSISTENCE_STOP_RETRY_ATTEMPTS = 8

	function resolveWorkletUrl() {
		const recorderScript = [...document.scripts]
			.find(script => /\/pore-browser-pcm-recorder\.js(?:\?|$)/.test(script.src))
		if (recorderScript?.src) return new URL('pore-browser-pcm-worklet.js', recorderScript.src).href
		return 'pore-browser-pcm-worklet.js'
	}

	class PoREBrowserPcmRecorder {
		constructor({ AudioContextClass = window.AudioContext || window.webkitAudioContext, workletUrl = resolveWorkletUrl(), persistenceStoreFactory = () => new window.PoREBrowserPcmPersistenceStore(), persistenceChunkBytes = 128 * 1024, maxPersistenceQueueBytes = persistenceChunkBytes * PERSISTENCE_RETRY_BUFFER_CHUNKS, onPersistenceSafetyStop = null } = {}) {
			this.AudioContextClass = AudioContextClass; this.workletUrl = workletUrl; this.persistenceStoreFactory = persistenceStoreFactory; this.persistenceChunkBytes = persistenceChunkBytes; this.maxPersistenceQueueBytes = maxPersistenceQueueBytes; this.onPersistenceSafetyStop = onPersistenceSafetyStop
			this.state = 'idle'; this.context = null; this.source = null; this.worklet = null; this.stream = null; this.sampleRate = null; this.channels = 1; this.startedAt = null; this.stoppedAt = null; this.sequence = 0
			this.captureId = null; this.recordingSessionId = null; this.productionId = null; this.productionLabel = null; this.participantLabel = null; this.recordingId = null; this.persistenceStore = null; this.pendingParts = []; this.pendingBytes = 0; this.persistedChunkIndex = 0; this.persistenceQueue = []; this.persistenceQueueBytes = 0; this.persistenceDrainPromise = null; this.persistenceRetryTimer = null; this.persistenceRetryDelayMs = PERSISTENCE_RETRY_INITIAL_MS; this.persistenceState = 'idle'; this.persistenceSafetyStopEmitted = false; this.openingSignet = null; this.capturedSamples = 0; this.openingTestTonePending = false; this.openingTestToneSamplesWritten = 0
		}
		getState() { return this.state }
		isRecording() { return this.state === 'recording' }

		async primeAudioContext() {
			if (!this.AudioContextClass) throw new Error('Web Audio is not available in this browser')
			if (!this.context || this.context.state === 'closed') this.context = new this.AudioContextClass()
			if (this.context.state === 'suspended') await this.context.resume()
			return this.context
		}

		async start(track, metadata = {}) {
			if (this.isRecording()) throw new Error('PoRE PCM recording is already active')
			if (!track || track.kind !== 'audio') throw new Error('PoRE requires an owned audio MediaStreamTrack')
			if (track.readyState !== 'live') throw new Error('PoRE cannot start from an ended audio track')
			if (!this.AudioContextClass) throw new Error('Web Audio is not available in this browser')
			if (!window.PoREBrowserPcmPersistenceStore && !metadata.persistenceStore) throw new Error('PoRE durable browser preservation is not available')

			this.startedAt = new Date().toISOString(); this.stoppedAt = null; this.sequence += 1; this.stream = new MediaStream([track]); this.captureId = metadata.captureId || technicalId('browser-capture'); this.recordingSessionId = metadata.recordingSessionId || technicalId('browser-session'); this.productionId = metadata.productionId || null; this.productionLabel = metadata.productionLabel || this.productionId; this.participantLabel = metadata.participantLabel || null; this.recordingId = metadata.recordingId || null; this.pendingParts = []; this.pendingBytes = 0; this.persistedChunkIndex = 0; this.persistenceQueue = []; this.persistenceQueueBytes = 0; this.persistenceDrainPromise = null; this._clearPersistenceRetry(); this.persistenceRetryDelayMs = PERSISTENCE_RETRY_INITIAL_MS; this.persistenceState = 'healthy'; this.persistenceSafetyStopEmitted = false; this.openingSignet = null; this.capturedSamples = 0; this.openingTestTonePending = false; this.openingTestToneSamplesWritten = 0; this.persistenceStore = metadata.persistenceStore || this.persistenceStoreFactory()
			try {
				await this.primeAudioContext(); await this.context.audioWorklet.addModule(this.workletUrl); this.sampleRate = this.context.sampleRate
				await this.persistenceStore.beginCapture({ captureId: this.captureId, recordingSessionId: this.recordingSessionId, productionId: this.productionId, productionLabel: this.productionLabel, participantLabel: this.participantLabel, recordingId: this.recordingId, sequence: this.sequence, startedAt: this.startedAt, sampleRate: this.sampleRate, channels: this.channels, encoding: 'pcm_s24le', format: 'audio/wav' })
				this.source = this.context.createMediaStreamSource(this.stream); this.worklet = new AudioWorkletNode(this.context, 'pore-pcm-processor', { numberOfInputs: 1, numberOfOutputs: 1, channelCount: 1, channelCountMode: 'explicit', channelInterpretation: 'speakers' })
				this.worklet.port.onmessage = event => { if (this.state === 'recording' && event.data instanceof Float32Array) this._acceptSamples(event.data) }
				this.source.connect(this.worklet); const sink = this.context.createGain(); sink.gain.value = 0; this.worklet.connect(sink); sink.connect(this.context.destination); if (this.context.state === 'suspended') await this.context.resume(); this.state = 'recording'
				window.dispatchEvent(new CustomEvent('pore:recording-started', { detail: { sequence: this.sequence, startedAt: this.startedAt, sampleRate: this.sampleRate, channels: this.channels, format: 'pcm_s24le', trackId: track.id, trackLabel: track.label, captureId: this.captureId, recordingSessionId: this.recordingSessionId, participantLabel: this.participantLabel } }))
			} catch (error) { this.state = 'error'; await this._cleanup(); throw error }
		}

		markOpeningSignet(at = new Date().toISOString()) {
			if (!this.isRecording()) throw new Error('PoRE opening signet requires an active local capture')
			if (this.openingSignet) return this.openingSignet
			this.openingTestTonePending = true
			this.openingTestToneSamplesWritten = 0
			this.openingSignet = { kind: 'opening-test-tone', occurredAt: at, elapsedMs: null, sampleOffset: null, toneHz: OPENING_TEST_TONE_HZ, toneDurationMs: OPENING_TEST_TONE_MS }
			window.dispatchEvent(new CustomEvent('pore:recording-opening-signet', { detail: { sequence: this.sequence, captureId: this.captureId, recordingSessionId: this.recordingSessionId, ...this.openingSignet } }))
			return this.openingSignet
		}

		_acceptSamples(samples) {
			if (this.openingTestTonePending) {
				const sampleOffset = this.capturedSamples
				const toneSamples = Math.round(this.sampleRate * OPENING_TEST_TONE_MS / 1000)
				const remaining = toneSamples - this.openingTestToneSamplesWritten
				const tone = Math.min(remaining, samples.length)
				for (let i = 0; i < tone; i += 1) {
					const sampleIndex = this.openingTestToneSamplesWritten + i
					samples[i] = Math.max(-1, Math.min(1, samples[i] + OPENING_TEST_TONE_AMPLITUDE * Math.sin(2 * Math.PI * OPENING_TEST_TONE_HZ * (sampleIndex / this.sampleRate))))
				}
				this.openingTestToneSamplesWritten += tone
				this.openingTestTonePending = this.openingTestToneSamplesWritten < toneSamples
				if (this.openingSignet && this.openingSignet.sampleOffset === null) { this.openingSignet.sampleOffset = sampleOffset; this.openingSignet.elapsedMs = sampleOffset * 1000 / this.sampleRate }
			}
			this.capturedSamples += samples.length
			const chunk = float32ToPcm24(samples); this.pendingParts.push(chunk); this.pendingBytes += chunk.length
			if (this.pendingBytes >= this.persistenceChunkBytes) this._queuePersistenceChunk()
		}

		_queuePersistenceChunk() {
			if (!this.pendingBytes) return
			const parts = this.pendingParts; const size = this.pendingBytes; const index = this.persistedChunkIndex; this.pendingParts = []; this.pendingBytes = 0; this.persistedChunkIndex += 1
			const blob = new Blob(parts, { type: 'application/octet-stream' })
			this.persistenceQueue.push({ index, blob, size })
			this.persistenceQueueBytes += size
			if (this.persistenceQueueBytes >= this.maxPersistenceQueueBytes) this._emitPersistenceSafetyStop()
			void this._drainPersistenceQueue()
		}

		async _drainPersistenceQueue() {
			if (this.persistenceDrainPromise) return this.persistenceDrainPromise
			if (!this.persistenceQueue.length) return
			if (this.persistenceRetryTimer) return
			this.persistenceDrainPromise = (async () => {
				while (this.persistenceQueue.length) {
					const entry = this.persistenceQueue[0]
					try {
						await this.persistenceStore.appendChunk(this.captureId, entry.index, entry.blob)
						this.persistenceQueue.shift(); this.persistenceQueueBytes -= entry.size; this.persistenceRetryDelayMs = PERSISTENCE_RETRY_INITIAL_MS
						this._publishPersistenceState(this.persistenceQueue.length ? 'degraded' : 'healthy')
					} catch (error) {
						this._publishPersistenceState('recovering', error)
						if (this.persistenceQueueBytes >= this.maxPersistenceQueueBytes) this._emitPersistenceSafetyStop(error)
						this._schedulePersistenceRetry()
						return
					}
				}
			})()
			try { await this.persistenceDrainPromise } finally { this.persistenceDrainPromise = null }
		}

		_schedulePersistenceRetry() {
			if (this.persistenceRetryTimer) return
			const delay = this.persistenceRetryDelayMs
			this.persistenceRetryDelayMs = Math.min(PERSISTENCE_RETRY_MAX_MS, this.persistenceRetryDelayMs * 2)
			this.persistenceRetryTimer = window.setTimeout(() => { this.persistenceRetryTimer = null; void this._drainPersistenceQueue() }, delay)
		}

		_clearPersistenceRetry() {
			if (this.persistenceRetryTimer) window.clearTimeout(this.persistenceRetryTimer)
			this.persistenceRetryTimer = null
		}

		async _flushPersistenceQueue() {
			this._clearPersistenceRetry()
			for (let attempt = 0; this.persistenceQueue.length && attempt < PERSISTENCE_STOP_RETRY_ATTEMPTS; attempt += 1) {
				await this._drainPersistenceQueue()
				if (!this.persistenceQueue.length) return
				this._clearPersistenceRetry()
				await new Promise(resolve => window.setTimeout(resolve, this.persistenceRetryDelayMs))
			}
			if (this.persistenceQueue.length) throw new Error(`PoRE durable capture persistence did not recover (${this.persistenceQueue.length} chunk(s) pending)`)
		}

		_publishPersistenceState(state, error = null) {
			if (this.persistenceState === state && !error) return
			this.persistenceState = state
			window.dispatchEvent(new CustomEvent('pore:recording-persistence-state', { detail: { captureId: this.captureId, recordingSessionId: this.recordingSessionId, state, pendingChunks: this.persistenceQueue.length, pendingBytes: this.persistenceQueueBytes, error } }))
		}

		_emitPersistenceSafetyStop(error = null) {
			if (this.persistenceSafetyStopEmitted) return
			this.persistenceSafetyStopEmitted = true
			window.dispatchEvent(new CustomEvent('pore:recording-persistence-safety-stop', { detail: { captureId: this.captureId, recordingSessionId: this.recordingSessionId, pendingChunks: this.persistenceQueue.length, pendingBytes: this.persistenceQueueBytes, error } }))
			if (typeof this.onPersistenceSafetyStop === 'function') void this.onPersistenceSafetyStop({ captureId: this.captureId, recordingSessionId: this.recordingSessionId, pendingChunks: this.persistenceQueue.length, pendingBytes: this.persistenceQueueBytes, error })
		}

		async stop(reason = 'host') {
			if (!this.isRecording()) return null; this.state = 'stopping'; this.stoppedAt = new Date().toISOString(); this._clearPersistenceRetry()
			try {
				this.worklet?.disconnect(); this.source?.disconnect(); if (this.context?.state !== 'closed') await this.context?.close(); this._queuePersistenceChunk(); await this._flushPersistenceQueue()
				const stored = await this.persistenceStore.getCapture(this.captureId); if (!stored || !stored.chunks.length) throw new Error('PoRE durable capture contains no persisted audio chunks')
				const pcmSize = stored.chunks.reduce((total, chunk) => total + chunk.size, 0)
				const artifact = { kind: 'audio', format: 'audio/wav', encoding: 'pcm_s24le', size: 44 + pcmSize, sequence: this.sequence, captureId: this.captureId, recordingSessionId: this.recordingSessionId, productionId: this.productionId, recordingId: this.recordingId, startedAt: this.startedAt, stoppedAt: this.stoppedAt, stopReason: reason, sampleRate: this.sampleRate, channels: this.channels, participantLabel: this.participantLabel, openingSignet: this.openingSignet, source: { productionId: this.productionId, productionLabel: this.productionLabel, recordingId: this.recordingId, captureId: this.captureId, recordingSessionId: this.recordingSessionId, participantLabel: this.participantLabel, trackId: this.stream?.getAudioTracks?.()[0]?.id || null, startedAt: this.startedAt, openingSignet: this.openingSignet } }
				await this.persistenceStore.finalizeCapture(this.captureId, { stoppedAt: this.stoppedAt, stopReason: reason, size: artifact.size, chunkCount: stored.chunks.length, openingSignet: this.openingSignet })
				await this._cleanup(); this.state = 'idle'; this.persistenceState = 'idle'; window.dispatchEvent(new CustomEvent('pore:recording-finalized', { detail: artifact })); return artifact
			} catch (error) { this.state = 'error'; await this._cleanup(); window.dispatchEvent(new CustomEvent('pore:recording-error', { detail: { error } })); throw error }
		}

		async _cleanup() { this._clearPersistenceRetry(); if (this.stream) this.stream.getTracks().forEach(track => track.stop()); this.stream = null; this.source = null; this.worklet = null; this.context = null; this.pendingParts = []; this.pendingBytes = 0 }
	}

	function technicalId(prefix) { if (window.crypto?.randomUUID) return `${prefix}-${window.crypto.randomUUID()}`; return `${prefix}-${Date.now()}-${Math.random().toString(16).slice(2)}` }
	function float32ToPcm24(samples) { const output = new Uint8Array(samples.length * 3); for (let i = 0; i < samples.length; i += 1) { const value = Math.max(-1, Math.min(1, samples[i])); const integer = Math.round(value * (value < 0 ? 8388608 : 8388607)); const offset = i * 3; output[offset] = integer & 0xff; output[offset + 1] = (integer >> 8) & 0xff; output[offset + 2] = (integer >> 16) & 0xff } return output }

	window.PoREBrowserPcmRecorder = PoREBrowserPcmRecorder
})()
