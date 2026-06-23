// Behavioral suite for <sui-popover>. The component is a thin wrapper over the
// native popover API (popover="auto") — open/close, Escape-to-close,
// outside-click dismiss, and focus-return-to-trigger are DELEGATED to the
// platform, so the component's own contract is: the wiring that enables those
// (popover="auto", popovertarget, ARIA), reflecting `open`, and emitting a
// unified `sui-toggle` event. We assert that contract; the platform-owned
// dismissal semantics are covered by the browser, not this unit suite.

import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-popover/sui-popover.compiled.js'
import '../src/sui-portal/sui-portal.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const panelOf = (el: HTMLElement) =>
  shadow(el).querySelector('[part="panel"]') as HTMLElement | null

function makePopover(attrs: Record<string, string | boolean> = {}) {
  return mount(
    reg,
    'sui-popover',
    attrs,
    '<button slot="trigger">Open</button><div>Panel body</div>',
  )
}
const triggerOf = (el: HTMLElement) =>
  el.querySelector('[slot="trigger"]') as HTMLButtonElement

interface PopoverApi extends HTMLElement {
  show(): void
  hide(): void
  toggle(): void
}

describe('<sui-popover>', () => {

  it('closed by default — open attribute absent, trigger aria-expanded=false', async () => {
    const { el } = makePopover()
    await flush()
    expect(el.hasAttribute('open')).toBe(false)
    expect(triggerOf(el).getAttribute('aria-expanded')).toBe('false')
  })

  it('panel is a native auto-popover (delegates Escape + light-dismiss to the platform)', async () => {
    const { el } = makePopover()
    await flush()
    const panel = panelOf(el)!
    expect(panel.getAttribute('popover')).toBe('auto')
    expect(panel.getAttribute('role')).toBe('dialog')
  })

  it('wires the trigger to the panel: popovertarget + aria-haspopup + aria-controls', async () => {
    const { el } = makePopover()
    await flush()
    const trigger = triggerOf(el)
    const panel = panelOf(el)!
    expect(panel.id).not.toBe('')
    // The popovertarget linkage is what gives the trigger native open/close
    // + focus-return without a hand-rolled click handler.
    expect(trigger.getAttribute('popovertarget')).toBe(panel.id)
    expect(trigger.getAttribute('aria-controls')).toBe(panel.id)
    expect(trigger.getAttribute('aria-haspopup')).toBe('dialog')
  })

  it('opening via the open prop emits sui-toggle{open:true} (bubbles + composed) and flips aria-expanded', async () => {
    const { el } = makePopover()
    await flush()
    const events: CustomEvent[] = []
    el.addEventListener('sui-toggle', (e) => events.push(e as CustomEvent))
    el.setAttribute('open', '')
    await flush()
    const opened = events.find((e) => (e.detail as { open: boolean }).open === true)!
    expect(opened).toBeDefined()
    expect(opened.detail).toEqual({ open: true })
    expect(opened.bubbles).toBe(true)
    expect(opened.composed).toBe(true)
    expect(triggerOf(el).getAttribute('aria-expanded')).toBe('true')
  })

  it('show()/hide() public methods toggle open state + emit sui-toggle', async () => {
    const { el } = makePopover()
    await flush()
    const events: boolean[] = []
    el.addEventListener('sui-toggle', (e) => events.push((e as CustomEvent).detail.open))
    ;(el as unknown as PopoverApi).show()
    await flush()
    expect(events[events.length - 1]).toBe(true)
    expect(triggerOf(el).getAttribute('aria-expanded')).toBe('true')
    ;(el as unknown as PopoverApi).hide()
    await flush()
    expect(events[events.length - 1]).toBe(false)
    expect(triggerOf(el).getAttribute('aria-expanded')).toBe('false')
  })

  it('reflects placement on the host attribute', () => {
    const { el } = makePopover({ placement: 'top-end' })
    expect(el.getAttribute('placement')).toBe('top-end')
  })

  it('disconnect — no error', () => {
    const { container } = makePopover()
    expect(container.isConnected).toBe(true)
    container.remove()
    expect(container.isConnected).toBe(false)
  })
})
