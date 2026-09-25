import { strict as assert } from 'node:assert'

const tests = []
const beforeEachHooks = []

globalThis.window = globalThis
globalThis.navigator = { mediaDevices: {} }
globalThis.window.OCA = undefined
globalThis.window.OC = undefined

class Element {
	constructor(tagName = 'div') {
		this.tagName = tagName.toUpperCase()
		this.children = []
		this.parentNode = null
		this.attributes = new Map()
		this.textContent = ''
		this.className = ''
		this.dataset = {}
		this._innerHTML = ''
		this._textContent = ''
	}
	get textContent() { return this._textContent + this.children.map(child => child.textContent || '').join('') }
	set textContent(value) { this._textContent = String(value) }
	set innerHTML(value) {
		this._innerHTML = String(value)
		this.children = []
		if (this._innerHTML.includes('pore-talk-storage-root')) {
			const label = new Element('label')
			label.setAttribute('for', 'pore-talk-storage-root')
			this.appendChild(label)
			const input = new Element('input')
			input.setAttribute('id', 'pore-talk-storage-root')
			this.appendChild(input)
			const status = new Element('p')
			status.setAttribute('class', 'pore-talk-recording__settings-status')
			this.appendChild(status)
		}
	}
	get innerHTML() { return this._innerHTML }
	appendChild(child) { this.children.push(child); child.parentNode = this; return child }
	append(...children) { children.forEach(child => this.appendChild(child)); }
	removeChild(child) { const i = this.children.indexOf(child); if (i >= 0) this.children.splice(i, 1); child.parentNode = null; return child }
	setAttribute(name, value) { this.attributes.set(name, String(value)); if (name === 'class') this.className = String(value); this[name] = String(value) }
	getAttribute(name) { return this.attributes.get(name) ?? null }
	addEventListener() {}
	removeEventListener() {}
	querySelector(selector) { return this._walk().find(node => matches(node, selector)) || null }
	querySelectorAll(selector) { return this._walk().filter(node => matches(node, selector)) }
	closest(selector) {
		let node = this
		while (node) { if (matches(node, selector)) return node; node = node.parentNode }
		return null
	}
	_walk() { return [this, ...this.children.flatMap(child => child._walk ? child._walk() : [])] }
}
function matches(node, selector) {
	if (!(node instanceof Element)) return false
	const aria = selector.match(/^\[aria-label="([^"]+)"\]$/)
	if (aria) return node.getAttribute('aria-label') === aria[1]
	if (selector.startsWith('.')) return node.className.split(/\s+/).includes(selector.slice(1))
	return node.tagName.toLowerCase() === selector.toLowerCase()
}

const document = new Element('document')
document.head = new Element('head')
document.body = new Element('body')
document.appendChild(document.head)
document.appendChild(document.body)
document.createElement = tag => new Element(tag)
document.createElementNS = (_ns, tag) => new Element(tag)
document.getElementById = id => document._walk().find(node => node.id === id) || null
document.scripts = []
globalThis.document = document
globalThis.Element = Element
globalThis.requestAnimationFrame = cb => setTimeout(cb, 0)
globalThis.cancelAnimationFrame = id => clearTimeout(id)

const eventListeners = new Map()
window.addEventListener = (name, fn) => {
	if (!eventListeners.has(name)) eventListeners.set(name, new Set())
	eventListeners.get(name).add(fn)
}
window.removeEventListener = (name, fn) => eventListeners.get(name)?.delete(fn)
window.dispatchEvent = event => {
	for (const fn of eventListeners.get(event.type) || []) fn(event)
	return true
}
window.setTimeout = setTimeout
window.clearTimeout = clearTimeout
window.innerWidth = 1280
window.indexedDB = undefined

globalThis.CustomEvent = class CustomEvent extends Event {
	constructor(type, options = {}) { super(type, options); this.detail = options.detail }
}
globalThis.btoa = value => Buffer.from(String(value), 'binary').toString('base64')

function deepEqual(actual, expected) {
	if (expected && expected.__objectContaining) {
		return Object.entries(expected.value).every(([key, value]) => deepEqual(actual?.[key], value))
	}
	try { assert.deepStrictEqual(actual, expected); return true } catch { return false }
}

function makeExpect(actual) {
	const api = {
		toBe(expected) { assert.strictEqual(actual, expected) },
		toEqual(expected) { assert.ok(deepEqual(actual, expected), 'Expected values to be deeply equal') },
		toBeInstanceOf(expected) { assert.ok(actual instanceof expected) },
		toHaveBeenCalledTimes(expected) { assert.strictEqual(actual?.mock?.calls?.length, expected) },
		toHaveBeenCalled() { assert.ok((actual?.mock?.calls?.length ?? 0) > 0) },
		toHaveBeenCalledWith(...expected) { assert.ok((actual?.mock?.calls || []).some(call => call.length === expected.length && call.every((arg, i) => deepEqual(arg, expected[i])))) },
		toHaveLength(expected) { assert.strictEqual(actual?.length, expected) },
		toBeTruthy() { assert.ok(actual) },
		toBeUndefined() { assert.strictEqual(actual, undefined) },
		toBeNull() { assert.strictEqual(actual, null) },
		toContain(expected) { assert.ok(actual?.includes?.(expected)) },
		toMatch(expected) { assert.match(actual, expected) },
		resolves: {
			toEqual(expected) { return Promise.resolve(actual).then(value => { expect(value).toEqual(expected) }) },
		},
		rejects: {
			toThrow(expected) {
				return Promise.resolve(actual).then(
					() => { throw new Error('Expected promise to reject') },
					error => {
						if (expected) assert.match(String(error?.message || error), expected instanceof RegExp ? expected : new RegExp(String(expected)))
					},
				)
			},
		},
	}
	api.not = {
		toBe(expected) { assert.notStrictEqual(actual, expected) },
		toHaveBeenCalled() { assert.strictEqual(actual?.mock?.calls?.length ?? 0, 0) },
		toContain(expected) { assert.ok(!actual?.includes?.(expected)) },
		toBeNull() { assert.notStrictEqual(actual, null) },
		toBeUndefined() { assert.notStrictEqual(actual, undefined) },
	}
	return api
}
makeExpect.objectContaining = value => ({ __objectContaining: true, value })
globalThis.expect = makeExpect

function makeFn(implementation = () => undefined) {
	const fn = (...args) => {
		fn.mock.calls.push(args)
		if (fn._once.length) return fn._once.shift()(...args)
		return implementation(...args)
	}
	fn.mock = { calls: [] }
	fn._once = []
	fn.mockImplementation = impl => { implementation = impl; return fn }
	fn.mockReturnValue = value => { implementation = () => value; return fn }
	fn.mockReturnValueOnce = value => { fn._once.push(() => value); return fn }
	fn.mockResolvedValue = value => { implementation = () => Promise.resolve(value); return fn }
	fn.mockResolvedValueOnce = value => { fn._once.push(() => Promise.resolve(value)); return fn }
	fn.mockRejectedValue = value => { implementation = () => Promise.reject(value); return fn }
	fn.mockRejectedValueOnce = value => { fn._once.push(() => Promise.reject(value)); return fn }
	return fn
}
globalThis.jest = { fn: makeFn, restoreAllMocks() {} }

globalThis.describe = (_name, fn) => fn()
globalThis.beforeEach = fn => beforeEachHooks.push(fn)
globalThis.it = (name, fn) => tests.push({ name, fn })

const files = [
	'../web/pore-browser-completion-job.test.js',
	'../web/pore-browser-pcm-recorder.test.js',
	'../web/pore-browser-recording-lifecycle.test.js',
	'../web/pore-browser-runtime-transport.test.js',
	'../web/pore-recording-coordination.test.js',
	'../web/pore-recording-controller-persistence-safety.test.js',
	'../web/pore-recording-controller.test.js',
	'../web/pore-talk-recording-state-bridge.test.js',
	'../web/pore-talk-recording-ui.test.js',
	'../web/pore-talk-capture-init.test.js',
	'../web/pore-talk-audio-adapter.test.js',
	'../web/pore-talk-recording-host.test.js',
]

for (const file of files) await import(new URL(file, import.meta.url))

let failed = 0
for (const test of tests) {
	try {
		for (const hook of beforeEachHooks) await hook()
		let timer = null
		try {
			await Promise.race([
				test.fn(),
				new Promise((_, reject) => {
					timer = setTimeout(() => reject(new Error('JavaScript test timed out after 5s')), 5000)
				}),
			])
		} finally {
			if (timer) clearTimeout(timer)
		}
		process.stdout.write(`PASS ${test.name}\n`)
	} catch (error) {
		failed += 1
		process.stderr.write(`FAIL ${test.name}: ${error.stack || error}\n`)
	}
}

if (failed) {
	process.exit(1)
}
process.exit(0)