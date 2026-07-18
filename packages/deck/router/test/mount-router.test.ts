import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'

// Record every runtime.mount() call. Hoisted so the vi.mock factory (also
// hoisted) can close over it.
const { mountCalls } = vi.hoisted(() => ({
  mountCalls: [] as Array<{ target: unknown; name: unknown; opts: any }>,
}))

// Mock the runtime instead of importing it: the real @ship/sail-runtime pulls
// chrome.ts → @ship/deck-router, a cycle vitest can't resolve (pre-existing).
// A minimal signal + a recording mount is all the router needs.
// The mock calls opts.module.__mount(target) when present so layout-module
// stubs can simulate rendering (proves authored markup is rendered, not dropped).
vi.mock('@ship/sail-runtime', () => {
  const signal = <T>(initial: T) => {
    let v = initial
    const s = (() => v) as { (): T; set(n: T): void; peek(): T; __peek(): T }
    s.set = (n: T) => { v = n }
    s.peek = () => v
    s.__peek = () => v
    return s
  }
  const mount = vi.fn(async (target: unknown, name: unknown, opts: any) => {
    if (opts?.module?.__mount) {
      opts.module.__mount(target)
    }
    mountCalls.push({ target, name, opts })
    return { dispose: vi.fn() }
  })
  // Faithful re-implementation of the runtime's error-boundary helpers (same
  // Symbol key + DOM walk) so the router's default route boundary behaves
  // exactly as it does against the real runtime, without pulling the cycle.
  const BOUNDARY_KEY = Symbol.for('sail:error-boundary')
  const attachBoundary = (host: any, handle: unknown) => { host[BOUNDARY_KEY] = handle }
  const detachBoundary = (host: any) => { delete host[BOUNDARY_KEY] }
  const findNearestBoundary = (node: any): unknown => {
    let cur: any = node
    while (cur) {
      if (cur instanceof Element && cur[BOUNDARY_KEY]) return cur[BOUNDARY_KEY]
      cur = cur.parentNode ?? cur.host ?? null
    }
    return null
  }
  const runProtected = (node: any, source: string, fn: () => unknown): unknown => {
    try {
      return fn()
    } catch (err) {
      const handle = findNearestBoundary(node) as { catch(e: unknown, s: string): void } | null
      if (handle) { handle.catch(err, source); return undefined }
      throw err
    }
  }
  return { signal, mount, attachBoundary, detachBoundary, findNearestBoundary, runProtected }
})

import { mountRouter, type RouterHandle } from '../src/mount-router'
import { navigate, setEditMode } from '../src/index'
import { findNearestBoundary, runProtected } from '@ship/sail-runtime'

// Generic, app-agnostic fixtures. The router is framework code (deck) and must
// not know any app's routes — so these are abstract paths/tags, never an app's
// real IA. `/a/b` is a nested route; `x-chrome` is an arbitrary layout tag.
const MOD_INDEX = { __id: 'index' }
const MOD_LEAF = { __id: 'leaf' }

let router: RouterHandle | undefined

beforeEach(() => {
  // Edit mode skips the defineRoutes baseline gate so each test can register
  // its own pattern set without tripping the static-baseline check.
  setEditMode(true)
  history.replaceState({}, '', '/')
  document.body.innerHTML = '<div id="root"></div>'
  mountCalls.length = 0
})

afterEach(() => {
  router?.dispose()
  router = undefined
})

const start = (layouts: { pathPrefix: string; tag: string }[] = []) =>
  mountRouter({
    root: '#root',
    routes: [{ pattern: '/' }, { pattern: '/a/b' }],
    layouts,
    modules: { '/': MOD_INDEX, '/a/b': MOD_LEAF },
  })

const flush = () => vi.waitFor(() => expect(mountCalls.length).toBeGreaterThan(0))

describe('mountRouter', () => {
  it('mounts the matched index module into root on start', async () => {
    router = start()
    await flush()
    expect(mountCalls).toHaveLength(1)
    expect(mountCalls[0].opts.module).toBe(MOD_INDEX)
    expect(mountCalls[0].target).toBe(document.getElementById('root'))
  })

  it('navigate() routes through the registered port and mounts the new module', async () => {
    router = start()
    await flush()
    navigate('/a/b')
    await vi.waitFor(() => expect(mountCalls).toHaveLength(2))
    expect(mountCalls[1].opts.module).toBe(MOD_LEAF)
    expect(location.pathname).toBe('/a/b')
  })

  it('wraps the route in the layout chrome chain and mounts into the inner slot', async () => {
    router = start([{ pathPrefix: '/a', tag: 'x-chrome' }])
    await flush()
    navigate('/a/b')
    await vi.waitFor(() => expect(mountCalls).toHaveLength(2))
    const root = document.getElementById('root')!
    const chrome = root.querySelector('x-chrome')
    expect(chrome).not.toBeNull()
    const slot = chrome!.querySelector('div[slot="route"]')
    expect(slot).not.toBeNull()
    expect(mountCalls[1].target).toBe(slot)
  })

  it('intercepts same-origin anchor clicks as SPA navigations', async () => {
    router = start()
    await flush()
    const a = document.createElement('a')
    a.href = '/a/b'
    a.textContent = 'leaf'
    document.body.appendChild(a)
    a.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true, button: 0 }))
    await vi.waitFor(() => expect(mountCalls).toHaveLength(2))
    expect(mountCalls[1].opts.module).toBe(MOD_LEAF)
    expect(location.pathname).toBe('/a/b')
  })

  it('shows a fallback and does not call mount() when no route matches', async () => {
    router = start()
    await flush()
    navigate('/does-not-exist')
    await vi.waitFor(() => expect(location.pathname).toBe('/does-not-exist'))
    // No matching module → mount() not called a second time; fallback shown.
    expect(mountCalls).toHaveLength(1)
    const root = document.getElementById('root')!
    // The fallback wrapper class is rendered when <ship-error-state> is
    // not registered (e.g. an app that hasn't adopted @ship/primitives).
    // When registered, the router renders the element directly.
    expect(root.querySelector('.deck-not-found, ship-error-state')).not.toBeNull()
   })

  it('renders <ship-error-state> as the 404 surface when registered', async () => {
    // The hub's chrome view auto-registers <ship-error-state>; the router
    // then mounts the upgraded element instead of the bare fallback div.
    if (!customElements.get('ship-error-state')) {
      class ShipErrorState extends HTMLElement {
        static observedAttributes = ['variant', 'body', 'action-label']
        connectedCallback() {
          if (!this.shadowRoot) this.attachShadow({ mode: 'open' })
        }
      }
      customElements.define('ship-error-state', ShipErrorState)
    }
    router = start()
    await flush()
    navigate('/does-not-exist')
    await vi.waitFor(() => expect(location.pathname).toBe('/does-not-exist'))
    const root = document.getElementById('root')!
    const card = root.querySelector('ship-error-state')
    expect(card).not.toBeNull()
    expect(card!.getAttribute('variant')).toBe('not-found')
    expect(card!.getAttribute('body')).toContain('/does-not-exist')
    expect(card!.getAttribute('action-label')).toBe('Go back')
   })

  it('no-match with a matching layout prefix shows chrome + fallback in route slot', async () => {
    // The chrome element comes from the layout chain; the fallback lives in the
    // slot inside it — never a blank screen.
    history.replaceState({}, '', '/a/missing')
    router = mountRouter({
      root: '#root',
      routes: [{ pattern: '/a/b' }],
      layouts: [{ pathPrefix: '/a', tag: 'x-chrome' }],
      modules: { '/a/b': MOD_LEAF },
    })
    const root = document.getElementById('root')!
    // buildMountTarget is now async; wait for the layout to settle before asserting.
    await vi.waitFor(() => expect(root.querySelector('x-chrome')).not.toBeNull())
    const slot = root.querySelector('x-chrome div[slot="route"]')
    expect(slot).not.toBeNull()
    expect(slot!.querySelector('.deck-not-found, ship-error-state')).not.toBeNull()
  })

  it('passes arbitrary named clients through (no client name baked into the router)', async () => {
    const clients = { api: { tag: 'some-client' } }
    router = mountRouter({
      root: '#root',
      routes: [{ pattern: '/' }],
      modules: { '/': MOD_INDEX },
      clients,
    })
    await flush()
    expect(mountCalls[0].opts.clients).toBe(clients)
  })
})

// ---------------------------------------------------------------------------
// Layout-module rendering (the root-cause fix: layouts render authored markup)
// ---------------------------------------------------------------------------

describe('layout module rendering', () => {
  // A layout module stub with a slotted child: proves the authored markup
  // is rendered rather than silently dropped (the pre-fix bug).
  const LAYOUT_MOD_SLOTTED = {
    __mount(target: Element) {
      const chrome = document.createElement('x-chrome')
      const extra = document.createElement('x-extra')
      extra.setAttribute('slot', 'foo')
      chrome.appendChild(extra)
      target.appendChild(chrome)
    },
  }

  it('renders authored layout markup including slotted children', async () => {
    // The slotted child <x-extra slot="foo"> is composed in the _layout.sail
    // template. Before the fix, only the bare chrome tag was createElement'd
    // — the authored content was silently dropped. This test proves it now renders.
    history.replaceState({}, '', '/a/b')
    router = mountRouter({
      root: '#root',
      routes: [{ pattern: '/a/b' }],
      layouts: [{ pathPrefix: '/a', tag: 'x-chrome' }],
      modules: { '/a/b': MOD_LEAF },
      layoutModules: { '/a': LAYOUT_MOD_SLOTTED },
    })
    const root = document.getElementById('root')!
    await vi.waitFor(() => expect(root.querySelector('x-chrome')).not.toBeNull())
    // The authored <x-extra slot="foo"> must be present.
    expect(root.querySelector('x-extra[slot="foo"]')).not.toBeNull()
  })

  it('mounts route content into the innermost layout slot="route"', async () => {
    // Existing slotting contract holds: route mounts into div[slot="route"]
    // inside the chrome, even when the chrome is rendered via a layout module.
    const LAYOUT_MOD = {
      __mount(target: Element) {
        target.appendChild(document.createElement('x-chrome'))
      },
    }
    history.replaceState({}, '', '/a/b')
    router = mountRouter({
      root: '#root',
      routes: [{ pattern: '/a/b' }],
      layouts: [{ pathPrefix: '/a', tag: 'x-chrome' }],
      modules: { '/a/b': MOD_LEAF },
      layoutModules: { '/a': LAYOUT_MOD },
    })
    const root = document.getElementById('root')!
    await vi.waitFor(() => expect(mountCalls.length).toBeGreaterThan(1))
    const chrome = root.querySelector('x-chrome')!
    const slot = chrome.querySelector('div[slot="route"]')
    expect(slot).not.toBeNull()
    const routeMountCall = mountCalls.find((c) => c.opts?.module === MOD_LEAF)
    expect(routeMountCall).toBeDefined()
    expect(routeMountCall!.target).toBe(slot)
  })

  it('composes a 2-layout chain outer→inner with route at the innermost slot', async () => {
    // Two layouts: outer at '/' (x-outer) and inner at '/a' (x-inner).
    // Chain: root > x-outer > x-inner > div[slot="route"]. Route innermost.
    const OUTER_MOD = {
      __mount(target: Element) {
        target.appendChild(document.createElement('x-outer'))
      },
    }
    const INNER_MOD = {
      __mount(target: Element) {
        target.appendChild(document.createElement('x-inner'))
      },
    }
    history.replaceState({}, '', '/a/b')
    router = mountRouter({
      root: '#root',
      routes: [{ pattern: '/a/b' }],
      layouts: [
        { pathPrefix: '/', tag: 'x-outer' },
        { pathPrefix: '/a', tag: 'x-inner' },
      ],
      modules: { '/a/b': MOD_LEAF },
      layoutModules: { '/': OUTER_MOD, '/a': INNER_MOD },
    })
    const root = document.getElementById('root')!
    // 2 layout mounts + 1 route mount = at least 3 calls
    await vi.waitFor(() => expect(mountCalls.length).toBeGreaterThanOrEqual(3))
    const outer = root.querySelector('x-outer')
    expect(outer).not.toBeNull()
    const inner = outer!.querySelector('x-inner')
    expect(inner).not.toBeNull()
    // x-inner is itself a light-DOM child of x-outer's shadow tree — it must
    // carry slot="route" to land in x-outer's named slot in a real browser
    // (jsdom has no shadow DOM, so only the attribute is checkable here).
    expect(inner!.getAttribute('slot')).toBe('route')
    const innerSlot = inner!.querySelector('div[slot="route"]')
    expect(innerSlot).not.toBeNull()
    const routeMountCall = mountCalls.find((c) => c.opts?.module === MOD_LEAF)
    expect(routeMountCall!.target).toBe(innerSlot)
  })

  it('falls back to bare createElement when pathPrefix has no layoutModule entry', async () => {
    // layoutModules omits '/a' → createElement('x-chrome') fallback.
    // DOM structure identical to pre-module behavior; no regression.
    history.replaceState({}, '', '/a/b')
    router = mountRouter({
      root: '#root',
      routes: [{ pattern: '/a/b' }],
      layouts: [{ pathPrefix: '/a', tag: 'x-chrome' }],
      modules: { '/a/b': MOD_LEAF },
      layoutModules: {},
    })
    const root = document.getElementById('root')!
    await vi.waitFor(() => expect(mountCalls.length).toBeGreaterThan(0))
    const chrome = root.querySelector('x-chrome')
    expect(chrome).not.toBeNull()
    const slot = chrome!.querySelector('div[slot="route"]')
    expect(slot).not.toBeNull()
  })
})

// ---------------------------------------------------------------------------
// Default route error boundary — every route outlet is wrapped so a throwing
// view degrades to an error card instead of a dead subtree.
// ---------------------------------------------------------------------------

describe('default route error boundary', () => {
  it('installs a boundary on the route outlet by default (rootless case)', async () => {
    router = start()
    await flush()
    const root = document.getElementById('root')!
    // The mount target is still the root (boundary attaches, does not re-parent).
    expect(mountCalls[0].target).toBe(root)
    // A node anywhere under the outlet resolves to the installed boundary.
    expect(findNearestBoundary(root)).not.toBeNull()
  })

  it('installs the boundary on the inner route slot under chrome', async () => {
    router = start([{ pathPrefix: '/a', tag: 'x-chrome' }])
    await flush()
    navigate('/a/b')
    await vi.waitFor(() => expect(mountCalls).toHaveLength(2))
    const slot = document.getElementById('root')!.querySelector('div[slot="route"]')!
    expect(findNearestBoundary(slot)).not.toBeNull()
  })

  it('a view that throws on render degrades to an error card, not a crash', async () => {
    router = start()
    await flush()
    const root = document.getElementById('root')!
    // Simulate the view throwing through a runtime-protected call site.
    const viewNode = document.createElement('span')
    root.appendChild(viewNode)
    expect(() =>
      runProtected(viewNode, 'render', () => { throw new Error('view boom') }),
    ).not.toThrow()
    // No sail-error-state registered here → the registration-free bare card.
    expect(root.querySelector('.deck-route-error')).not.toBeNull()
    expect(root.hasAttribute('data-route-error')).toBe(true)
  })

  it('errorBoundary:false opts the whole app out', async () => {
    router = mountRouter({
      root: '#root',
      routes: [{ pattern: '/' }],
      modules: { '/': MOD_INDEX },
      errorBoundary: false,
    })
    await flush()
    const root = document.getElementById('root')!
    expect(findNearestBoundary(root)).toBeNull()
  })
})
