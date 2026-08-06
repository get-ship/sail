import { afterEach, describe, expect, it, vi } from 'vitest'
import '../src/sail-timeline/sail-timeline.compiled.js'
import '../src/sail-timeline/sail-timeline-item.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => (el.shadowRoot ?? el)

// ── <sail-timeline> ────────────────────────────────────────────────────────────

describe('<sail-timeline>', () => {

  it('renders a shadow root with a list div and slot', () => {
    const { el } = mount(reg, 'sail-timeline')
    expect(shadow(el).querySelector('slot')).not.toBeNull()
  })

  it('renders the list container', () => {
    const { el } = mount(reg, 'sail-timeline')
    expect(shadow(el).querySelector('slot')).not.toBeNull()
  })

  it('host has role=list', async () => {
    const { el } = mount(reg, 'sail-timeline')
    await flush()
    expect(el.getAttribute('role')).toBe('list')
  })
})

// ── <sail-timeline-item> ───────────────────────────────────────────────────────

describe('<sail-timeline-item>', () => {

  it('projects a title and a body', () => {
    const { el } = mount(reg, 'sail-timeline-item')
    expect(shadow(el).querySelector('slot[name="title"]')).not.toBeNull()
    expect(shadow(el).querySelector('slot:not([name])')).not.toBeNull()
  })

  it('tone reflects as attribute', () => {
    const { el } = mount(reg, 'sail-timeline-item', { tone: 'success' })
    expect(el.getAttribute('tone')).toBe('success')
  })

  it('the timestamp renders as a <time>', async () => {
    const { el } = mount(reg, 'sail-timeline-item', { at: '1m ago' })
    await flush()
    const atEl = shadow(el).querySelector('time')
    expect(atEl).not.toBeNull()
    expect(atEl!.textContent).toBe('1m ago')
  })

  it('no <time> when there is no timestamp', async () => {
    const { el } = mount(reg, 'sail-timeline-item')
    await flush()
    expect(shadow(el).querySelector('time')).toBeNull()
  })

  it('host has role=listitem by default', async () => {
    const { el } = mount(reg, 'sail-timeline-item')
    await flush()
    expect(el.getAttribute('role')).toBe('listitem')
  })

  it('host has role=button when interactive', async () => {
    const { el } = mount(reg, 'sail-timeline-item', { interactive: true })
    await flush()
    expect(el.getAttribute('role')).toBe('button')
  })

  it('host gets tabindex=0 when interactive', async () => {
    const { el } = mount(reg, 'sail-timeline-item', { interactive: true })
    await flush()
    expect(el.getAttribute('tabindex')).toBe('0')
  })

  it('invokes onSelect on click when interactive (empty detail)', async () => {
    const { el } = mount(reg, 'sail-timeline-item', { interactive: true })
    await flush()
    const fn = vi.fn()
    (el as any).onSelect = (d: any) => { fn(d) }
    el.click()
    expect(fn).toHaveBeenCalledOnce()
    expect(fn.mock.calls[0]![0]).toEqual({})
  })

  it('does not invoke onSelect on click when not interactive', async () => {
    const { el } = mount(reg, 'sail-timeline-item')
    await flush()
    const fn = vi.fn()
    (el as any).onSelect = (d: any) => { fn(d) }
    el.click()
    expect(fn).not.toHaveBeenCalled()
  })

  it('invokes onSelect on Enter when interactive', async () => {
    const { el } = mount(reg, 'sail-timeline-item', { interactive: true })
    await flush()
    const fn = vi.fn()
    (el as any).onSelect = (d: any) => { fn(d) }
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    expect(fn).toHaveBeenCalledOnce()
  })

  it('invokes onSelect on Space when interactive', async () => {
    const { el } = mount(reg, 'sail-timeline-item', { interactive: true })
    await flush()
    const fn = vi.fn()
    (el as any).onSelect = (d: any) => { fn(d) }
    el.dispatchEvent(new KeyboardEvent('keydown', { key: ' ', bubbles: true }))
    expect(fn).toHaveBeenCalledOnce()
  })

  it('does not invoke onSelect on Enter when not interactive', async () => {
    const { el } = mount(reg, 'sail-timeline-item')
    await flush()
    const fn = vi.fn()
    (el as any).onSelect = (d: any) => { fn(d) }
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    expect(fn).not.toHaveBeenCalled()
  })
})

// ── integration ───────────────────────────────────────────────────────────────

describe('<sail-timeline> + <sail-timeline-item> integration', () => {
  it('renders items inside the timeline list slot', async () => {
    const container = reg.mk()
    container.innerHTML = `
      <sail-timeline>
        <sail-timeline-item at="1m ago" tone="success">
          <span slot="title">Deployed</span>body text
        </sail-timeline-item>
        <sail-timeline-item tone="info">
          <span slot="title">Started</span>
        </sail-timeline-item>
      </sail-timeline>
    `
    const timeline = container.querySelector('sail-timeline') as HTMLElement
    await flush()
    const items = timeline.querySelectorAll('sail-timeline-item')
    expect(items.length).toBe(2)
    const first = items[0] as HTMLElement
    expect(first.getAttribute('tone')).toBe('success')
    expect(first.getAttribute('at')).toBe('1m ago')
    expect(shadow(first).querySelector('time')!.textContent).toBe('1m ago')
  })
})
