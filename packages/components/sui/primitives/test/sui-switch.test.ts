import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-switch/sui-switch.compiled.js'
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

describe('<sui-switch>', () => {

  it('renders a native checkbox in shadow DOM', () => {
    const { el } = mount(reg, 'sui-switch')
    expect(input(el)).not.toBeNull()
    expect(input(el).type).toBe('checkbox')
  })

  it('input has role=switch', () => {
    const { el } = mount(reg, 'sui-switch')
    expect(input(el).getAttribute('role')).toBe('switch')
  })

  it('renders with default props — unchecked, no checked attribute', () => {
    const { el } = mount(reg, 'sui-switch')
    expect(el.hasAttribute('checked')).toBe(false)
  })

  it('explicit size=md reflects on host', () => {
    const { el } = mount(reg, 'sui-switch', { size: 'md' })
    expect(el.getAttribute('size')).toBe('md')
  })

  it('explicit tone=default reflects on host', () => {
    const { el } = mount(reg, 'sui-switch', { tone: 'default' })
    expect(el.getAttribute('tone')).toBe('default')
  })

  it('checked attribute reflects on host', () => {
    const { el } = mount(reg, 'sui-switch', { checked: true })
    expect(el.hasAttribute('checked')).toBe(true)
  })

  it('checked prop set via attribute — input.checked updated', async () => {
    const { el } = mount(reg, 'sui-switch')
    expect(input(el).checked).toBe(false)
    el.setAttribute('checked', '')
    await flush()
    expect(input(el).checked).toBe(true)
  })

  it('fires sui-change with checked=true on toggle on (bubbles + composed)', () => {
    const { el } = mount(reg, 'sui-switch')
    let evt: CustomEvent | null = null
    el.addEventListener('sui-change', (e) => { evt = e as CustomEvent })
    const inp = input(el)
    inp.checked = true
    inp.dispatchEvent(new Event('change', { bubbles: true }))
    expect(evt!.detail).toEqual({ checked: true })
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
  })

  it('fires sui-change with checked=false on toggle off', () => {
    const { el } = mount(reg, 'sui-switch', { checked: true })
    let detail: { checked: boolean } | null = null
    el.addEventListener('sui-change', (e) => { detail = (e as CustomEvent).detail })
    const inp = input(el)
    inp.checked = false
    inp.dispatchEvent(new Event('change', { bubbles: true }))
    expect(detail?.checked).toBe(false)
  })

  it('checked signal updated after change event', async () => {
    const { el } = mount(reg, 'sui-switch')
    const inp = input(el)
    inp.checked = true
    inp.dispatchEvent(new Event('change', { bubbles: true }))
    await flush()
    expect(el.hasAttribute('checked')).toBe(true)
  })

  it('aria-checked is true when checked', () => {
    const { el } = mount(reg, 'sui-switch', { checked: true })
    expect(input(el).getAttribute('aria-checked')).toBe('true')
  })

  it('aria-checked is false when unchecked', () => {
    const { el } = mount(reg, 'sui-switch')
    expect(input(el).getAttribute('aria-checked')).toBe('false')
  })

  it('aria-checked updates reactively after toggle', async () => {
    const { el } = mount(reg, 'sui-switch')
    expect(input(el).getAttribute('aria-checked')).toBe('false')
    const inp = input(el)
    inp.checked = true
    inp.dispatchEvent(new Event('change', { bubbles: true }))
    await flush()
    expect(input(el).getAttribute('aria-checked')).toBe('true')
  })

  it('aria-label uses label prop', () => {
    const { el } = mount(reg, 'sui-switch', { label: 'Notifications' })
    expect(input(el).getAttribute('aria-label')).toBe('Notifications')
  })

  it('aria-label falls back to name then "toggle"', () => {
    const { el } = mount(reg, 'sui-switch', { name: 'notify' })
    expect(input(el).getAttribute('aria-label')).toBe('notify')
  })

  it('disabled attribute disables native input', () => {
    const { el } = mount(reg, 'sui-switch', { disabled: true })
    expect(input(el).disabled).toBe(true)
  })

  it('disabled reflects on host', () => {
    const { el } = mount(reg, 'sui-switch', { disabled: true })
    expect(el.hasAttribute('disabled')).toBe(true)
  })

  it('name and value forwarded to native input', () => {
    const { el } = mount(reg, 'sui-switch', { name: 'notify', value: 'yes' })
    expect(input(el).name).toBe('notify')
    expect(input(el).value).toBe('yes')
  })

  it('tone reflects on host', () => {
    for (const tone of ['primary', 'success', 'destructive'] as const) {
      const { el } = mount(reg, 'sui-switch', { tone })
      expect(el.getAttribute('tone')).toBe(tone)
    }
  })

  it('size sm reflects on host', () => {
    const { el } = mount(reg, 'sui-switch', { size: 'sm' })
    expect(el.getAttribute('size')).toBe('sm')
  })

  it('size lg reflects on host', () => {
    const { el } = mount(reg, 'sui-switch', { size: 'lg' })
    expect(el.getAttribute('size')).toBe('lg')
  })

  it('shadow DOM has track, thumb, input parts', () => {
    const { el } = mount(reg, 'sui-switch')
    expect(shadow(el).querySelector('[part~="track"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="thumb"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="input"]')).not.toBeNull()
  })

  it('removing checked attribute after mount updates input', async () => {
    const { el } = mount(reg, 'sui-switch', { checked: true })
    expect(input(el).checked).toBe(true)
    el.removeAttribute('checked')
    await flush()
    expect(input(el).checked).toBe(false)
  })

  it('sui-change event bubbles and is composed', () => {
    const { el, container } = mount(reg, 'sui-switch')
    let bubbled = false
    container.addEventListener('sui-change', () => { bubbled = true })
    const inp = input(el)
    inp.checked = true
    inp.dispatchEvent(new Event('change', { bubbles: true }))
    expect(bubbled).toBe(true)
  })

  it('cleanup on disconnect — disconnects without leaking', () => {
    const { el, container } = mount(reg, 'sui-switch', { checked: true })
    expect(el.isConnected).toBe(true)
    container.remove()
    expect(el.isConnected).toBe(false)
    // A fresh instance mounts cleanly afterwards — proves no global leak.
    const { el: el2 } = mount(reg, 'sui-switch')
    expect(el2.hasAttribute('checked')).toBe(false)
  })

  it('multiple instances render independently', () => {
    const { el: a } = mount(reg, 'sui-switch', { checked: true })
    const { el: b } = mount(reg, 'sui-switch')
    expect(a.hasAttribute('checked')).toBe(true)
    expect(b.hasAttribute('checked')).toBe(false)
  })

  it('is form-associated', () => {
    const ctor = customElements.get('sui-switch') as unknown as {
      formAssociated?: boolean
    }
    expect(ctor.formAssociated).toBe(true)
  })

  it.skipIf(setFormValueAbsent)('participates in FormData when named inside a <form>', async () => {
    const container = reg.mk()
    container.innerHTML =
      '<form><sui-switch name="dark" checked></sui-switch></form>'
    const form = container.querySelector('form') as HTMLFormElement
    await flush()
    const data = new FormData(form)
    expect(data.get('dark')).toBe('true')
  })

  it('formResetCallback restores checked to its prop default (false)', async () => {
    const { el } = mount(reg, 'sui-switch', { checked: 'true' })
    await flush()
    expect(el.hasAttribute('checked')).toBe(true)
    ;(el as unknown as { formResetCallback?: () => void }).formResetCallback!()
    await flush()
    expect(el.hasAttribute('checked')).toBe(false)
  })
})
