import { describe, expect, it, vi, afterEach } from 'vitest'
import { useScrollAnchorHook } from '../../src/template-hooks/scroll-anchor'
import type { TemplateHookContext } from '../../src/template-hook'

function makeCtx(
  el: HTMLElement,
  attrs: Record<string, unknown>,
): { ctx: TemplateHookContext<Record<string, unknown>>; cleanups: (() => void)[] } {
  const cleanups: (() => void)[] = []
  const ctx: TemplateHookContext<Record<string, unknown>> = {
    attrs,
    el,
    cleanup: (fn) => cleanups.push(fn),
  }
  return { ctx, cleanups }
}

describe('useScrollAnchorHook', () => {
  afterEach(() => {
    // Reset hash between tests
    window.history.replaceState({}, '', '/')
  })

  it('registers under name "scroll-anchor"', () => {
    expect(useScrollAnchorHook.name).toBe('scroll-anchor')
  })

  it('does nothing when id is not provided', () => {
    const el = document.createElement('div')
    const { ctx, cleanups } = makeCtx(el, { id: '' })
    expect(() => useScrollAnchorHook.factory(ctx as never)).not.toThrow()
    expect(cleanups).toHaveLength(0)
  })

  it('scrolls into view on mount when hash already matches', () => {
    window.history.replaceState({}, '', '#features')

    const el = document.createElement('div')
    const scrollIntoView = vi.fn()
    el.scrollIntoView = scrollIntoView

    const { ctx, cleanups } = makeCtx(el, { id: 'features' })
    useScrollAnchorHook.factory(ctx as never)

    expect(scrollIntoView).toHaveBeenCalledTimes(1)
    for (const fn of cleanups) fn()
  })

  it('does not scroll when hash does not match on mount', () => {
    window.history.replaceState({}, '', '#other')

    const el = document.createElement('div')
    const scrollIntoView = vi.fn()
    el.scrollIntoView = scrollIntoView

    const { ctx, cleanups } = makeCtx(el, { id: 'features' })
    useScrollAnchorHook.factory(ctx as never)

    expect(scrollIntoView).not.toHaveBeenCalled()
    for (const fn of cleanups) fn()
  })

  it('scrolls into view when hashchange matches', () => {
    window.history.replaceState({}, '', '/')

    const el = document.createElement('div')
    const scrollIntoView = vi.fn()
    el.scrollIntoView = scrollIntoView

    const { ctx, cleanups } = makeCtx(el, { id: 'features' })
    useScrollAnchorHook.factory(ctx as never)

    expect(scrollIntoView).not.toHaveBeenCalled()

    window.history.replaceState({}, '', '#features')
    window.dispatchEvent(new HashChangeEvent('hashchange'))

    expect(scrollIntoView).toHaveBeenCalledTimes(1)
    for (const fn of cleanups) fn()
  })

  it('ignores hashchange for a different hash', () => {
    const el = document.createElement('div')
    const scrollIntoView = vi.fn()
    el.scrollIntoView = scrollIntoView

    const { ctx, cleanups } = makeCtx(el, { id: 'features' })
    useScrollAnchorHook.factory(ctx as never)

    window.history.replaceState({}, '', '#other')
    window.dispatchEvent(new HashChangeEvent('hashchange'))

    expect(scrollIntoView).not.toHaveBeenCalled()
    for (const fn of cleanups) fn()
  })

  it('removes hashchange listener on cleanup', () => {
    const el = document.createElement('div')
    const scrollIntoView = vi.fn()
    el.scrollIntoView = scrollIntoView

    const { ctx, cleanups } = makeCtx(el, { id: 'features' })
    useScrollAnchorHook.factory(ctx as never)
    for (const fn of cleanups) fn()

    window.history.replaceState({}, '', '#features')
    window.dispatchEvent(new HashChangeEvent('hashchange'))

    expect(scrollIntoView).not.toHaveBeenCalled()
  })
})
