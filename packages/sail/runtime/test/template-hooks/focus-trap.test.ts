import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest'
import { useFocusTrapHook } from '../../src/template-hooks/focus-trap'
import type { TemplateHookContext } from '../../src/template-hook'

function makeCtx(
  attrs: Record<string, unknown>,
  el: HTMLElement = document.body,
): { ctx: TemplateHookContext<Record<string, unknown>>; cleanups: (() => void)[] } {
  const cleanups: (() => void)[] = []
  const ctx: TemplateHookContext<Record<string, unknown>> = {
    attrs,
    el,
    cleanup: (fn) => cleanups.push(fn),
  }
  return { ctx, cleanups }
}

function runCleanups(cleanups: (() => void)[]): void {
  for (const fn of cleanups) fn()
}

describe('useFocusTrapHook', () => {
  let container: HTMLElement

  beforeEach(() => {
    container = document.createElement('div')
    document.body.appendChild(container)
  })

  afterEach(() => {
    container.remove()
  })

  it('registers under name "focus-trap"', () => {
    expect(useFocusTrapHook.name).toBe('focus-trap')
  })

  it('does nothing when target is not provided', () => {
    const { ctx, cleanups } = makeCtx({ target: undefined })
    expect(() => useFocusTrapHook.factory(ctx as never)).not.toThrow()
    expect(cleanups).toHaveLength(0)
  })

  it('focuses the first focusable element on mount', () => {
    const btn1 = document.createElement('button')
    const btn2 = document.createElement('button')
    container.appendChild(btn1)
    container.appendChild(btn2)

    const { ctx, cleanups } = makeCtx({ target: container })
    useFocusTrapHook.factory(ctx as never)

    expect(document.activeElement).toBe(btn1)
    runCleanups(cleanups)
  })

  it('wraps forward Tab from last to first focusable', () => {
    const btn1 = document.createElement('button')
    const btn2 = document.createElement('button')
    container.appendChild(btn1)
    container.appendChild(btn2)

    const { ctx, cleanups } = makeCtx({ target: container })
    useFocusTrapHook.factory(ctx as never)

    btn2.focus()
    const ev = new KeyboardEvent('keydown', { key: 'Tab', bubbles: true, cancelable: true })
    document.dispatchEvent(ev)

    expect(ev.defaultPrevented).toBe(true)
    expect(document.activeElement).toBe(btn1)
    runCleanups(cleanups)
  })

  it('wraps backward Shift+Tab from first to last focusable', () => {
    const btn1 = document.createElement('button')
    const btn2 = document.createElement('button')
    container.appendChild(btn1)
    container.appendChild(btn2)

    const { ctx, cleanups } = makeCtx({ target: container })
    useFocusTrapHook.factory(ctx as never)

    btn1.focus()
    const ev = new KeyboardEvent('keydown', {
      key: 'Tab',
      shiftKey: true,
      bubbles: true,
      cancelable: true,
    })
    document.dispatchEvent(ev)

    expect(ev.defaultPrevented).toBe(true)
    expect(document.activeElement).toBe(btn2)
    runCleanups(cleanups)
  })

  it('ignores non-Tab keys', () => {
    const btn = document.createElement('button')
    container.appendChild(btn)

    const { ctx, cleanups } = makeCtx({ target: container })
    useFocusTrapHook.factory(ctx as never)

    const ev = new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true })
    document.dispatchEvent(ev)
    expect(ev.defaultPrevented).toBe(false)
    runCleanups(cleanups)
  })

  it('removes the keydown listener on cleanup', () => {
    const btn = document.createElement('button')
    container.appendChild(btn)

    const { ctx, cleanups } = makeCtx({ target: container })
    useFocusTrapHook.factory(ctx as never)
    runCleanups(cleanups)

    btn.focus()
    const spy = vi.spyOn(btn, 'focus')
    const ev = new KeyboardEvent('keydown', { key: 'Tab', bubbles: true, cancelable: true })
    document.dispatchEvent(ev)
    expect(spy).not.toHaveBeenCalled()
    spy.mockRestore()
  })
})
