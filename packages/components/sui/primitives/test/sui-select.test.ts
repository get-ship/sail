import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-select/sui-select.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

// HOST-CONDITIONAL (intentional, not fossilized) — every `it.skipIf(setFormValueAbsent)`
// below is gated on jsdom's partial ElementInternals.setFormValue; the FormData
// round-trip needs the real webview tier (proof owed in #164). When jsdom ships
// a real setFormValue, the gate flips automatically — no test logic changes.
const setFormValueAbsent =
  typeof (window as unknown as { ElementInternals?: { prototype: ElementInternals } })
    .ElementInternals?.prototype?.setFormValue !== 'function'

const shadow = (el: HTMLElement) => el.shadowRoot!
const innerSelect = (el: HTMLElement) =>
  shadow(el).querySelector('select') as HTMLSelectElement

const threeOpts =
  '<option value="">Choose</option>' +
  '<option value="a">Alpha</option>' +
  '<option value="b">Bravo</option>'

describe('<sui-select>', () => {

  it('default value is empty', async () => {
    const { el } = mount(reg, 'sui-select', {}, threeOpts)
    await flush()
    expect(innerSelect(el).value).toBe('')
  })

  it('renders a shadow <select> and the chrome parts', async () => {
    const { el } = mount(reg, 'sui-select')
    await flush()
    const root = shadow(el)
    expect(root.querySelector('[part~="field"]')).not.toBeNull()
    expect(root.querySelector('[part~="select"]')).not.toBeNull()
    expect(root.querySelector('[part~="chevron"]')).not.toBeNull()
  })

  it('clones light-DOM <option>s into the shadow <select>', async () => {
    const { el } = mount(reg, 'sui-select', {}, threeOpts)
    await flush()
    const opts = innerSelect(el).querySelectorAll('option')
    expect(opts.length).toBe(3)
    expect(opts[1].value).toBe('a')
    expect(opts[2].textContent).toBe('Bravo')
  })

  it('clones <optgroup> children too', async () => {
    const html =
      '<optgroup label="g"><option value="x">X</option><option value="y">Y</option></optgroup>'
    const { el } = mount(reg, 'sui-select', {}, html)
    await flush()
    const sel = innerSelect(el)
    expect(sel.querySelectorAll('optgroup').length).toBe(1)
    expect(sel.querySelectorAll('option').length).toBe(2)
  })

  it('changing native select fires sui-change with detail.value', async () => {
    const { el } = mount(reg, 'sui-select', { value: 'a' }, threeOpts)
    await flush()
    let received: string | undefined
    el.addEventListener('sui-change', (e) => {
      received = (e as CustomEvent).detail.value
    })
    const sel = innerSelect(el)
    sel.value = 'b'
    sel.dispatchEvent(new Event('change', { bubbles: true }))
    expect(received).toBe('b')
  })

  it('sui-change bubbles + is composed', async () => {
    const { el, container } = mount(reg, 'sui-select', { value: 'a' }, threeOpts)
    await flush()
    let outside = false
    container.addEventListener('sui-change', () => { outside = true })
    const sel = innerSelect(el)
    sel.value = 'b'
    sel.dispatchEvent(new Event('change', { bubbles: true }))
    expect(outside).toBe(true)
  })

  it('sui-change event contract: bubbles/composed flags + {value} payload shape', async () => {
    const { el } = mount(reg, 'sui-select', { value: 'a' }, threeOpts)
    await flush()
    let evt: CustomEvent | null = null
    el.addEventListener('sui-change', (e) => { evt = e as CustomEvent })
    const sel = innerSelect(el)
    sel.value = 'b'
    sel.dispatchEvent(new Event('change', { bubbles: true }))
    expect(evt).not.toBeNull()
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
    expect(evt!.detail).toEqual({ value: 'b' })
  })

  it('value prop reflects to underlying select.value', async () => {
    const { el } = mount(reg, 'sui-select', { value: 'a' }, threeOpts)
    await flush()
    expect(innerSelect(el).value).toBe('a')
  })

  it('updating the value attribute updates shadow select.value', async () => {
    const { el } = mount(reg, 'sui-select', { value: 'a' }, threeOpts)
    await flush()
    el.setAttribute('value', 'b')
    await flush()
    expect(innerSelect(el).value).toBe('b')
  })

  it('tone reflects on host', async () => {
    const { el } = mount(reg, 'sui-select', { tone: 'success' })
    await flush()
    expect(el.getAttribute('tone')).toBe('success')
  })

  it('size reflects on host', async () => {
    const { el } = mount(reg, 'sui-select', { size: 'sm' })
    await flush()
    expect(el.getAttribute('size')).toBe('sm')
  })

  it('disabled reflects and disables inner select', async () => {
    const { el } = mount(reg, 'sui-select', { disabled: true }, threeOpts)
    await flush()
    expect(el.hasAttribute('disabled')).toBe(true)
    expect(innerSelect(el).disabled).toBe(true)
  })

  it('required reflects and propagates to inner select', async () => {
    const { el } = mount(reg, 'sui-select', { required: true }, threeOpts)
    await flush()
    expect(el.hasAttribute('required')).toBe(true)
    expect(innerSelect(el).required).toBe(true)
  })

  it('invalid reflects and sets aria-invalid on inner select', async () => {
    const { el } = mount(reg, 'sui-select', { invalid: true }, threeOpts)
    await flush()
    expect(el.hasAttribute('invalid')).toBe(true)
    expect(innerSelect(el).getAttribute('aria-invalid')).toBe('true')
  })

  it('aria-label passes through from label prop', async () => {
    const { el } = mount(reg, 'sui-select', { label: 'Pick one' }, threeOpts)
    await flush()
    expect(innerSelect(el).getAttribute('aria-label')).toBe('Pick one')
  })

  it('aria-describedby passes through from described-by prop', async () => {
    const { el } = mount(reg, 'sui-select', { 'described-by': 'hint' }, threeOpts)
    await flush()
    expect(innerSelect(el).getAttribute('aria-describedby')).toBe('hint')
  })

  it('placeholder state: empty value adds data-placeholder on host', async () => {
    const { el } = mount(reg, 'sui-select', { value: '' }, threeOpts)
    await flush()
    expect(el.hasAttribute('data-placeholder')).toBe(true)
  })

  it('placeholder state cleared once a real value is chosen', async () => {
    const { el } = mount(reg, 'sui-select', { value: 'a' }, threeOpts)
    await flush()
    expect(el.hasAttribute('data-placeholder')).toBe(false)
  })

  it('renders the current option text in the visible label span', async () => {
    const { el } = mount(reg, 'sui-select', { value: 'b' }, threeOpts)
    await flush()
    const label = shadow(el).querySelector('.value') as HTMLSpanElement
    expect(label.textContent).toBe('Bravo')
  })

  it('falls back to raw value when no options match', async () => {
    const { el } = mount(reg, 'sui-select', { value: 'zz' })
    await flush()
    const label = shadow(el).querySelector('.value') as HTMLSpanElement
    expect(label.textContent).toBe('zz')
  })

  it('form association: shadow select tracks the user-selected value', async () => {
    const { el } = mount(reg, 'sui-select', { value: 'a' }, threeOpts)
    await flush()
    const sel = innerSelect(el)
    expect(sel.value).toBe('a')
    sel.value = 'b'
    sel.dispatchEvent(new Event('change', { bubbles: true }))
    await flush()
    expect(sel.value).toBe('b')
  })

  it('formResetCallback restores the prop default', async () => {
    // Per the <prop form-value /> contract, reset goes to the declared
    // prop default (`""` for sui-select) — not the value captured at
    // connect time. Callers who want connect-time semantics should
    // implement them in their controller.
    const { el } = mount(reg, 'sui-select', { value: 'a' }, threeOpts)
    await flush()
    const sel = innerSelect(el)
    sel.value = 'b'
    sel.dispatchEvent(new Event('change', { bubbles: true }))
    await flush()
    expect(sel.value).toBe('b')
    const host = el as HTMLElement & { formResetCallback?: () => void }
    expect(typeof host.formResetCallback).toBe('function')
    host.formResetCallback!()
    await flush()
    expect(innerSelect(el).value).toBe('')
  })

  it('is form-associated', () => {
    const ctor = customElements.get('sui-select') as
      unknown as { formAssociated?: boolean }
    expect(ctor.formAssociated).toBe(true)
  })

  it.skipIf(setFormValueAbsent)('participates in FormData when named inside a <form>', async () => {
    const container = reg.mk()
    container.innerHTML = `<form><sui-select name="fruit" value="a">${threeOpts}</sui-select></form>`
    const form = container.querySelector('form') as HTMLFormElement
    await flush()
    const data = new FormData(form)
    expect(data.get('fruit')).toBe('a')
  })

  it('formStateRestoreCallback sets the value', async () => {
    const { el } = mount(reg, 'sui-select', { value: 'a' }, threeOpts)
    await flush()
    const host = el as HTMLElement & {
      formStateRestoreCallback?: (s: string | null) => void
    }
    expect(typeof host.formStateRestoreCallback).toBe('function')
    host.formStateRestoreCallback!('b')
    await flush()
    expect(innerSelect(el).value).toBe('b')
  })

  it('cleanup on disconnect — disconnects without leaking', () => {
    const { el, container } = mount(reg, 'sui-select', {}, threeOpts)
    expect(el.isConnected).toBe(true)
    container.remove()
    expect(el.isConnected).toBe(false)
    // A fresh instance mounts cleanly afterwards — proves no global leak.
    const { el: el2 } = mount(reg, 'sui-select', {}, threeOpts)
    expect(innerSelect(el2)).not.toBeNull()
  })
})
