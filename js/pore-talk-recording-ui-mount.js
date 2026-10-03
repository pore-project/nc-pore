/*
 * NC-PoRe Talk recording UI mount adapter.
 *
 * Talk does not currently expose a public extension point for active-call
 * media controls. Keep the Talk-specific DOM dependency isolated here.
 */
(function () {
	'use strict'

	// The visible in-call controls live in BottomBar -> TopBarMediaControls.
	// LocalAudioControlButton renders this wrapper around the microphone and
	// its device selector. Insert PoRE immediately after that group so it sits
	// between microphone and camera without modifying Talk itself.
	const MOUNT_SELECTOR = '.bottom-bar .buttons-bar .local-audio-control-wrapper'
	const ROOT_ATTRIBUTE = 'data-pore-talk-recording-ui'

	let observer = null
	let mountedHost = null
	let mountedRoot = null
	let participantStatusObserver = null
	let latestContext = null
	let talkCallPeers = []
	let talkCallPeersToken = null
	let talkCallPeersFetchedAt = 0
	let talkCallPeersFetch = null

	function getMountHost() {
		return document.querySelector(MOUNT_SELECTOR)
	}

	function unmount() {
		if (mountedRoot?.parentNode) mountedRoot.remove()
		mountedRoot = null
		mountedHost = null
	}

	function mount() {
		if (!window.PoRETalkRecordingUi?.mount) return

		const host = getMountHost()
		if (!host) {
			unmount()
			return
		}

		if (mountedHost === host && mountedRoot?.isConnected) return

		unmount()

		const root = document.createElement('div')
		root.setAttribute(ROOT_ATTRIBUTE, 'true')
		root.className = 'pore-talk-recording-ui-mount'
		host.insertAdjacentElement('afterend', root)

		try {
			window.PoRETalkRecordingUi.mount({ mountElement: root })
			mountedHost = host
			mountedRoot = root
			window.dispatchEvent(new CustomEvent('pore:recording-ui-mount', { detail: { mountElement: root } }))
		} catch (error) {
			root.remove()
			console.error('[NC-PoRe] Failed to mount Talk recording UI', error)
		}
	}

	function currentActorId() {
		return window.__poreTalkRecordingCoordinator?.talk_actorId || latestContext?.actorId || null
	}

	function currentConversationToken() {
		return latestContext?.productionId
			|| window.__poreTalkRecordingCoordinator?.sessionId
			|| window.location.pathname.match(/(?:\/apps\/spreed)?\/(?:call|room)\/([^/]+)/)?.[1]
			|| null
	}

	async function refreshTalkCallPeers(force = false) {
		const token = currentConversationToken()
		if (!token) return
		const now = Date.now()
		if (!force && talkCallPeersToken === token && now - talkCallPeersFetchedAt < 5000) return
		if (talkCallPeersFetch) return talkCallPeersFetch

		talkCallPeersFetch = (async () => {
			try {
				const response = await fetch('/ocs/v2.php/apps/spreed/api/v4/call/' + encodeURIComponent(token), {
					headers: {
					'OCS-APIRequest': 'true',
					'Accept': 'application/json',
				},
				})
				if (!response.ok) throw new Error('Talk call participant request failed: ' + response.status)
				const payload = await response.json()
				const peers = payload?.ocs?.data
				if (!Array.isArray(peers)) throw new Error('Talk call participant response is invalid')
				talkCallPeers = peers
				talkCallPeersToken = token
				talkCallPeersFetchedAt = Date.now()
			} catch (error) {
				talkCallPeersToken = token
				talkCallPeersFetchedAt = Date.now()
				console.debug('[NC-PoRe] Talk call participant mapping unavailable', error)
			} finally {
				talkCallPeersFetch = null
				renderParticipantStatuses()
			}
		})()
		return talkCallPeersFetch
	}

	function ensureRelativePosition(element) {
		if (getComputedStyle(element).position === 'static') element.style.position = 'relative'
	}

	function getTileForSession(sessionId) {
		return Array.from(document.querySelectorAll('[data-tile-session-id]'))
			.find(element => element.getAttribute('data-tile-session-id') === sessionId) || null
	}

	function mountStatusOnTile(tile, context) {
		const Status = window.PoRETalkParticipantStatus
		if (!Status || !tile) return null
		ensureRelativePosition(tile)

		let indicator = tile.querySelector(':scope > .pore-talk-participant-status')
		if (!indicator) {
			indicator = Status.create(context)
			tile.appendChild(indicator)
		} else {
			Status.updateExisting(indicator, context)
		}
		return indicator
	}

	function renderParticipantStatuses() {
		const Status = window.PoRETalkParticipantStatus
		if (!Status || !latestContext) return

		const role = latestContext.role || 'none'
		const recordingState = latestContext
		const recordingParticipants = Array.isArray(recordingState.participants) ? recordingState.participants : []
		const actorId = currentActorId()
		const desired = new Set()

		if (role === 'listener' || latestContext.listener === true) {
			document.querySelectorAll('[data-pore-talk-participant-status]').forEach(Status.destroy)
			return
		}

		const getRecordingParticipant = id => recordingParticipants.find(participant => participant?.id === id) || null
		const renderOne = (talkParticipant, recordingParticipant, renderRemoteTile = true) => {
			if (!talkParticipant?.actorId) return
			const statusContext = {
				state: recordingState.state || 'preparing',
				listener: recordingParticipant === null,
				ready: recordingParticipant?.ready === true,
				confirmed: recordingState.confirmed === true && recordingParticipant !== null,
				productionStatus: recordingState.productionStatus || null,
				errorMessage: recordingState.errorMessage || '',
			}

			if (talkParticipant.actorId === actorId) {
				const localTile = document.querySelector('.localVideoContainer')
				if (localTile) {
					const indicator = mountStatusOnTile(localTile, statusContext)
					if (indicator) desired.add(indicator)
				}
			}

			if (renderRemoteTile && talkParticipant.sessionId) {
				const tile = getTileForSession(talkParticipant.sessionId)
				if (tile) {
					const indicator = mountStatusOnTile(tile, statusContext)
					if (indicator) desired.add(indicator)
				}
			}
		}

		if (role === 'host') {
			for (const talkParticipant of talkCallPeers) {
				renderOne(talkParticipant, getRecordingParticipant(talkParticipant.actorId))
			}
		} else if (role === 'participant' && actorId) {
			const talkParticipant = talkCallPeers.find(participant => participant.actorId === actorId)
			const recordingParticipant = getRecordingParticipant(actorId)
			if (talkParticipant && recordingParticipant) renderOne(talkParticipant, recordingParticipant, false)
		}

		document.querySelectorAll('[data-pore-talk-participant-status]').forEach(indicator => {
			if (!desired.has(indicator)) Status.destroy(indicator)
		})

		if (role === 'host') void refreshTalkCallPeers()
	}


	function start() {
		if (observer) return
		mount()
		participantStatusObserver = new MutationObserver(mutations => {
			const poreSelector = '[data-pore-talk-participant-status], #' + 'pore-talk-participant-status-popover'
			const relevantMutation = mutations.some(mutation => {
				const nodes = [...mutation.addedNodes, ...mutation.removedNodes]
				return nodes.some(node => node.nodeType === Node.ELEMENT_NODE && !node.matches(poreSelector))
			})
			if (relevantMutation) renderParticipantStatuses()
		})
		participantStatusObserver.observe(document.body, { childList: true, subtree: true })
		observer = new MutationObserver(mount)
		observer.observe(document.body, { childList: true, subtree: true })
	}

	function stop() {
		if (observer) {
			observer.disconnect()
			observer = null
		}
		if (participantStatusObserver) {
			participantStatusObserver.disconnect()
			participantStatusObserver = null
		}
		window.PoRETalkParticipantStatus?.hidePopover()
		document.querySelectorAll('[data-pore-talk-participant-status]').forEach(element => element.remove())
		latestContext = null
		talkCallPeers = []
		talkCallPeersToken = null
		talkCallPeersFetchedAt = 0
		talkCallPeersFetch = null
		unmount()
	}

	window.PoRETalkRecordingUiMount = {
		start,
		stop,
		mount,
		unmount,
		getMountElement: () => mountedRoot,
	}

	window.addEventListener('pore:recording-ui-context', event => {
		latestContext = { ...(latestContext || {}), ...(event.detail || {}) }
		renderParticipantStatuses()
	})

	window.addEventListener('pore:recording-state', event => {
		latestContext = { ...(latestContext || {}), ...(event.detail || {}) }
		renderParticipantStatuses()
	})

	window.addEventListener('pore:talk-production-identity', event => {
		latestContext = {
			...(latestContext || {}),
			productionId: event.detail?.conversationId || latestContext?.productionId || null,
			productionLabel: event.detail?.productionLabel || latestContext?.productionLabel || null,
		}
		renderParticipantStatuses()
	})

	if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', start, { once: true })
	else start()
})()
