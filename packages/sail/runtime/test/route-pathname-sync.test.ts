// route.pathname same-tick SSOT (E2a): after history mutates, `update()` re-seeds
// pathname from the live location in the same tick — no frame lag, no history wrap.
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { navigate } from '../src/navigate'
import { routeWithUpdate } from '../src/route'

describe('route.pathname sync', () => {
  beforeEach(() => {
    history.replaceState({}, '', '/')
    ;(globalThis as { __shipRouterPort?: unknown }).__shipRouterPort = undefined
    ;(globalThis as { __shipSetRouteUpdate?: unknown }).__shipSetRouteUpdate = undefined
  })
  afterEach(() => {
    ;(globalThis as { __shipRouterPort?: unknown }).__shipRouterPort = undefined
    ;(globalThis as { __shipSetRouteUpdate?: unknown }).__shipSetRouteUpdate = undefined
  })

  it('update() re-reads location.pathname same-tick', () => {
    const r = routeWithUpdate()
    expect(r.pathname()).toBe('/')
    history.pushState({}, '', '/docs/guide/introduction')
    r.update()
    expect(r.pathname()).toBe('/docs/guide/introduction')
  })

  it('navigate no-port + wired updater updates pathname same-tick', () => {
    const r = routeWithUpdate()
    globalThis.__shipSetRouteUpdate = () => r.update()
    expect(r.pathname()).toBe('/')
    navigate('/components/button')
    // Same tick — no queueMicrotask / rAF lag.
    expect(r.pathname()).toBe('/components/button')
    expect(location.pathname).toBe('/components/button')
  })

  it('explicit opts.url stays frozen on update (test harness)', () => {
    const r = routeWithUpdate({ url: new URL('http://test.local/frozen') })
    expect(r.pathname()).toBe('/frozen')
    history.pushState({}, '', '/other')
    r.update()
    expect(r.pathname()).toBe('/frozen')
  })
})
