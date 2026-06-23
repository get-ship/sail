import { afterEach, describe, expect, it, vi } from 'vitest'
import '../src/sui-timeline/sui-timeline.compiled.js'
import '../src/sui-timeline/sui-timeline-item.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

// ── <sui-timeline> ────────────────────────────────────────────────────────────

describe('<sui-timeline>', () => {

  it('renders a shadow root with a list div and slot', () => {
    const { el } = mount(reg, 'sui-timeline')
    expect(shadow(el).querySelector('.list')).not.toBeNull()
    expect(shadow(el).querySelector('slot')).not.toBeNull()
  })

  it('list div has part=list', () => {
    const { el } = mount(reg, 'sui-timeline')
    expect(shadow(el).querySelector('[part~="list"]')).not.toBeNull()
  })

  it('host has role=list', async () => {
    const { el } = mount(reg, 'sui-timeline')
    await flush()
    expect(el.getAttribute('role')).toBe('list')
  })
})

// ── <sui-timeline-item> ───────────────────────────────────────────────────────

describe('<sui-timeline-item>', () => {

  it('renders gutter dot and body in shadow root', () => {
    const { el } = mount(reg, 'sui-timeline-item')
    expect(shadow(el).querySelector('.gutter')).not.toBeNull()
    expect(shadow(el).querySelector('[part="dot"]')).not.toBeNull()
    expect(shadow(el).querySelector('.body')).not.toBeNull()
  })

  it('dot has part=dot', () => {
    const { el } = mount(reg, 'sui-timeline-item')
    expect(shadow(el).querySelector('[part~="dot"]')).not.toBeNull()
  })

  it('body has part=body', () => {
    const { el } = mount(reg, 'sui-timeline-item')
    expect(shadow(el).querySelector('[part~="body"]')).not.toBeNull()
  })

  it('tone reflects as attribute', () => {
    const { el } = mount(reg, 'sui-timeline-item', { tone: 'success' })
    expect(el.getAttribute('tone')).toBe('success')
  })

  it('at attribute is rendered in .at span', async () => {
    const { el } = mount(reg, 'sui-timeline-item', { at: '1m ago' })
    await flush()
    const atEl = shadow(el).querySelector('.at')
    expect(atEl).not.toBeNull()
    expect(atEl!.textContent).toBe('1m ago')
  })

  it('.at element not rendered when at is empty', async () => {
    const { el } = mount(reg, 'sui-timeline-item')
    await flush()
    expect(shadow(el).querySelector('.at')).toBeNull()
  })

  it('at part is set on the .at element', async () => {
    const { el } = mount(reg, 'sui-timeline-item', { at: '2h ago' })
    await flush()
    const atEl = shadow(el).querySelector('.at')!
    expect(atEl.getAttribute('part')).toBe('at')
  })

  it('host has role=listitem by default', async () => {
    const { el } = mount(reg, 'sui-timeline-item')
    await flush()
    expect(el.getAttribute('role')).toBe('listitem')
  })

  it('host has role=button when interactive', async () => {
    const { el } = mount(reg, 'sui-timeline-item', { interactive: true })
    await flush()
    expect(el.getAttribute('role')).toBe('button')
  })

  it('host gets tabindex=0 when interactive', async () => {
    const { el } = mount(reg, 'sui-timeline-item', { interactive: true })
    await flush()
    expect(el.getAttribute('tabindex')).toBe('0')
  })

  it('dispatches sui-select on click when interactive (bubbles + composed, empty detail)', async () => {
    const { el } = mount(reg, 'sui-timeline-item', { interactive: true })
    await flush()
    const fn = vi.fn()
    el.addEventListener('sui-select', fn)
    el.click()
    expect(fn).toHaveBeenCalledOnce()
    const evt = fn.mock.calls[0]![0] as CustomEvent
    expect(evt.detail).toEqual({})
    expect(evt.bubbles).toBe(true)
    expect(evt.composed).toBe(true)
  })

  it('does not dispatch sui-select on click when not interactive', async () => {
    const { el } = mount(reg, 'sui-timeline-item')
    await flush()
    const fn = vi.fn()
    el.addEventListener('sui-select', fn)
    el.click()
    expect(fn).not.toHaveBeenCalled()
  })

  it('dispatches sui-select on Enter when interactive', async () => {
    const { el } = mount(reg, 'sui-timeline-item', { interactive: true })
    await flush()
    const fn = vi.fn()
    el.addEventListener('sui-select', fn)
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    expect(fn).toHaveBeenCalledOnce()
  })

  it('dispatches sui-select on Space when interactive', async () => {
    const { el } = mount(reg, 'sui-timeline-item', { interactive: true })
    await flush()
    const fn = vi.fn()
    el.addEventListener('sui-select', fn)
    el.dispatchEvent(new KeyboardEvent('keydown', { key: ' ', bubbles: true }))
    expect(fn).toHaveBeenCalledOnce()
  })

  it('does not dispatch sui-select on Enter when not interactive', async () => {
    const { el } = mount(reg, 'sui-timeline-item')
    await flush()
    const fn = vi.fn()
    el.addEventListener('sui-select', fn)
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    expect(fn).not.toHaveBeenCalled()
  })
})

// ── integration ───────────────────────────────────────────────────────────────

describe('<sui-timeline> + <sui-timeline-item> integration', () => {
  it('renders items inside the timeline list slot', async () => {
    const container = reg.mk()
    container.innerHTML = `
      <sui-timeline>
        <sui-timeline-item at="1m ago" tone="success">
          <span slot="title">Deployed</span>body text
        </sui-timeline-item>
        <sui-timeline-item tone="info">
          <span slot="title">Started</span>
        </sui-timeline-item>
      </sui-timeline>
    `
    const timeline = container.querySelector('sui-timeline') as HTMLElement
    await flush()
    const items = timeline.querySelectorAll('sui-timeline-item')
    expect(items.length).toBe(2)
    const first = items[0] as HTMLElement
    expect(first.getAttribute('tone')).toBe('success')
    expect(first.getAttribute('at')).toBe('1m ago')
    expect(shadow(first).querySelector('.at')!.textContent).toBe('1m ago')
  })
})
