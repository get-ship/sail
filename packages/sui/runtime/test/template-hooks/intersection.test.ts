import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest'
import { useIntersectionHook } from '../../src/template-hooks/intersection'
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

// Stub for IntersectionObserver (not available in jsdom).
class StubIntersectionObserver {
  private callback: IntersectionObserverCallback
  readonly root: Element | null = null
  readonly rootMargin: string = ''
  readonly thresholds: ReadonlyArray<number> = []

  static instances: StubIntersectionObserver[] = []
  observedElements: Element[] = []
  disconnected = false

  constructor(cb: IntersectionObserverCallback) {
    this.callback = cb
    StubIntersectionObserver.instances.push(this)
  }

  observe(el: Element): void {
    this.observedElements.push(el)
  }

  disconnect(): void {
    this.disconnected = true
  }

  unobserve(_el: Element): void {}
  takeRecords(): IntersectionObserverEntry[] { return [] }

  // Test helper — fire the callback with synthetic entries.
  trigger(entries: Partial<IntersectionObserverEntry>[]): void {
    this.callback(entries as IntersectionObserverEntry[], this)
  }
}

describe('useIntersectionHook', () => {
  let origIO: typeof IntersectionObserver | undefined

  beforeEach(() => {
    origIO =
      typeof IntersectionObserver !== 'undefined'
        ? IntersectionObserver
        : undefined
    StubIntersectionObserver.instances = []
    // @ts-expect-error — installing stub
    globalThis.IntersectionObserver = StubIntersectionObserver
  })

  afterEach(() => {
    if (origIO) {
      globalThis.IntersectionObserver = origIO
    } else {
      // @ts-expect-error — removing stub
      delete globalThis.IntersectionObserver
    }
  })

  it('registers under name "intersection"', () => {
    expect(useIntersectionHook.name).toBe('intersection')
  })

  it('does nothing when on callback is not provided', () => {
    const el = document.createElement('div')
    const { ctx, cleanups } = makeCtx(el, { on: undefined })
    expect(() => useIntersectionHook.factory(ctx as never)).not.toThrow()
    expect(cleanups).toHaveLength(0)
  })

  it('creates an IntersectionObserver and observes the element', () => {
    const el = document.createElement('div')
    const on = vi.fn()
    const { ctx, cleanups } = makeCtx(el, { on })
    useIntersectionHook.factory(ctx as never)

    const [obs] = StubIntersectionObserver.instances
    expect(obs).toBeDefined()
    expect(obs!.observedElements).toContain(el)
    for (const fn of cleanups) fn()
  })

  it('fires the on callback when the element is intersecting', () => {
    const el = document.createElement('div')
    const on = vi.fn()
    const { ctx, cleanups } = makeCtx(el, { on })
    useIntersectionHook.factory(ctx as never)

    const [obs] = StubIntersectionObserver.instances
    obs!.trigger([{ isIntersecting: true }])

    expect(on).toHaveBeenCalledTimes(1)
    for (const fn of cleanups) fn()
  })

  it('does not fire the on callback when not intersecting', () => {
    const el = document.createElement('div')
    const on = vi.fn()
    const { ctx, cleanups } = makeCtx(el, { on })
    useIntersectionHook.factory(ctx as never)

    const [obs] = StubIntersectionObserver.instances
    obs!.trigger([{ isIntersecting: false }])

    expect(on).not.toHaveBeenCalled()
    for (const fn of cleanups) fn()
  })

  it('disconnects the observer on cleanup', () => {
    const el = document.createElement('div')
    const on = vi.fn()
    const { ctx, cleanups } = makeCtx(el, { on })
    useIntersectionHook.factory(ctx as never)

    const [obs] = StubIntersectionObserver.instances
    for (const fn of cleanups) fn()

    expect(obs!.disconnected).toBe(true)
  })

  it('skips setup when IntersectionObserver is unavailable', () => {
    // @ts-expect-error — removing IO
    delete globalThis.IntersectionObserver

    const el = document.createElement('div')
    const on = vi.fn()
    const { ctx, cleanups } = makeCtx(el, { on })
    expect(() => useIntersectionHook.factory(ctx as never)).not.toThrow()
    expect(cleanups).toHaveLength(0)

    // @ts-expect-error — restore
    globalThis.IntersectionObserver = StubIntersectionObserver
  })
})
