class PoREPcmProcessor extends AudioWorkletProcessor {
	constructor() {
		super()
		this.flushRequested = false
		this.port.onmessage = event => {
			if (event.data?.type === 'flush') this.flushRequested = true
		}
	}

	process(inputs) {
		const input = inputs[0]?.[0]
		if (input?.length) this.port.postMessage(input.slice())
		if (this.flushRequested) {
			this.port.postMessage({ type: 'flush-complete' })
			this.flushRequested = false
		}
		return true
	}
}
registerProcessor('pore-pcm-processor', PoREPcmProcessor)
