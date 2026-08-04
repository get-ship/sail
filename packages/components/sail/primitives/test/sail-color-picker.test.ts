import { afterEach, describe, expect, it } from 'vitest'
import '../src/sail-color-picker/sail-color-picker.compiled.js'
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

describe('<sail-color-picker>', () => {

  it('renders the hex input and preview swatch', async () => {
    const { el } = mount(reg, 'sail-color-picker', { value: '#ff0000' })
    await flush()
    expect(inputOf(el)).not.toBeNull()
    // Trigger is a button swatch (use:swatch) — always shows the color.
    expect(shadow(el).querySelector('button[aria-label="Choose color"]')).not.toBeNull()
    expect(panelOf(el)).toBeNull()
  })

  it('reflects value attribute round-trip', async () => {
    const { el } = mount(reg, 'sail-color-picker', { value: '#abcdef' })
    await flush()
    expect(el.getAttribute('value')).toBe('#abcdef')
    el.setAttribute('value', '#123456')
    await flush()
    expect(el.getAttribute('value')).toBe('#123456')
  })

  it('opens the panel on input click and renders swatches + HSL sliders', async () => {
    const { el } = mount(reg, 'sail-color-picker', { value: '#ff0000' })
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

  it('clicking a swatch commits its hex and dispatches sail-change', async () => {
    // Start away from black so the first base-ramp swatch is a real change.
    const { el } = mount(reg, 'sail-color-picker', { value: '#ff0000' })
    await flush()
    inputOf(el).click()
    await flush()
    const events: CustomEvent[] = []
    el.addEventListener('sail-change', (e) => { events.push(e as CustomEvent) })
    const sw =
      swatches(el).find((b) => (b.dataset.hex || '').toLowerCase() !== '#ff0000') ?? swatches(el)[0]!
    const hex = sw.dataset.hex!
    expect(hex.toLowerCase()).not.toBe('#ff0000')
    sw.click()
    await flush()
    const last = events.at(-1)!
    expect(last).toBeTruthy()
    expect(last.detail).toEqual({ value: hex })
    expect(last.bubbles).toBe(true)
    expect(last.composed).toBe(true)
    // Prop/event is the control SSOT (attribute reflection may lag under suppress).
    const live = (el as HTMLElement & { value?: string }).value ?? el.getAttribute('value')
    expect(String(live).toLowerCase()).toBe(hex.toLowerCase())
  })

  it('while open, ignores external value thrash so HSL does not jump mid-drag', async () => {
    const { el } = mount(reg, 'sail-color-picker', { value: '#ff0000' })
    await flush()
    // Open panel via trigger (button or text input).
    const trigger =
      (shadow(el).querySelector('button[aria-label="Choose color"]') as HTMLElement | null) ??
      inputOf(el)
    trigger.click()
    await flush()
    expect(panelOf(el)).not.toBeNull()

    const [h] = ranges(el)
    expect(h).toBeTruthy()
    // Start a drag gesture on hue.
    h!.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true }))
    h!.value = '200'
    h!.dispatchEvent(new Event('input', { bubbles: true }))
    await flush()
    const hueDuring = h!.value

    // Parent pushes a lossy oklch→hex rewrite (different hex, same-ish color).
    el.setAttribute('value', '#00ff00')
    await flush()

    // Slider must keep the user's draft, not snap to the external prop.
    expect(h!.value).toBe(hueDuring)
    expect(Number(h!.value)).toBe(200)
  })

  it('emits zero sail-change on mount', async () => {
    const events: CustomEvent[] = []
    const { el } = mount(reg, 'sail-color-picker', { value: '#5194d5' })
    el.addEventListener('sail-change', (e) => {
      events.push(e as CustomEvent)
    })
    await flush()
    await flush()
    expect(events).toHaveLength(0)
  })

  it('emits zero sail-change on open then dismiss without edit', async () => {
    const { el } = mount(reg, 'sail-color-picker', { value: '#5194d5' })
    await flush()
    const events: CustomEvent[] = []
    el.addEventListener('sail-change', (e) => {
      events.push(e as CustomEvent)
    })
    const trigger =
      (shadow(el).querySelector('button[aria-label="Choose color"]') as HTMLElement | null) ??
      inputOf(el)
    trigger.click()
    await flush()
    expect(panelOf(el)).not.toBeNull()
    // Outside dismiss (same path as studio: document pointerdown).
    document.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true }))
    await flush()
    expect(panelOf(el)).toBeNull()
    expect(events).toHaveLength(0)
    expect(el.getAttribute('value')).toBe('#5194d5')
  })

  it('variant=swatch: full-bleed trigger opens panel and swatch pick emits sail-change', async () => {
    // Studio themes color facet uses variant=swatch (no hex field).
    const { el } = mount(reg, 'sail-color-picker', {
      value: '#ff0000',
      variant: 'swatch',
    })
    await flush()
    expect(inputOf(el)).toBeNull()
    const trigger = shadow(el).querySelector(
      'button[data-color-trigger-btn][data-variant="swatch"], button[aria-label="Choose color"]',
    ) as HTMLButtonElement | null
    expect(trigger).not.toBeNull()
    trigger!.click()
    await flush()
    expect(panelOf(el)).not.toBeNull()
    const events: CustomEvent[] = []
    el.addEventListener('sail-change', (e) => {
      events.push(e as CustomEvent)
    })
    const sw =
      swatches(el).find((b) => (b.dataset.hex || '').toLowerCase() !== '#ff0000') ??
      swatches(el)[0]!
    const hex = sw.dataset.hex!
    sw.click()
    await flush()
    expect(events.length).toBeGreaterThanOrEqual(1)
    expect(events.at(-1)!.detail).toEqual({ value: hex })
    const live = (el as HTMLElement & { value?: string }).value ?? el.getAttribute('value')
    expect(String(live).toLowerCase()).toBe(hex.toLowerCase())
  })
})
