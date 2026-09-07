/*
 * NC-PoRE settings embedded into the Nextcloud Talk settings dialog.
 * Talk exposes this extension point as OCA.Talk.Settings.
 */

(() => {
	'use strict'

	const APP_ID = 'pore'
	const SETTINGS_URL = '/ocs/v2.php/apps/pore/v1/settings'
	const DEFAULT_ROOT = 'audio'
	const SECTION_ID = 'pore'
	const ELEMENT_NAME = 'pore-talk-settings'

	const request = async (method, storageRoot = null) => {
		const options = {
			method,
			credentials: 'same-origin',
			headers: {
				Accept: 'application/json',
				'OCS-APIRequest': 'true',
			},
		}
		if (method === 'PUT') {
			const form = new URLSearchParams()
			form.set('storageRoot', storageRoot ?? '')
			options.body = form
		}

		const response = await fetch(SETTINGS_URL, options)
		const payload = await response.json()
		if (!response.ok || payload?.ocs?.meta?.status !== 'ok') {
			throw new Error(payload?.ocs?.meta?.message || 'Unable to update NC-PoRe settings')
		}
		return payload.ocs.data
	}

	class PoReTalkSettings extends HTMLElement {
		connectedCallback() {
			this.render()
			void this.load()
		}

		render() {
			this.innerHTML = `
				<div class="pore-talk-settings">
					<label for="pore-storage-root">Speicherort für Aufnahmen</label>
					<input id="pore-storage-root" type="text" autocomplete="off" placeholder="${DEFAULT_ROOT}">
					<p class="pore-talk-settings__description">Relativer Pfad innerhalb deiner Nextcloud-Dateien. Leer bedeutet audio.</p>
					<p class="pore-talk-settings__status" aria-live="polite"></p>
				</div>
			`
			this.input = this.querySelector('#pore-storage-root')
			this.status = this.querySelector('.pore-talk-settings__status')
			this.input.addEventListener('blur', () => void this.save())
			this.input.addEventListener('keydown', event => {
				if (event.key === 'Enter') {
					event.preventDefault()
					this.input.blur()
				}
			})
		}

		async load() {
			try {
				const settings = await request('GET')
				this.input.value = settings.storage_root || ''
				this.input.placeholder = settings.default_storage_root || DEFAULT_ROOT
			} catch (error) {
				this.setStatus(error.message, true)
			}
		}

		async save() {
			const value = this.input.value.trim()
			this.setStatus('')
			this.input.disabled = true
			try {
				const settings = await request('PUT', value)
				this.input.value = settings.storage_root || ''
				this.setStatus('Gespeichert')
			} catch (error) {
				this.setStatus(error.message, true)
			} finally {
				this.input.disabled = false
			}
		}

		setStatus(message, error = false) {
			this.status.textContent = message
			this.status.classList.toggle('pore-talk-settings__status--error', error)
		}
	}

	if (!customElements.get(ELEMENT_NAME)) customElements.define(ELEMENT_NAME, PoReTalkSettings)

	const register = () => {
		const settings = window.OCA?.Talk?.Settings
		if (!settings?.registerSection) return false
		settings.unregisterSection?.(SECTION_ID)
		settings.registerSection({
			id: SECTION_ID,
			name: 'NC-PoRE',
			element: ELEMENT_NAME,
		})
		return true
	}

	const tryRegister = () => {
		if (register()) return
		window.setTimeout(tryRegister, 100)
	}

	tryRegister()
})()
