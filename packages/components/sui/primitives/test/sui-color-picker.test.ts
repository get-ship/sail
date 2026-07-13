import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-color-picker/sui-color-picker.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => (el.shadowRoot ?? el)
const inputOf = (el: HTMLElement) => shadow(el).querySelector('input[type="text"]') as HTMLInputElement
const panelOf = (el: HTMLElement) => shadow(el).querySelector('[role="dialog"]') as HTMLElement | null
const swatches = (el: HTMLElement) =>
  Array.from(shadow(el).querySelectorAll('button[data-hex]')) as HTMLButtonElement[]
const ranges = (el: HTMLElement) =>
  Array.from(shadow(el).querySelectorAll('[role="dialog"] input[type="range"]')) as HTMLInputElement[]

describe('<sui-color-picker>', () => {

  it('renders the hex input and preview swatch', async () => {
    const { el } = mount(reg, 'sui-color-picker', { value: '#ff0000' })
    await flush()
    expect(inputOf(el)).not.toBeNull()
    expect(shadow(el).querySelector('span')).not.toBeNull()
    expect(panelOf(el)).toBeNull()
  })

  it('reflects value attribute round-trip', async () => {
    const { el } = mount(reg, 'sui-color-picker', { value: '#abcdef' })
    await flush()
    expect(el.getAttribute('value')).toBe('#abcdef')
    el.setAttribute('value', '#123456')
    await flush()
    expect(el.getAttribute('value')).toBe('#123456')
  })

  it('opens the panel on input click and renders swatches + HSL sliders', async () => {
    const { el } = mount(reg, 'sui-color-picker', { value: '#ff0000' })
    await flush()
    inputOf(el).click()
    await flush()
    expect(panelOf(el)).not.toBeNull()
    expect(swatches(el).length).toBeGreaterThanOrEqual(8)
    expect(ranges(el)).toHaveLength(3)
    const [h, s, l] = ranges(el)
    expect(h!.min).toBe('0')
    expect(h!.max).toBe('360')
    expect(s!.max).toBe('100')
    expect(l!.max).toBe('100')
  })

  it('clicking a swatch commits its hex and dispatches sui-change', async () => {
    const { el } = mount(reg, 'sui-color-picker', { value: '#000000' })
    await flush()
    inputOf(el).click()
    await flush()
    const events: CustomEvent[] = []
    el.addEventListener('sui-change', (e) => { events.push(e as CustomEvent) })
    const sw = swatches(el)[0]!
    const hex = sw.dataset.hex!
    sw.click()
    await flush()
    const last = events.at(-1)!
    expect(last.detail).toEqual({ value: hex })
    expect(last.bubbles).toBe(true)
    expect(last.composed).toBe(true)
    expect(el.getAttribute('value')).toBe(hex)
  })
})
