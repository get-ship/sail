// use:anchor — the ONE positioner for the popover family. jsdom has no layout
// (getBoundingClientRect returns zeros), so we stub the trigger rect, the
// panel's offset box, and the viewport, then assert the fixed top/left the
// action computes for every placement + the block/inline flip fallbacks.

import { afterEach, beforeAll, describe, expect, it } from 'vitest'
import { anchor, type AnchorPlacement } from '../src/index'

const VW = 1000
const VH = 800

beforeAll(() => {
  Object.defineProperty(window, 'innerWidth', { value: VW, configurable: true })
  Object.defineProperty(window, 'innerHeight', { value: VH, configurable: true })
})

type Rect = { left: number; top: number; right: number; bottom: number }

function harness(triggerRect: Rect, panel = { w: 120, h: 80 }) {
  const el = document.createElement('div')
  el.id = 'pop'
  document.body.appendChild(el)
  Object.defineProperty(el, 'offsetWidth', { value: panel.w, configurable: true })
  Object.defineProperty(el, 'offsetHeight', { value: panel.h, configurable: true })
  // Pretend it's always in the top layer so reposition() runs.
  el.matches = ((s: string) => s === ':popover-open') as typeof el.matches

  const trig = document.createElement('button')
  trig.setAttribute('popovertarget', 'pop')
  document.body.appendChild(trig)
  trig.getBoundingClientRect = () =>
    ({ ...triggerRect, width: triggerRect.right - triggerRect.left, height: triggerRect.bottom - triggerRect.top, x: triggerRect.left, y: triggerRect.top, toJSON() {} }) as DOMRect

  const place = (p: AnchorPlacement) => {
    const h = anchor(el, p) as { update: (p: AnchorPlacement) => void; destroy: () => void }
    h.update(p) // synchronous reposition (initial attach only wires listeners)
    const out = { top: el.style.top, left: el.style.left }
    h.destroy()
    return out
  }
  return { el, place }
}

afterEach(() => {
  document.querySelectorAll('div#pop, button[popovertarget]').forEach((n) => n.remove())
})

describe('use:anchor placement', () => {
  // Roomy trigger dead-center: no clamp, no flip. 100×40 box at (400,300).
  const center: Rect = { left: 400, top: 300, right: 500, bottom: 340 }

  it('bottom — below, left-aligned', () => {
    expect(harness(center).place('bottom')).toEqual({ top: '344px', left: '400px' })
  })
  it('bottom-end — below, right-aligned', () => {
    expect(harness(center).place('bottom-end')).toEqual({ top: '344px', left: '380px' })
  })
  it('top — above, left-aligned', () => {
    expect(harness(center).place('top')).toEqual({ top: '216px', left: '400px' })
  })
  it('top-end — above, right-aligned', () => {
    expect(harness(center).place('top-end')).toEqual({ top: '216px', left: '380px' })
  })
  it('right — beside, top-aligned', () => {
    expect(harness(center).place('right')).toEqual({ top: '300px', left: '504px' })
  })
  it('right-end — beside, bottom-aligned', () => {
    expect(harness(center).place('right-end')).toEqual({ top: '260px', left: '504px' })
  })
  it('left — beside, top-aligned', () => {
    expect(harness(center).place('left')).toEqual({ top: '300px', left: '276px' })
  })
  it('left-end — beside, bottom-aligned', () => {
    expect(harness(center).place('left-end')).toEqual({ top: '260px', left: '276px' })
  })
})

describe('use:anchor flip fallbacks', () => {
  it('bottom flips up when it would overflow the viewport bottom', () => {
    // Trigger near the bottom edge: below (740+40+4+80=864 > 800) overflows,
    // room above (740-80-4=656 ≥ 0) → flip up.
    const near: Rect = { left: 400, top: 740, right: 500, bottom: 780 }
    expect(harness(near).place('bottom')).toEqual({ top: '656px', left: '400px' })
  })

  it('right flips left when it would overflow the viewport right', () => {
    // Trigger near the right edge: right (960+4+120=1084 > 1000) overflows,
    // room left (900-120-4=776 ≥ 0) → flip left.
    const near: Rect = { left: 900, top: 300, right: 960, bottom: 340 }
    expect(harness(near).place('right')).toEqual({ top: '300px', left: '776px' })
  })
})
