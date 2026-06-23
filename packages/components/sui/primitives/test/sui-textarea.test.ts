import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-textarea/sui-textarea.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

// ElementInternals.setFormValue is partial under jsdom — the FormData
// round-trip needs the real webview tier (proof owed in #164). Gate, not skip
// silently past the only assert.
const setFormValueAbsent =
  typeof (window as unknown as { ElementInternals?: { prototype: ElementInternals } })
    .ElementInternals?.prototype?.setFormValue !== 'function'

const shadow = (el: HTMLElement) => el.shadowRoot!
const ta = (el: HTMLElement) => shadow(el).querySelector('textarea') as HTMLTextAreaElement

describe('<sui-textarea>', () => {

  it('renders a native textarea in shadow DOM', () => {
    const { el } = mount(reg, 'sui-textarea')
    expect(ta(el)).not.toBeNull()
  })

  it('renders with default rows=4', () => {
    const { el } = mount(reg, 'sui-textarea')
    expect(ta(el).rows).toBe(4)
  })

  it('rows prop passes to native textarea', () => {
    const { el } = mount(reg, 'sui-textarea', { rows: '6' })
    expect(ta(el).rows).toBe(6)
  })

  it('placeholder passes to native textarea', () => {
    const { el } = mount(reg, 'sui-textarea', { placeholder: 'Enter text...' })
    expect(ta(el).placeholder).toBe('Enter text...')
  })

  it('name passes to native textarea', () => {
    const { el } = mount(reg, 'sui-textarea', { name: 'body' })
    expect(ta(el).name).toBe('body')
  })

  it('fires sui-input on user keystroke', () => {
    const { el } = mount(reg, 'sui-textarea')
    let val: string | undefined
    el.addEventListener('sui-input', (e) => { val = (e as CustomEvent).detail.value })
    const t = ta(el)
    t.value = 'hello'
    t.dispatchEvent(new Event('input', { bubbles: true }))
    expect(val).toBe('hello')
  })

  it('sui-input event bubbles and is composed with {value} payload', () => {
    const { el, container } = mount(reg, 'sui-textarea')
    let evt: CustomEvent | null = null
    container.addEventListener('sui-input', (e) => { evt = e as CustomEvent })
    const t = ta(el)
    t.value = 'hi'
    t.dispatchEvent(new Event('input', { bubbles: true }))
    expect(evt).not.toBeNull()
    expect(evt!.detail).toEqual({ value: 'hi' })
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
  })

  it('fires sui-change on blur if value changed (bubbles + composed)', async () => {
    const { el } = mount(reg, 'sui-textarea')
    let evt: CustomEvent | null = null
    el.addEventListener('sui-change', (e) => { evt = e as CustomEvent })
    const t = ta(el)
    // Simulate typing
    t.value = 'typed text'
    t.dispatchEvent(new Event('input', { bubbles: true }))
    await flush()
    // Simulate blur
    t.dispatchEvent(new Event('blur', { bubbles: true }))
    expect(evt!.detail).toEqual({ value: 'typed text' })
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
  })

  it('does not fire sui-change on blur if value unchanged', () => {
    const { el } = mount(reg, 'sui-textarea')
    let fired = false
    el.addEventListener('sui-change', () => { fired = true })
    const t = ta(el)
    t.dispatchEvent(new Event('blur', { bubbles: true }))
    expect(fired).toBe(false)
  })

  it('disabled reflects on host', () => {
    const { el } = mount(reg, 'sui-textarea', { disabled: true })
    expect(el.hasAttribute('disabled')).toBe(true)
  })

  it('disabled disables native textarea', () => {
    const { el } = mount(reg, 'sui-textarea', { disabled: true })
    expect(ta(el).disabled).toBe(true)
  })

  it('required reflects on host', () => {
    const { el } = mount(reg, 'sui-textarea', { required: true })
    expect(el.hasAttribute('required')).toBe(true)
  })

  it('required passes to native textarea', () => {
    const { el } = mount(reg, 'sui-textarea', { required: true })
    expect(ta(el).required).toBe(true)
  })

  it('aria-required set when required', () => {
    const { el } = mount(reg, 'sui-textarea', { required: true })
    expect(ta(el).getAttribute('aria-required')).toBe('true')
  })

  it('readonly reflects on host', () => {
    const { el } = mount(reg, 'sui-textarea', { readonly: true })
    expect(el.hasAttribute('readonly')).toBe(true)
  })

  it('readonly passes to native textarea', () => {
    const { el } = mount(reg, 'sui-textarea', { readonly: true })
    expect(ta(el).readOnly).toBe(true)
  })

  it('invalid reflects on host', () => {
    const { el } = mount(reg, 'sui-textarea', { invalid: true })
    expect(el.hasAttribute('invalid')).toBe(true)
  })

  it('aria-invalid set when invalid', () => {
    const { el } = mount(reg, 'sui-textarea', { invalid: true })
    expect(ta(el).getAttribute('aria-invalid')).toBe('true')
  })

  it('aria-invalid is false when not invalid', () => {
    const { el } = mount(reg, 'sui-textarea')
    expect(ta(el).getAttribute('aria-invalid')).toBe('false')
  })

  it('aria-label set from label prop', () => {
    const { el } = mount(reg, 'sui-textarea', { label: 'Description' })
    expect(ta(el).getAttribute('aria-label')).toBe('Description')
  })

  it('aria-describedby set from described-by prop', () => {
    const { el } = mount(reg, 'sui-textarea', { 'described-by': 'hint-id' })
    expect(ta(el).getAttribute('aria-describedby')).toBe('hint-id')
  })

  it('tone reflects on host', () => {
    const { el } = mount(reg, 'sui-textarea', { tone: 'success' })
    expect(el.getAttribute('tone')).toBe('success')
  })

  it('variant code reflects on host', () => {
    const { el } = mount(reg, 'sui-textarea', { variant: 'code' })
    expect(el.getAttribute('variant')).toBe('code')
  })

  it('size sm reflects on host', () => {
    const { el } = mount(reg, 'sui-textarea', { size: 'sm' })
    expect(el.getAttribute('size')).toBe('sm')
  })

  it('part=textarea present on native element', () => {
    const { el } = mount(reg, 'sui-textarea')
    expect(shadow(el).querySelector('[part~="textarea"]')).not.toBeNull()
  })

  it('setting rows attribute reactively updates textarea', async () => {
    const { el } = mount(reg, 'sui-textarea')
    expect(ta(el).rows).toBe(4)
    el.setAttribute('rows', '8')
    await flush()
    expect(ta(el).rows).toBe(8)
  })

  it('cleanup on disconnect — disconnects without leaking', () => {
    const { el, container } = mount(reg, 'sui-textarea')
    expect(el.isConnected).toBe(true)
    container.remove()
    expect(el.isConnected).toBe(false)
    // A fresh instance mounts cleanly afterwards — proves no global leak.
    const { el: el2 } = mount(reg, 'sui-textarea')
    expect(ta(el2)).not.toBeNull()
  })

  it('is form-associated', () => {
    const ctor = customElements.get('sui-textarea') as unknown as {
      formAssociated?: boolean
    }
    expect(ctor.formAssociated).toBe(true)
  })

  it.skipIf(setFormValueAbsent)('participates in FormData when named inside a <form>', async () => {
    const container = reg.mk()
    container.innerHTML =
      '<form><sui-textarea name="note" value="hi"></sui-textarea></form>'
    const form = container.querySelector('form') as HTMLFormElement
    await flush()
    const data = new FormData(form)
    expect(data.get('note')).toBe('hi')
  })

  it('form.reset() restores the prop default', async () => {
    const container = reg.mk()
    container.innerHTML =
      '<form><sui-textarea name="note" value="initial"></sui-textarea></form>'
    const el = container.querySelector('sui-textarea') as HTMLElement
    await flush()
    ;(el as unknown as { formResetCallback?: () => void }).formResetCallback!()
    await flush()
    expect(ta(el).value).toBe('')
  })
})
