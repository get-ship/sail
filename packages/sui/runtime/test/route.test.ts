import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { effect, signal } from '../src/signal'
import { live } from '../src/live'
import { defineRoutes, navigate, route, setRouterPort } from '../src/route'

// Reset the global popstate / updater state before each test by
// poking at the module-level cache via a fresh navigate(). The tests
// keep all URL state inside the page's history, so push history back
// to a known root before every case.
beforeEach(() => {
  window.history.replaceState({}, '', '/')
})

afterEach(() => {
  window.history.replaceState({}, '', '/')
  // Clear any router port a test installed so it doesn't leak into others.
  ;(globalThis as { __shipRouterPort?: unknown }).__shipRouterPort = undefined
})

const tick = () => new Promise((r) => setTimeout(r, 0))

describe('route()', () => {
  it('seeds pathname / hash from the supplied URL', () => {
    const r = route({ url: new URL('http://x/agents/42#top') })
    expect(r.pathname()).toBe('/agents/42')
    expect(r.hash()).toBe('#top')
  })

  it('returns a Signal for params.<name>; value is undefined when no shape matches', () => {
    const r = route({ url: new URL('http://x/agents/42') })
    const idSig = r.params.id
    expect(typeof idSig).toBe('function')
    expect(idSig()).toBeUndefined()
  })

  it('matches a registered shape and returns the captured param', () => {
    defineRoutes([{ pattern: '/agents/:id' }])
    const r = route({ url: new URL('http://x/agents/42') })
    expect(r.params.id()).toBe('42')
  })

  it('exposes query params as signals seeded from URLSearchParams', () => {
    const r = route({ url: new URL('http://x/?q=foo&page=2') })
    expect(r.query.q()).toBe('foo')
    expect(r.query.page()).toBe('2')
    expect(r.query.missing()).toBeUndefined()
  })

  it('writing to params or query throws (URL is the source of truth)', () => {
    const r = route({ url: new URL('http://x/') })
    expect(() => {
      // @ts-expect-error intentional misuse for the test
      r.params.id = 'nope'
    }).toThrow(/read-only/)
    expect(() => {
      // @ts-expect-error intentional misuse for the test
      delete r.query.q
    }).toThrow(/cannot delete/)
  })

  it('JSON.stringify(params) emits a flat record of currently-known keys', () => {
    defineRoutes([{ pattern: '/agents/:id' }])
    const r = route({ url: new URL('http://x/agents/42?q=foo') })
    // Reading first materialises the keys.
    void r.params.id()
    void r.query.q()
    expect(JSON.parse(JSON.stringify(r.params))).toEqual({ id: '42' })
    expect(JSON.parse(JSON.stringify(r.query))).toEqual({ q: 'foo' })
  })

  it('reading params.id inside an effect tracks the signal', () => {
    defineRoutes([{ pattern: '/agents/:id' }])
    window.history.replaceState({}, '', '/agents/1')
    const r = route()
    const seen: Array<string | undefined> = []
    const dispose = effect(() => {
      seen.push(r.params.id())
    })
    expect(seen).toEqual(['1'])
    navigate('/agents/2')
    expect(seen).toEqual(['1', '2'])
    dispose()
  })

  it('navigate() updates pathname and lazily-created query signals', () => {
    const r = route()
    const seen: string[] = []
    const dispose = effect(() => {
      seen.push(r.pathname())
    })
    expect(seen).toEqual(['/'])
    navigate('/foo?q=bar')
    expect(seen).toEqual(['/', '/foo'])
    expect(r.query.q()).toBe('bar')
    dispose()
  })

  it('popstate triggers the global updater', () => {
    const r = route()
    window.history.pushState({}, '', '/before')
    // popstate doesn't fire on pushState — synthesise one after a
    // direct history mutation so the listener installed by route()
    // sees it.
    window.history.replaceState({}, '', '/after')
    window.dispatchEvent(new PopStateEvent('popstate'))
    expect(r.pathname()).toBe('/after')
  })

  it('live(source) re-fires when route.params.<name> changes', async () => {
    defineRoutes([{ pattern: '/agents/:id' }])
    window.history.replaceState({}, '', '/agents/1')
    const r = route()
    const calls: Array<string | undefined> = []
    const sig = live(() => {
      const id = r.params.id()
      calls.push(id)
      return Promise.resolve(`agent:${id}`)
    })
    await tick()
    await tick()
    expect(calls).toEqual(['1'])
    const v1 = sig()
    expect(v1.kind).toBe('ok')
    if (v1.kind === 'ok') expect(v1.value).toBe('agent:1')

    navigate('/agents/2')
    expect(sig().kind).toBe('loading')
    await tick()
    await tick()
    expect(calls).toEqual(['1', '2'])
    const v2 = sig()
    expect(v2.kind).toBe('ok')
    if (v2.kind === 'ok') expect(v2.value).toBe('agent:2')
  })

  it('defineRoutes after route() construction repopulates already-read params', () => {
    window.history.replaceState({}, '', '/widgets/abc')
    const r = route()
    // No shape registered yet — id is undefined.
    expect(r.params.id()).toBeUndefined()
    defineRoutes([{ pattern: '/widgets/:id' }])
    // After registration, the global updater re-evaluates params.
    expect(r.params.id()).toBe('abc')
  })

  it('writes to other signals do not bleed into route', () => {
    const other = signal(0)
    const r = route()
    let pathseen = ''
    const dispose = effect(() => {
      pathseen = r.pathname()
    })
    other.set(1)
    expect(pathseen).toBe('/')
    dispose()
  })
})

describe('navigate() + RouterPort', () => {
  it('delegates to a registered host RouterPort instead of touching history', () => {
    const port = { navigate: vi.fn(), back: vi.fn() }
    setRouterPort(port)
    navigate('/agents/42')
    expect(port.navigate).toHaveBeenCalledWith('/agents/42', {})
    // The host owns history; navigate() must not push when a port is set.
    expect(window.location.pathname).toBe('/')
  })

  it('forwards replace option to the port', () => {
    const port = { navigate: vi.fn(), back: vi.fn() }
    setRouterPort(port)
    navigate('/login', { replace: true })
    expect(port.navigate).toHaveBeenCalledWith('/login', { replace: true })
  })

  it('falls back to in-place history + signal update when no port is registered', () => {
    // (afterEach clears the port; none registered here.)
    const r = route()
    let seen = ''
    const dispose = effect(() => {
      seen = r.pathname()
    })
    navigate('/foo')
    expect(window.location.pathname).toBe('/foo')
    expect(seen).toBe('/foo')
    dispose()
  })
})
