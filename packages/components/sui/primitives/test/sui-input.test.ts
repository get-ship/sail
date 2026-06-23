import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-input/sui-input.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const input = (el: HTMLElement) => shadow(el).querySelector('input') as HTMLInputElement

// ElementInternals.setFormValue is partial under jsdom — the FormData
// round-trip needs the real webview tier (proof owed in #164). Gate, not skip
// silently past the only assert.
const setFormValueAbsent =
  typeof (window as unknown as { ElementInternals?: { prototype: ElementInternals } })
    .ElementInternals?.prototype?.setFormValue !== 'function'

describe('<sui-input>', () => {

  it('renders a native input in shadow DOM', () => {
    const { el } = mount(reg, 'sui-input')
    expect(input(el)).not.toBeNull()
  })

  it('is form-associated', () => {
    const ctor = customElements.get('sui-input') as unknown as {
      formAssociated?: boolean
    }
    expect(ctor.formAssociated).toBe(true)
  })

  it.skipIf(setFormValueAbsent)('participates in FormData when named inside a <form>', async () => {
    const container = reg.mk()
    container.innerHTML = '<form><sui-input name="q" value="hello"></sui-input></form>'
    const form = container.querySelector('form') as HTMLFormElement
    await flush()
    const data = new FormData(form)
    expect(data.get('q')).toBe('hello')
  })

  it('formResetCallback restores value to the prop default (empty)', async () => {
    const { el } = mount(reg, 'sui-input', { value: 'typed' })
    await flush()
    ;(el as unknown as { formResetCallback?: () => void }).formResetCallback!()
    await flush()
    expect(input(el).value).toBe('')
  })

  it('formDisabledCallback drives the disabled prop', async () => {
    const { el } = mount(reg, 'sui-input')
    await flush()
    expect(el.hasAttribute('disabled')).toBe(false)
    ;(el as unknown as { formDisabledCallback?: (d: boolean) => void })
      .formDisabledCallback!(true)
    await flush()
    expect(el.hasAttribute('disabled')).toBe(true)
  })
})

describe('<sui-input> — event contracts', () => {
  it('typing fires sui-input with {value}; bubbles + composed', async () => {
    const { el } = mount(reg, 'sui-input')
    await flush()
    let evt: CustomEvent | null = null
    el.addEventListener('sui-input', (e) => { evt = e as CustomEvent })
    const inp = input(el)
    inp.value = 'hel'
    inp.dispatchEvent(new Event('input', { bubbles: true }))
    expect(evt).not.toBeNull()
    expect(evt!.detail).toEqual({ value: 'hel' })
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
  })

  it('native change fires sui-input-change with {value}; bubbles + composed', async () => {
    const { el } = mount(reg, 'sui-input')
    await flush()
    let evt: CustomEvent | null = null
    el.addEventListener('sui-input-change', (e) => { evt = e as CustomEvent })
    const inp = input(el)
    inp.value = 'hello'
    inp.dispatchEvent(new Event('change', { bubbles: true }))
    expect(evt).not.toBeNull()
    expect(evt!.detail).toEqual({ value: 'hello' })
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
  })
})
