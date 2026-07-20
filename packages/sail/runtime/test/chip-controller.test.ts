// L1 chip element-controller parity — keyboard, aria-pressed, remove.
// Behavior moved from use:chip into element-controllers/chip; attach via
// data-sail-el discovery (attachElementControllers).
import { describe, expect, it, beforeEach, afterEach } from 'vitest'
import {
  attachElementControllers,
  chip as chipController,
  __resetElementControllerAttach,
} from '../src/element-controllers'

function makeChip(attrs: Record<string, string | boolean> = {}): HTMLElement {
  const el = document.createElement('span')
  el.className = 'sail-chip'
  el.setAttribute('data-sail-el', 'chip')
  for (const [k, v] of Object.entries(attrs)) {
    if (v === false) continue
    if (v === true) el.setAttribute(k, '')
    else el.setAttribute(k, v)
  }
  el.append('Label')
  const remove = document.createElement('button')
  remove.className = 'remove'
  remove.type = 'button'
  remove.setAttribute('aria-label', 'Remove')
  remove.tabIndex = -1
  el.append(remove)
  document.body.append(el)
  return el
}

describe('chip element controller', () => {
  afterEach(() => {
    document.body.replaceChildren()
  })

  it('sets role=button, aria-pressed, tabindex', () => {
    const el = makeChip({ selected: true })
    const dispose = attachElementControllers(document.body)
    expect(el.getAttribute('role')).toBe('button')
    expect(el.getAttribute('aria-pressed')).toBe('true')
    expect(el.getAttribute('tabindex')).toBe('0')
    dispose()
  })

  it('disables via aria-disabled and tabindex -1', () => {
    const el = makeChip({ disabled: true, selected: true })
    attachElementControllers(document.body)
    expect(el.getAttribute('aria-disabled')).toBe('true')
    expect(el.getAttribute('tabindex')).toBe('-1')
    expect(el.getAttribute('aria-pressed')).toBe('true')
  })

  it('syncs aria-pressed when selected toggles', () => {
    const el = makeChip()
    attachElementControllers(document.body)
    expect(el.getAttribute('aria-pressed')).toBe('false')
    el.setAttribute('selected', '')
    // MutationObserver is async in jsdom — flush microtasks
    return Promise.resolve().then(() => {
      expect(el.getAttribute('aria-pressed')).toBe('true')
      el.removeAttribute('selected')
      return Promise.resolve().then(() => {
        expect(el.getAttribute('aria-pressed')).toBe('false')
      })
    })
  })

  it('Enter/Space activate host.click', () => {
    const el = makeChip()
    attachElementControllers(document.body)
    let clicks = 0
    el.addEventListener('click', () => {
      clicks++
    })
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    el.dispatchEvent(new KeyboardEvent('keydown', { key: ' ', bubbles: true }))
    expect(clicks).toBe(2)
  })

  it('Backspace/Delete on removable fires sail-remove', () => {
    const el = makeChip({ removable: true })
    attachElementControllers(document.body)
    const events: Event[] = []
    el.addEventListener('sail-remove', (e) => events.push(e))
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Backspace', bubbles: true }))
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Delete', bubbles: true }))
    expect(events).toHaveLength(2)
  })

  it('remove button click fires sail-remove and stops activate', () => {
    const el = makeChip({ removable: true })
    attachElementControllers(document.body)
    const removes: Event[] = []
    let clicks = 0
    el.addEventListener('sail-remove', (e) => removes.push(e))
    el.addEventListener('click', () => {
      clicks++
    })
    const btn = el.querySelector('.remove') as HTMLElement
    btn.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    expect(removes).toHaveLength(1)
    // stopPropagation on remove click — host click listener still sees
    // target-phase if listening on host; the action stops bubbling of the
    // button's click so outer hosts don't treat remove as activate. Host
    // listener on the chip itself still receives the event in bubble phase
    // from the button unless stopped — controller stops it.
    expect(clicks).toBe(0)
  })

  it('attach is idempotent (no double-binding)', () => {
    const el = makeChip({ removable: true })
    const d1 = attachElementControllers(document.body)
    const d2 = attachElementControllers(document.body)
    const events: Event[] = []
    el.addEventListener('sail-remove', (e) => events.push(e))
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Delete', bubbles: true }))
    expect(events).toHaveLength(1)
    d1()
    d2()
  })

  it('direct controller matches attach path', () => {
    const el = makeChip({ selected: true })
    __resetElementControllerAttach(el)
    chipController(el)
    expect(el.getAttribute('role')).toBe('button')
    expect(el.getAttribute('aria-pressed')).toBe('true')
  })
})
