import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-autocomplete/sui-autocomplete.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const inputOf = (el: HTMLElement) => shadow(el).querySelector('input[role="combobox"]') as HTMLInputElement
const opts = (el: HTMLElement) => Array.from(shadow(el).querySelectorAll('li.opt[data-value]')) as HTMLLIElement[]

describe('<sui-autocomplete>', () => {

  it('renders a shadow root with role=combobox aria-autocomplete=list', async () => {
    const { el } = mount(reg, 'sui-autocomplete')
    await flush()
    const inp = inputOf(el)
    expect(inp).not.toBeNull()
    expect(inp.getAttribute('role')).toBe('combobox')
    expect(inp.getAttribute('aria-autocomplete')).toBe('list')
  })

  it('reads suggestions from light-DOM <option> children', async () => {
    const { el } = mount(reg, 'sui-autocomplete', {}, '<option value="apple">Apple</option><option value="apricot">Apricot</option>')
    await flush()
    inputOf(el).focus()
    await flush()
    expect(opts(el).map((o) => o.dataset.value)).toEqual(['apple', 'apricot'])
  })

  it('reads suggestions from suggestions="…JSON…" attribute', async () => {
    const { el } = mount(reg, 'sui-autocomplete', { suggestions: JSON.stringify(['cat', 'cab', 'car']) })
    await flush()
    inputOf(el).focus()
    await flush()
    expect(opts(el).map((o) => o.dataset.value)).toEqual(['cat', 'cab', 'car'])
  })

  it('Enter commits the typed value when no suggestion is highlighted', async () => {
    const { el } = mount(reg, 'sui-autocomplete')
    await flush()
    const events: CustomEvent[] = []
    el.addEventListener('sui-change', (e) => { events.push(e as CustomEvent) })
    const inp = inputOf(el)
    inp.focus()
    inp.value = 'freeform'
    inp.dispatchEvent(new Event('input', { bubbles: true }))
    await flush()
    inp.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true }))
    await flush()
    const last = events.at(-1)!
    expect(last.detail).toEqual({ value: 'freeform', label: 'freeform' })
    expect(last.bubbles).toBe(true)
    expect(last.composed).toBe(true)
    expect(inp.value).toBe('freeform')
  })

  it('reflects value attribute round-trip', async () => {
    const { el } = mount(reg, 'sui-autocomplete', { value: 'hello' })
    await flush()
    expect(el.getAttribute('value')).toBe('hello')
    el.setAttribute('value', 'world')
    await flush()
    expect(el.getAttribute('value')).toBe('world')
  })
})
