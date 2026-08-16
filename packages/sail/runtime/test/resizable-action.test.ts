// use:resizable — L1 <resizable> handle. Stamps separator ARIA and
// writes flex-basis % on the start pane (previous sibling).

import { afterEach, describe, expect, it, vi } from 'vitest'
import { resizable } from '../src/actions/resizable'

function rect(left: number, top: number, width: number, height: number): DOMRect {
  return {
    x: left,
    y: top,
    left,
    top,
    width,
    height,
    right: left + width,
    bottom: top + height,
    toJSON: () => ({}),
  } as DOMRect
}

function firePointer(
  target: EventTarget,
  type: string,
  init: { clientX: number; clientY: number; button?: number },
): void {
  target.dispatchEvent(
    new MouseEvent(type, {
      bubbles: true,
      cancelable: true,
      clientX: init.clientX,
      clientY: init.clientY,
      button: init.button ?? 0,
    }),
  )
}

function mountSplit(orientation: 'horizontal' | 'vertical' = 'horizontal'): {
  host: HTMLElement
  start: HTMLElement
  handle: HTMLElement
  end: HTMLElement
} {
  const host = document.createElement('div')
  host.setAttribute('data-sail-host', 'resizable')
  if (orientation === 'vertical') host.setAttribute('orientation', 'vertical')
  const start = document.createElement('div')
  const handle = document.createElement('button')
  handle.type = 'button'
  const end = document.createElement('div')
  host.append(start, handle, end)
  document.body.append(host)

  vi.spyOn(host, 'getBoundingClientRect').mockReturnValue(
    orientation === 'horizontal' ? rect(0, 0, 400, 200) : rect(0, 0, 200, 400),
  )
  vi.spyOn(start, 'getBoundingClientRect').mockReturnValue(
    orientation === 'horizontal' ? rect(0, 0, 194, 200) : rect(0, 0, 200, 194),
  )
  vi.spyOn(handle, 'getBoundingClientRect').mockReturnValue(
    orientation === 'horizontal' ? rect(194, 0, 12, 200) : rect(0, 194, 200, 12),
  )
  vi.spyOn(end, 'getBoundingClientRect').mockReturnValue(
    orientation === 'horizontal' ? rect(206, 0, 194, 200) : rect(0, 206, 200, 194),
  )

  return { host, start, handle, end }
}

describe('use:resizable', () => {
  afterEach(() => {
    vi.restoreAllMocks()
    document.body.replaceChildren()
  })

  it('stamps role=separator and aria-orientation from the host', () => {
    const { handle } = mountSplit()
    resizable(handle, undefined as void)
    expect(handle.getAttribute('role')).toBe('separator')
    expect(handle.getAttribute('orientation')).toBe('horizontal')
    expect(handle.getAttribute('aria-orientation')).toBe('horizontal')
    expect(handle.getAttribute('aria-valuemin')).toBe('0')
    expect(handle.getAttribute('aria-valuemax')).toBe('100')
  })

  it('restamps aria-orientation when the host orientation attr flips', async () => {
    const { host, handle } = mountSplit()
    resizable(handle, undefined as void)
    host.setAttribute('orientation', 'vertical')
    await Promise.resolve()
    expect(handle.getAttribute('aria-orientation')).toBe('vertical')
  })

  it('pointer drag sets flex-basis % on the start pane', () => {
    const { start, handle } = mountSplit()
    resizable(handle, undefined as void)
    firePointer(handle, 'pointerdown', { clientX: 200, clientY: 100 })
    firePointer(window, 'pointermove', { clientX: 120, clientY: 100 })
    expect(start.style.flexGrow).toBe('0')
    expect(start.style.flexShrink).toBe('0')
    expect(start.style.flexBasis).toMatch(/%$/)
    const pct = parseFloat(start.style.flexBasis)
    expect(pct).toBeGreaterThan(0)
    expect(pct).toBeLessThan(50)
    expect(handle.getAttribute('aria-valuenow')).toBe(String(Math.round(pct)))
  })

  it('arrow keys nudge the start pane', () => {
    const { start, handle } = mountSplit()
    resizable(handle, undefined as void)
    handle.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true }))
    const right = parseFloat(start.style.flexBasis)
    expect(right).toBeGreaterThan(48)
    handle.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowLeft', bubbles: true }))
    const left = parseFloat(start.style.flexBasis)
    expect(left).toBeLessThan(right)
  })

  it('vertical arrows follow orientation', () => {
    const { start, handle } = mountSplit('vertical')
    resizable(handle, undefined as void)
    handle.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))
    expect(parseFloat(start.style.flexBasis)).toBeGreaterThan(48)
  })

  it('destroy tears down listeners', () => {
    const { start, handle } = mountSplit()
    const result = resizable(handle, undefined as void)
    result?.destroy?.()
    firePointer(handle, 'pointerdown', { clientX: 200, clientY: 100 })
    firePointer(window, 'pointermove', { clientX: 80, clientY: 100 })
    expect(start.style.flexBasis).toBe('')
  })
})
