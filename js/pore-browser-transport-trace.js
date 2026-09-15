/* NC-PoRe — temporary browser transport lifecycle trace. */
(() => {
	'use strict'

	const stamp = () => new Date().toISOString()
	const trace = (event, detail = {}) => console.info(`[NC-PoRe][TRACE ${stamp()}] ${event}`, detail)

	trace('trace-script-loaded', {
		initRuntimeTransport: Boolean(window.__poreBrowserRuntimeTransport),
		completionJob: Boolean(window.__poreBrowserCompletionJob),
		recordingController: Boolean(window.__poreTalkRecordingController),
		persistenceStore: Boolean(window.__poreBrowserPcmPersistenceStore),
	})

	window.addEventListener('pore:recording-ui-stop-local', event => trace('ui-stop-local', { reason: event.detail?.reason || null }))
	window.addEventListener('pore:recording-local-finalized', event => trace('local-finalized', {
		captureId: event.detail?.captureId || event.detail?.source?.captureId || null,
		recordingId: event.detail?.recordingId || event.detail?.source?.recordingId || null,
		recordingSessionId: event.detail?.recordingSessionId || event.detail?.source?.recordingSessionId || null,
	}))
	window.addEventListener('pore:recording-artifact-persistence-ready', event => trace('persistence-ready', {
		captureId: event.detail?.captureId || null,
		recordingId: event.detail?.recordingId || null,
		recordingSessionId: event.detail?.recordingSessionId || null,
	}))
	window.addEventListener('pore:recording-transport-ready', event => trace('transport-ready', {
		captureId: event.detail?.captureId || null,
		size: event.detail?.size || null,
		sha256: event.detail?.payloadSha256 || null,
		blobSize: event.detail?.blob?.size || null,
	}))
	window.addEventListener('pore:recording-transport-completed', event => trace('transport-completed', {
		artifactId: event.detail?.artifact_id || event.detail?.artifactId || null,
		fileId: event.detail?.file_id || event.detail?.fileId || null,
		sha256: event.detail?.sha256 || null,
	}))
	window.addEventListener('pore:recording-local-error', event => trace('local-error', {
		message: event.detail?.error?.message || String(event.detail?.error || 'unknown'),
		name: event.detail?.error?.name || null,
	}))
})()
