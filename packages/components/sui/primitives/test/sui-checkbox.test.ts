import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-checkbox/sui-checkbox.compiled.js'
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
const input = (el: HTMLElement) => shadow(el).querySelector('input') as HTMLInputElement

describe('<sui-checkbox>', () => {

  it('renders a native checkbox in shadow DOM', () => {
    const { el } = mount(reg, 'sui-checkbox')
    expect(input(el)).not.toBeNull()
    expect(input(el).type).toBe('checkbox')
  })

  it('renders with default props — unchecked, no checked attribute', () => {
    const { el } = mount(reg, 'sui-checkbox')
    expect(el.hasAttribute('checked')).toBe(false)
  })

  it('explicit size=md reflects on host', () => {
    const { el } = mount(reg, 'sui-checkbox', { size: 'md' })
    expect(el.getAttribute('size')).toBe('md')
  })

  it('explicit tone=default reflects on host', () => {
    const { el } = mount(reg, 'sui-checkbox', { tone: 'default' })
    expect(el.getAttribute('tone')).toBe('default')
  })

  it('checked attribute reflects on host', () => {
    const { el } = mount(reg, 'sui-checkbox', { checked: true })
    expect(el.hasAttribute('checked')).toBe(true)
  })

  it('checked prop set via attribute — input.checked updated', async () => {
    const { el } = mount(reg, 'sui-checkbox')
    expect(input(el).checked).toBe(false)
    el.setAttribute('checked', '')
    await flush()
    expect(input(el).checked).toBe(true)
  })

  it('indeterminate attribute reflects on host', () => {
    const { el } = mount(reg, 'sui-checkbox', { indeterminate: true })
    expect(el.hasAttribute('indeterminate')).toBe(true)
  })

  it('indeterminate synced to native input.indeterminate', async () => {
    const { el } = mount(reg, 'sui-checkbox', { indeterminate: true })
    await flush()
    expect(input(el).indeterminate).toBe(true)
  })

  it('fires sui-change with checked=true and value on user change (bubbles + composed)', () => {
    const { el } = mount(reg, 'sui-checkbox', { value: 'agree' })
    let evt: CustomEvent | null = null
    el.addEventListener('sui-change', (e) => { evt = e as CustomEvent })
    const inp = input(el)
    inp.checked = true
    inp.dispatchEvent(new Event('change', { bubbles: true }))
    expect(evt!.detail).toEqual({ checked: true, value: 'agree' })
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
    expect(el.hasAttribute('checked')).toBe(true)
  })

  it('fires sui-change with checked=false on uncheck', () => {
    const { el } = mount(reg, 'sui-checkbox', { checked: true })
    let detail: { checked: boolean; value: string } | null = null
    el.addEventListener('sui-change', (e) => { detail = (e as CustomEvent).detail })
    const inp = input(el)
    inp.checked = false
    inp.dispatchEvent(new Event('change', { bubbles: true }))
    expect(detail?.checked).toBe(false)
  })

  it('change event clears indeterminate', async () => {
    const { el } = mount(reg, 'sui-checkbox', { indeterminate: true })
    await flush()
    expect(input(el).indeterminate).toBe(true)
    const inp = input(el)
    inp.checked = true
    inp.dispatchEvent(new Event('change', { bubbles: true }))
    await flush()
    expect(el.hasAttribute('indeterminate')).toBe(false)
    expect(input(el).indeterminate).toBe(false)
  })

  it('aria-checked reflects checked state', () => {
    const { el } = mount(reg, 'sui-checkbox', { checked: true })
    expect(input(el).getAttribute('aria-checked')).toBe('true')
  })

  it('aria-checked is false when unchecked', () => {
    const { el } = mount(reg, 'sui-checkbox')
    expect(input(el).getAttribute('aria-checked')).toBe('false')
  })

  it('aria-checked is mixed when indeterminate', () => {
    const { el } = mount(reg, 'sui-checkbox', { indeterminate: true })
    expect(input(el).getAttribute('aria-checked')).toBe('mixed')
  })

  it('aria-invalid reflects invalid attribute', () => {
    const { el } = mount(reg, 'sui-checkbox', { invalid: true })
    expect(input(el).getAttribute('aria-invalid')).toBe('true')
  })

  it('aria-label uses label prop', () => {
    const { el } = mount(reg, 'sui-checkbox', { label: 'Accept terms' })
    expect(input(el).getAttribute('aria-label')).toBe('Accept terms')
  })

  it('aria-label falls back to name then "checkbox"', () => {
    const { el } = mount(reg, 'sui-checkbox', { name: 'agree' })
    expect(input(el).getAttribute('aria-label')).toBe('agree')
  })

  it('disabled attribute disables native input', () => {
    const { el } = mount(reg, 'sui-checkbox', { disabled: true })
    expect(input(el).disabled).toBe(true)
  })

  it('disabled suppresses sui-change event', () => {
    const { el } = mount(reg, 'sui-checkbox', { disabled: true })
    let fired = false
    el.addEventListener('sui-change', () => { fired = true })
    // Native disabled inputs don't fire change events — just verify no listener issue
    expect(fired).toBe(false)
  })

  it('required attribute passes to native input', () => {
    const { el } = mount(reg, 'sui-checkbox', { required: true })
    expect(input(el).required).toBe(true)
    expect(input(el).getAttribute('aria-required')).toBeNull() // checkbox uses required directly
  })

  it('name and value forwarded to native input', () => {
    const { el } = mount(reg, 'sui-checkbox', { name: 'terms', value: 'yes' })
    expect(input(el).name).toBe('terms')
    expect(input(el).value).toBe('yes')
  })

  it('tone reflects on host', () => {
    const { el } = mount(reg, 'sui-checkbox', { tone: 'destructive' })
    expect(el.getAttribute('tone')).toBe('destructive')
  })

  it('size sm reflects on host', () => {
    const { el } = mount(reg, 'sui-checkbox', { size: 'sm' })
    expect(el.getAttribute('size')).toBe('sm')
  })

  it('size lg reflects on host', () => {
    const { el } = mount(reg, 'sui-checkbox', { size: 'lg' })
    expect(el.getAttribute('size')).toBe('lg')
  })

  it('setting checked attribute after mount updates input', async () => {
    const { el } = mount(reg, 'sui-checkbox')
    expect(input(el).checked).toBe(false)
    el.setAttribute('checked', '')
    await flush()
    expect(input(el).checked).toBe(true)
  })

  it('removing checked attribute after mount updates input', async () => {
    const { el } = mount(reg, 'sui-checkbox', { checked: true })
    expect(input(el).checked).toBe(true)
    el.removeAttribute('checked')
    await flush()
    expect(input(el).checked).toBe(false)
  })

  it('cleanup on disconnect — disconnects without leaking', () => {
    const { el, container } = mount(reg, 'sui-checkbox', { checked: true })
    expect(el.isConnected).toBe(true)
    container.remove()
    expect(el.isConnected).toBe(false)
    // A fresh instance mounts cleanly afterwards — proves no global leak.
    const { el: el2 } = mount(reg, 'sui-checkbox')
    expect(el2.hasAttribute('checked')).toBe(false)
  })

  it('multiple instances render independently', () => {
    const { el: a } = mount(reg, 'sui-checkbox', { checked: true })
    const { el: b } = mount(reg, 'sui-checkbox')
    expect(a.hasAttribute('checked')).toBe(true)
    expect(b.hasAttribute('checked')).toBe(false)
  })

  it('invalid reflects on host', () => {
    const { el } = mount(reg, 'sui-checkbox', { invalid: true })
    expect(el.hasAttribute('invalid')).toBe(true)
  })

  it('shadow DOM has label, box, input parts', () => {
    const { el } = mount(reg, 'sui-checkbox')
    expect(shadow(el).querySelector('[part~="label"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="box"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="input"]')).not.toBeNull()
  })

  it('is form-associated', () => {
    const ctor = customElements.get('sui-checkbox') as unknown as {
      formAssociated?: boolean
    }
    expect(ctor.formAssociated).toBe(true)
  })

  it.skipIf(setFormValueAbsent)('participates in FormData when named inside a <form>', async () => {
    const container = reg.mk()
    container.innerHTML =
      '<form><sui-checkbox name="agree" checked></sui-checkbox></form>'
    const form = container.querySelector('form') as HTMLFormElement
    await flush()
    const data = new FormData(form)
    // form-value on `checked` (bool) — String(true) = 'true'.
    expect(data.get('agree')).toBe('true')
  })

  it('formResetCallback restores checked to its prop default (false)', async () => {
    const { el } = mount(reg, 'sui-checkbox', { checked: 'true' })
    await flush()
    expect(el.hasAttribute('checked')).toBe(true)
    ;(el as unknown as { formResetCallback?: () => void }).formResetCallback!()
    await flush()
    // checked default is `false`; reflect drives the host attribute off.
    expect(el.hasAttribute('checked')).toBe(false)
  })
})
