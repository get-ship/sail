// Reactive DOM bindings. Each binding subscribes to its dependency
// signals and applies updates to one DOM node. Disposers returned
// for cleanup wiring (sail's component wrapper chains them).
//

import { effect, type Signal } from './signal'
import { morph } from './morph'

/**
 * Cleanup function returned by every `bind*` helper and `effect`. Calling
 * it detaches the binding's listeners and stops further updates.
 */
export type Disposer = () => void

/**
 * Bind reactive text content to a node. Reads `get()` once eagerly and
 * again on every tracked-dependency change; the result is coerced to a
 * string and written to `node.textContent`. `null` and `undefined`
 * become `''` so missing values don't leak the literal text.
 *
 * Compiler target: `<el>{expr}</el>` interpolations.
 *
 * @param node Any DOM node with a `textContent` setter.
 * @param get Reactive read of the value to display.
 * @returns Disposer that stops the binding.
 *
 * @example
 * ```ts
 * const greeting = signal('hi')
 * bindText(span, () => greeting())   // span text follows greeting()
 * ```
 *
 * @see bindAttr, bindProp for non-text targets.
 */
export function bindText(node: Node, get: () => unknown): Disposer {
  let prev: string | undefined
  return effect(() => {
    const v = get()
    const next = v == null ? '' : String(v)
    if (next === prev) return
    if (prev === undefined && node.textContent === next) {
      prev = next
      return
    }
    node.textContent = next
    prev = next
  })
}

/**
 * Toggle visibility based on a truthiness check. Falsy → `display:none`;
 * truthy → restores the previous `display` value (preserving any inline
 * value that was there before the binding hid the element).
 *
 * Use `bindShow` when the element should stay in the DOM (preserves
 * focus, scroll, child state); use `bindIf` when the branch should be
 * torn down and rebuilt.
 *
 * @param el Element whose `style.display` is toggled.
 * @param get Reactive predicate; truthy → visible, falsy → hidden.
 * @returns Disposer that stops the binding.
 *
 * @example
 * ```ts
 * const expanded = signal(false)
 * bindShow(panel, () => expanded())
 * ```
 *
 * @see bindIf for branch swap.
 */
export function bindShow(el: HTMLElement, get: () => unknown): Disposer {
  let prev: string | null = null
  return effect(() => {
    if (get()) {
      if (prev !== null) el.style.display = prev
      prev = null
    } else {
      if (prev === null) prev = el.style.display
      el.style.display = 'none'
    }
  })
}

/**
 * Tone group → members → public token name (mirrors registry-derived TONE table).
 * Used when `data-sail-bridge` marks a group-typed attr so dynamic values
 * reflect CSS var *references* (`--tone: var(--primary)`).
 */
const TONE_BRIDGE: Record<string, Record<string, string>> = {
  primary: { '': 'primary', foreground: 'primary-foreground', hover: 'primary-hover' },
  info: { '': 'info', foreground: 'info-foreground' },
  success: { '': 'success', foreground: 'success-foreground' },
  warning: { '': 'warning', foreground: 'warning-foreground' },
  destructive: { '': 'destructive', foreground: 'destructive-foreground' },
  accent: { '': 'accent', foreground: 'accent-foreground' },
  muted: { '': 'muted', foreground: 'muted-foreground' },
}

const TONE_BRIDGE_MEMBERS = ['', 'foreground', 'hover'] as const

/** Reflect a Tone group value onto `--{prop}` / `--{prop}-foreground` bridges. */
function reflectToneBridge(el: Element, prop: string, value: string | null): void {
  for (const m of TONE_BRIDGE_MEMBERS) {
    const css = m ? `--${prop}-${m}` : `--${prop}`
    writeCssVar(el, css, null)
  }
  if (value == null || value === '') return
  const group = TONE_BRIDGE[value]
  if (!group) return
  for (const [m, pub] of Object.entries(group)) {
    const css = m ? `--${prop}-${m}` : `--${prop}`
    writeCssVar(el, css, `var(--${pub})`)
  }
}

function bridgePropsFor(el: Element): string[] {
  const raw = el.getAttribute('data-sail-bridge')
  if (!raw) return []
  return raw.split(/\s+/).filter(Boolean)
}

/**
 * Bind a reactive value to a DOM attribute, with attribute-presence
 * semantics built in.
 *
 * Omission rules — chosen so authors write the obvious thing and get
 * correct DOM/ARIA semantics without thinking about absence:
 *   - `null`, `undefined`, `false`, `''` → remove the attribute
 *   - `true` → presence-only (`disabled`, `required`, ...)
 *   - anything else → `setAttribute(name, String(value))`
 *
 * The empty-string case matters for ARIA: `<input aria-label="">`
 * overrides the accessible-name computation, so silently writing it
 * is a real divergence. Removing empty strings means
 * `aria-label={maybeLabel}` does the right thing in either case.
 *
 * @param el Element whose attribute is mutated.
 * @param name Attribute name (kebab-case, as written in HTML).
 * @param get Reactive read of the attribute value.
 * @returns Disposer that stops the binding.
 *
 * @example
 * ```ts
 * const busy = signal(false)
 * bindAttr(btn, 'aria-busy', () => busy())
 * busy.set(true)   // <button aria-busy>
 * busy.set(false)  // <button>
 * ```
 *
 * @see bindProp when binding to a typed JS property.
 */
export function bindAttr(el: Element, name: string, get: () => unknown): Disposer {
  let prev: unknown
  let init = false
  const bridges = bridgePropsFor(el)
  const isBridge = bridges.includes(name)
  return effect(() => {
    const v = get()
    if (init && v === prev) return
    const next =
      v == null || v === false || v === '' ? null : v === true ? '' : String(v)
    if (init || el.getAttribute(name) !== next) {
      if (next === null) el.removeAttribute(name)
      else el.setAttribute(name, next)
    }
    if (isBridge) {
      reflectToneBridge(el, name, next)
    }
    prev = v
    init = true
  })
}

/**
 * Compiler-private custom-property write. Authors never call this —
 * `bind(name)` in Style is the channel; the emitter writes this.
 * `null` removes the property.
 */
export function writeCssVar(el: Element, name: string, value: string | null): void {
  const style = (el as HTMLElement).style
  if (!style || typeof style.setProperty !== 'function') return
  if (value == null) style.removeProperty(name)
  else style.setProperty(name, value)
}

/**
 * Bind a value to a JS property of an element (not an attribute). Use
 * this when the target is a typed property — arrays, objects, instances
 * — so the value flows through without an `attribute → string → parse`
 * round-trip.
 *
 * The compiler emits `bindProp` whenever the right-hand side is an
 * expression of non-string type assigned to a custom-element prop:
 * `<my-component items={list}>`.
 *
 * @param el Element to write the property on.
 * @param prop Property name (camelCase, as on the JS object).
 * @param get Reactive read of the property value.
 * @returns Disposer that stops the binding.
 *
 * @example
 * ```ts
 * const items = signal<Row[]>([])
 * bindProp(grid, 'items', () => items())
 * ```
 *
 * @see bindAttr for HTML attributes; bindValue for two-way input binding.
 */
export function bindProp<T extends Element>(
  el: T,
  prop: string,
  get: () => unknown,
): Disposer {
  return effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    ;(el as any)[prop] = get()
  })
}

const camelize = (s: string): string =>
  s.includes('-') ? s.replace(/-([a-z])/g, (_m, c: string) => c.toUpperCase()) : s

/**
 * Route a dynamic binding to the right channel so the AUTHOR never has to.
 * This is the toolchain owning the DOM's attribute/property split: for a custom
 * element, a value that can't be an attribute (object / array / function), or a
 * name the element exposes as a property (a declared prop → accessor), is
 * assigned as a PROPERTY — identity preserved; the component reflects it to an
 * attribute itself if it wants (e.g. `variant`, `tone`). Everything else is a
 * plain HTML attribute (aria-*, data-*, native).
 *
 * The compiler emits this for a plain `name={expr}` binding on a custom element
 * (`<sail-table columns={cols} tone={t}>`). `attr:`/`prop:` remain as explicit
 * escape hatches for the rare case the author wants to force a channel.
 *
 * The channel is decided once, at bind time, from the element + first value —
 * stable for the life of the binding.
 */
export function bindDynamic(el: Element, name: string, get: () => unknown): Disposer {
  const camel = camelize(name)
  const first = get()
  // Only custom elements have a property/attribute duality worth routing.
  // Native elements keep pure-attribute semantics — identical to bindAttr.
  const asProp =
    el.tagName.includes('-') &&
    (camel in el ||
      (first !== null && (typeof first === 'object' || typeof first === 'function')))
  return asProp ? bindProp(el, camel, get) : bindAttr(el, name, get)
}

function isValueControl(
  el: EventTarget | null,
): el is HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement {
  return (
    el instanceof HTMLInputElement ||
    el instanceof HTMLTextAreaElement ||
    el instanceof HTMLSelectElement
  )
}

function firstValueControl(
  el: Element,
): HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement | null {
  if (isValueControl(el)) return el
  return el.querySelector<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>(
    'input:not([hidden]), textarea, select',
  )
}

function readBoundValue(el: Element, ev?: Event): string {
  if (isValueControl(ev?.target ?? null)) return (ev!.target as HTMLInputElement).value
  const inner = firstValueControl(el)
  if (inner) return inner.value
  const host = el as HTMLInputElement
  if (typeof host.value === 'string') return host.value
  return el.getAttribute('value') ?? ''
}

function writeBoundValue(el: Element, v: string): void {
  if (isValueControl(el)) {
    if (el.value !== v) el.value = v
    return
  }
  // Wrapper hosts (<slider> splices to a div): attribute is what rangePair
  // observes; `.value` hits the accessor rangePair installs; the inner
  // range is the last-resort write if children exist and no accessor did.
  if (el.getAttribute('value') !== v) el.setAttribute('value', v)
  const host = el as HTMLInputElement
  if (host.value !== v) host.value = v
  const inner = firstValueControl(el)
  if (inner && inner.value !== v) inner.value = v
}

/**
 * Two-way bind a string signal to an `<input>` / `<textarea>` /
 * `<select>` element — or a wrapper host (L1 `<slider>`) whose native
 * control is a descendant. Initial render and signal-driven updates flow
 * signal → element; user edits flow element → signal via the `input`
 * event. Setting the same value re-checks identity to avoid moving the
 * caret. Non-string signal reads are coerced so number knobs (`signal(1)`)
 * compare equal to the DOM string.
 *
 * The optional `onWrite` callback fires after each user-driven signal
 * write — use it for dispatching public events or syncing form state
 * without a separate `oninput` handler. It is NOT invoked when the
 * signal is updated programmatically (otherwise it would loop with
 * the signal subscription).
 *
 * Compiler target: `bind:value={state}` (no callback) or
 * `bind:value={state} on:bind={fn}` (with callback) template syntax.
 *
 * @param el Form control or wrapper host whose value is two-way bound.
 * @param sig String signal that owns the value.
 * @param onWrite Optional callback invoked with the new value after
 *   each user-input-driven signal write.
 * @returns Disposer that stops the binding and removes the listener.
 *
 * @example
 * ```ts
 * const draft = signal('')
 * bindValue(input, draft, (v) => el.dispatchEvent(new CustomEvent('change', { detail: v })))
 * ```
 *
 * @see bindChecked for boolean controls; bindProp for one-way prop binding.
 */
export function bindValue(
  el: Element,
  sig: Signal<string>,
  onWrite?: (value: string) => void,
): Disposer {
  const dispose = effect(() => {
    const raw = sig()
    const v = raw == null ? '' : String(raw)
    writeBoundValue(el, v)
  })
  const onInput = (ev: Event) => {
    const v = readBoundValue(el, ev)
    sig.set(v)
    if (onWrite) onWrite(v)
  }
  el.addEventListener('input', onInput)
  return () => {
    dispose()
    el.removeEventListener('input', onInput)
  }
}

/**
 * Two-way bind a boolean signal to a checkbox or radio input. Signal
 * writes set `el.checked`; the `change` event writes back into the
 * signal.
 *
 * Compiler target: `bind:checked={state}` on `<input type="checkbox">`
 * or `<input type="radio">`.
 *
 * @param el Checkbox or radio input.
 * @param sig Boolean signal that owns the checked state.
 * @returns Disposer that stops the binding and removes the listener.
 *
 * @example
 * ```ts
 * const agreed = signal(false)
 * bindChecked(checkbox, agreed)
 * ```
 *
 * @see bindValue for text-valued inputs.
 */
export function bindChecked(
  el: HTMLInputElement,
  sig: Signal<boolean>,
): Disposer {
  const dispose = effect(() => {
    const v = sig()
    if (el.checked !== v) el.checked = v
  })
  const onChange = () => sig.set(el.checked)
  el.addEventListener('change', onChange)
  return () => {
    dispose()
    el.removeEventListener('change', onChange)
  }
}

/**
 * Bind a reactive HTML/SVG string to an element's contents. The string
 * is parsed once and morphed into `el` via `morph()` — DOM identity
 * preserved where possible, so focus/scroll/selection survive updates.
 *
 * Use this when the markup is produced by an external generator
 * (markdown renderer, SVG library, code highlighter) and putting the
 * shape in the `.sail` template would mean re-encoding the generator's
 * output by hand. Reach for `{#if}` / `{#for}` / `{#match}` first;
 * `bindHtml` is the escape hatch when those don't fit.
 *
 * `<el>.innerHTML = …` from inside an effect is forbidden by
 * `sail-controller-purity` — use this instead. The `morph()` step
 * preserves identity that a raw `innerHTML =` would destroy.
 *
 * @param el Element whose children are reconciled.
 * @param get Reactive read of the markup string.
 * @returns Disposer that stops the binding.
 *
 * @example
 * ```ts
 * const html = signal('<p>hello</p>')
 * bindHtml(slotEl, () => html())
 * ```
 *
 * @see morph for the patcher; bindText for plain text targets.
 */
export function bindHtml(el: Element, get: () => string): Disposer {
  const SVG_NS = 'http://www.w3.org/2000/svg'
  const isSvg = el.namespaceURI === SVG_NS
  const tag = el.tagName.toLowerCase()
  return effect(() => {
    const html = get()
    const tmp = isSvg
      ? el.ownerDocument.createElementNS(SVG_NS, tag)
      : el.ownerDocument.createElement(tag)
    tmp.innerHTML = html
    // Children-only contract: mirror the bound element's own attributes onto
    // the template so morph's attribute pass is a no-op — a bare template
    // element would otherwise strip authored class/part/slot from the host.
    for (const a of el.attributes) tmp.setAttribute(a.name, a.value)
    morph(el, tmp)
  })
}
