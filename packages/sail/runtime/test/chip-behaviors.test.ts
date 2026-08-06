// Chip behaviors — pressable / pressed / dismissible `use:` actions.
//
// Ports the chip-controller parity cases onto the three stacked actions the
// L1 `<chip>` will stamp instead of an element controller. Each action is
// attached directly (the `runAction` lifecycle is covered elsewhere); the
// composed test at the bottom asserts the stack reproduces the controller's
// observable behavior.
//
// MutationObserver callbacks are async in jsdom — attribute-flip assertions
// flush microtasks (`await Promise.resolve()`), same as chip-controller.test.
import { describe, expect, it, afterEach } from 'vitest'
import { pressable } from '../src/actions/pressable'
import { pressed } from '../src/actions/pressed'
import { dismissible } from '../src/actions/dismissible'

function makeHost(attrs: Record<string, string | boolean> = {}): HTMLElement {
  const el = document.createElement('span')
  el.className = 'sail-chip'
  for (const [k, v] of Object.entries(attrs)) {
    if (v === false) continue
    if (v === true) el.setAttribute(k, '')
    else el.setAttribute(k, v)
  }
  el.append('Label')
  const remove = document.createElement('button')
  remove.type = 'button'
  remove.setAttribute('aria-label', 'Remove')
  remove.tabIndex = -1
  el.append(remove)
  document.body.append(el)
  return el
}

describe('chip behaviors (use: actions)', () => {
  afterEach(() => {
    document.body.replaceChildren()
  })

  it('pressable: sets role=button and tabindex on a plain span', () => {
    const el = makeHost()
    pressable(el)
    expect(el.getAttribute('role')).toBe('button')
    expect(el.getAttribute('tabindex')).toBe('0')
    expect(el.hasAttribute('aria-disabled')).toBe(false)
    // pressable is not pressed — no aria-pressed from this action
    expect(el.hasAttribute('aria-pressed')).toBe(false)
  })

  it('pressable: disabled → aria-disabled + tabindex -1, observer flips back', async () => {
    const el = makeHost({ disabled: true })
    pressable(el)
    expect(el.getAttribute('aria-disabled')).toBe('true')
    expect(el.getAttribute('tabindex')).toBe('-1')
    el.removeAttribute('disabled')
    await Promise.resolve()
    expect(el.hasAttribute('aria-disabled')).toBe(false)
    expect(el.getAttribute('tabindex')).toBe('0')
    el.setAttribute('disabled', '')
    await Promise.resolve()
    expect(el.getAttribute('aria-disabled')).toBe('true')
    expect(el.getAttribute('tabindex')).toBe('-1')
  })

  it('pressable: Enter and Space each fire exactly one click; disabled suppresses', () => {
    const el = makeHost()
    pressable(el)
    let clicks = 0
    el.addEventListener('click', () => {
      clicks++
    })
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    el.dispatchEvent(new KeyboardEvent('keydown', { key: ' ', bubbles: true }))
    expect(clicks).toBe(2)
    el.setAttribute('disabled', '')
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    el.dispatchEvent(new KeyboardEvent('keydown', { key: ' ', bubbles: true }))
    expect(clicks).toBe(2)
  })

  it('pressed: initial aria-pressed from [selected]; observer syncs on toggle', async () => {
    const on = makeHost({ selected: true })
    pressed(on)
    expect(on.getAttribute('aria-pressed')).toBe('true')

    const off = makeHost()
    pressed(off)
    expect(off.getAttribute('aria-pressed')).toBe('false')

    off.setAttribute('selected', '')
    await Promise.resolve()
    expect(off.getAttribute('aria-pressed')).toBe('true')
    off.removeAttribute('selected')
    await Promise.resolve()
    expect(off.getAttribute('aria-pressed')).toBe('false')
  })

  it('dismissible: Backspace/Delete with [removable] invokes onRemove; without it does not fire', () => {
    const el = makeHost({ removable: true }) as HTMLElement & { onRemove?: () => void }
    let calls = 0
    el.onRemove = () => {
      calls++
    }
    dismissible(el)

    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Backspace', bubbles: true }))
    expect(calls).toBe(1)
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Delete', bubbles: true }))
    expect(calls).toBe(2)

    const plain = makeHost() as HTMLElement & { onRemove?: () => void }
    let none = 0
    plain.onRemove = () => {
      none++
    }
    dismissible(plain)
    plain.dispatchEvent(new KeyboardEvent('keydown', { key: 'Backspace', bubbles: true }))
    plain.dispatchEvent(new KeyboardEvent('keydown', { key: 'Delete', bubbles: true }))
    expect(none).toBe(0)
  })

  it('dismissible: Remove button click invokes onRemove and stops propagation to the host', () => {
    const el = makeHost({ removable: true })
    dismissible(el)
    let removes = 0
    let hostClicks = 0
    ;(el as any).onRemove = () => { removes++ }
    el.addEventListener('click', () => {
      hostClicks++
    })
    const btn = el.querySelector('button[aria-label="Remove"]') as HTMLElement
    btn.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    expect(removes).toBe(1)
    expect(hostClicks).toBe(0)
  })

  it('dismissible: disabled suppresses both paths', () => {
    const el = makeHost({ removable: true, disabled: true })
    dismissible(el)
    let removes = 0
    ;(el as any).onRemove = () => { removes++ }
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Backspace', bubbles: true }))
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Delete', bubbles: true }))
    const btn = el.querySelector('button[aria-label="Remove"]') as HTMLElement
    btn.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    expect(removes).toBe(0)
  })

  it('destroy: keys/clicks are inert and observers disconnected', async () => {
    const el = makeHost({ removable: true, selected: true })
    const pressableResult = pressable(el)
    const pressedResult = pressed(el)
    const dismissibleResult = dismissible(el)

    let clicks = 0
    let removes = 0
    el.addEventListener('click', () => {
      clicks++
    })
    ;(el as HTMLElement & { onRemove?: () => void }).onRemove = () => {
      removes++
    }

    pressableResult?.destroy?.()
    pressedResult?.destroy?.()
    dismissibleResult?.destroy?.()

    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Delete', bubbles: true }))
    expect(clicks).toBe(0)
    expect(removes).toBe(0)
    // Remove button click no longer invokes onRemove; with the action's
    // stopPropagation gone the click bubbles to the host (platform behavior).
    const btn = el.querySelector('button[aria-label="Remove"]') as HTMLElement
    btn.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    expect(removes).toBe(0)
    expect(clicks).toBe(1)

    // Observers disconnected: attribute flips no longer sync ARIA.
    el.removeAttribute('selected')
    el.setAttribute('disabled', '')
    await Promise.resolve()
    expect(el.getAttribute('aria-pressed')).toBe('true')
    expect(el.hasAttribute('aria-disabled')).toBe(false)
  })

  it('composed: one span with all three actions behaves like the chip controller', async () => {
    const el = makeHost({ removable: true, selected: true })
    const results = [pressable(el), pressed(el), dismissible(el)]

    // Static parity: role, tabindex, aria-pressed, no aria-disabled.
    expect(el.getAttribute('role')).toBe('button')
    expect(el.getAttribute('tabindex')).toBe('0')
    expect(el.getAttribute('aria-pressed')).toBe('true')
    expect(el.hasAttribute('aria-disabled')).toBe(false)

    // Keyboard activation.
    let clicks = 0
    el.addEventListener('click', () => {
      clicks++
    })
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    expect(clicks).toBe(1)

    // Dismiss via keyboard and via Remove button (with stopPropagation).
    let removes = 0
    ;(el as any).onRemove = () => { removes++ }
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Delete', bubbles: true }))
    expect(removes).toBe(1)
    const btn = el.querySelector('button[aria-label="Remove"]') as HTMLElement
    btn.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    expect(removes).toBe(2)
    expect(clicks).toBe(1)

    // Toggle reflection + disabled gating, controller-style.
    el.removeAttribute('selected')
    await Promise.resolve()
    expect(el.getAttribute('aria-pressed')).toBe('false')
    el.setAttribute('disabled', '')
    await Promise.resolve()
    expect(el.getAttribute('aria-disabled')).toBe('true')
    expect(el.getAttribute('tabindex')).toBe('-1')
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Backspace', bubbles: true }))
    expect(clicks).toBe(1)
    expect(removes).toBe(2)

    for (const r of results) r?.destroy?.()
  })
})
