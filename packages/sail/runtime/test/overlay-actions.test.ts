// Overlay family: live trigger bind (hover).
// Catalog-shaped markup: trigger sibling of panel; panel action may mount first.
import { afterEach, describe, expect, it, vi } from 'vitest'
import { hoverSurface } from '../src/actions/hover-surface'
import { initJsdomPolyfills } from '../../test-setup/src/jsdom-polyfills'

initJsdomPolyfills()

afterEach(() => {
  document.body.replaceChildren()
  vi.useRealTimers()
})

/** Drain MutationObserver + the action's rAF-coalesced rebind. */
async function flushMo(): Promise<void> {
  await Promise.resolve()
  await Promise.resolve()
  if (vi.isFakeTimers()) {
    await vi.advanceTimersByTimeAsync(0)
    await vi.advanceTimersByTimeAsync(16)
  } else {
    await new Promise<void>((r) => requestAnimationFrame(() => requestAnimationFrame(() => r())))
  }
}

function pointerEnter(el: EventTarget): void {
  el.dispatchEvent(new PointerEvent('pointerenter', { bubbles: true }))
}

describe('use:hoverSurface — live trigger bind', () => {
  it('opens on pointerenter after a late-mounted invoker (catalog mount order)', async () => {
    vi.useFakeTimers()
    const card = document.createElement('div')
    card.id = 'grid-hc'
    card.setAttribute('popover', '')
    card.setAttribute('open-delay', '0')
    card.setAttribute('close-delay', '0')
    document.body.append(card)

    const handle = hoverSurface(card) as { destroy: () => void }
    // Panel mounted first — no trigger yet (one-shot would stay dead).
    const show = vi.fn()
    ;(card as HTMLElement & { showPopover: () => void }).showPopover = show

    const trig = document.createElement('button')
    trig.setAttribute('popovertarget', 'grid-hc')
    trig.textContent = '@ada'
    document.body.append(trig)
    await flushMo()

    pointerEnter(trig)
    vi.runAllTimers()
    expect(show).toHaveBeenCalled()

    handle.destroy()
  })

  it('rebinds when popovertarget appears on an existing node', async () => {
    vi.useFakeTimers()
    const card = document.createElement('div')
    card.id = 'hc2'
    card.setAttribute('open-delay', '0')
    document.body.append(card)
    const show = vi.fn()
    ;(card as HTMLElement & { showPopover: () => void }).showPopover = show

    const handle = hoverSurface(card) as { destroy: () => void }
    const trig = document.createElement('span')
    document.body.append(trig)
    await flushMo()

    pointerEnter(trig)
    vi.runAllTimers()
    expect(show).not.toHaveBeenCalled()

    trig.setAttribute('popovertarget', 'hc2')
    await flushMo()
    pointerEnter(trig)
    vi.runAllTimers()
    expect(show).toHaveBeenCalled()

    handle.destroy()
  })
})
