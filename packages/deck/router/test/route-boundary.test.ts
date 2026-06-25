import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'

// Same mock strategy as mount-router.test.ts: the real @ship/sui-runtime pulls
// a cycle vitest can't resolve. We provide a FAITHFUL re-implementation of the
// error-boundary helpers (identical Symbol key + DOM walk + runProtected
// try/catch) so a throw routed through `runProtected` reaches the boundary
// exactly as it does against the real runtime.
vi.mock('@ship/sui-runtime', () => {
  const BOUNDARY_KEY = Symbol.for('sui:error-boundary')
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
  return { attachBoundary, detachBoundary, findNearestBoundary, runProtected }
})

import { installRouteBoundary, isDevMode } from '../src/route-boundary'
// The faithful runProtected from the mock — used to simulate a view throwing
// through the runtime's protected call site, the way bindFor/bindIf/effect do.
import { runProtected, findNearestBoundary } from '@ship/sui-runtime'

let outlet: HTMLElement

beforeEach(() => {
  document.head.querySelector('meta[name="ship-mode"]')?.remove()
  document.body.innerHTML = '<div id="outlet"></div>'
  outlet = document.getElementById('outlet')!
})

afterEach(() => {
  document.head.querySelector('meta[name="ship-mode"]')?.remove()
})

const setDevMeta = (on: boolean) => {
  document.head.querySelector('meta[name="ship-mode"]')?.remove()
  if (on) {
    const m = document.createElement('meta')
    m.setAttribute('name', 'ship-mode')
    m.setAttribute('content', 'development')
    document.head.appendChild(m)
  }
}

const defineSuiErrorState = () => {
  if (!customElements.get('sui-error-state')) {
    customElements.define('sui-error-state', class extends HTMLElement {})
  }
}

describe('isDevMode', () => {
  it('is true only when the daemon dev meta tag is present', () => {
    setDevMeta(false)
    expect(isDevMode()).toBe(false)
    setDevMeta(true)
    expect(isDevMode()).toBe(true)
  })
})

describe('installRouteBoundary — throwing route degrades to a card', () => {
  it('renders <sui-error-state> (not a crash) when a view throws on render', () => {
    defineSuiErrorState()
    installRouteBoundary(outlet, { onReset: () => {}, dev: true })

    // Simulate the view mounting a node and then throwing during render — the
    // runtime's render primitives wrap such call sites in runProtected().
    const viewNode = document.createElement('span')
    outlet.appendChild(viewNode)
    expect(() =>
      runProtected(viewNode, 'render', () => {
        throw new Error('probe: view exploded on render')
      }),
    ).not.toThrow()

    const card = outlet.querySelector('sui-error-state')
    expect(card).not.toBeNull()
    expect(outlet.hasAttribute('data-route-error')).toBe(true)
    // Broken view DOM is gone — degraded, not dead.
    expect(outlet.querySelector('span')).toBeNull()
  })

  it('shows the error message + stack in dev, redacts in prod', () => {
    defineSuiErrorState()

    // Dev outlet
    installRouteBoundary(outlet, { onReset: () => {}, dev: true })
    runProtected(outlet, 'render', () => { throw new Error('boom-dev') })
    const devCard = outlet.querySelector('sui-error-state')!
    expect(devCard.getAttribute('code')).toBe('boom-dev')
    expect(devCard.querySelector('pre[slot="details"]')).not.toBeNull()
    expect(devCard.querySelector('button[slot="action"]')).not.toBeNull()

    // Prod outlet — fresh
    const prodOutlet = document.createElement('div')
    document.body.appendChild(prodOutlet)
    installRouteBoundary(prodOutlet, { onReset: () => {}, dev: false })
    runProtected(prodOutlet, 'render', () => { throw new Error('boom-prod-secret') })
    const prodCard = prodOutlet.querySelector('sui-error-state')!
    expect(prodCard.getAttribute('code')).toBeNull()
    expect(prodCard.querySelector('pre[slot="details"]')).toBeNull()
    expect(prodCard.getAttribute('body') ?? '').not.toContain('boom-prod-secret')
  })

  it('retry clears the error state and re-runs the route mount', () => {
    defineSuiErrorState()
    const onReset = vi.fn()
    installRouteBoundary(outlet, { onReset, dev: true })
    runProtected(outlet, 'render', () => { throw new Error('retry-me') })
    expect(outlet.hasAttribute('data-route-error')).toBe(true)

    const retry = outlet.querySelector('button[slot="action"]') as HTMLButtonElement
    retry.click()
    expect(onReset).toHaveBeenCalledTimes(1)
    expect(outlet.hasAttribute('data-route-error')).toBe(false)
  })
})

describe('installRouteBoundary — opt-out / no double-wrap', () => {
  it('is a no-op when a boundary is already present in the ancestry', () => {
    // A layout supplied its own boundary on a wrapper above the outlet.
    const wrapper = document.createElement('div')
    document.body.appendChild(wrapper)
    const innerOutlet = document.createElement('div')
    wrapper.appendChild(innerOutlet)

    // Pre-install a boundary on the wrapper (simulating a layout's own).
    const layoutHandle = installRouteBoundary(wrapper, { onReset: () => {}, dev: true })
    expect(layoutHandle).not.toBeNull()

    // Installing on the inner outlet must detect the ancestor boundary and skip.
    const installed = installRouteBoundary(innerOutlet, { onReset: () => {}, dev: true })
    expect(installed).toBeNull()
    // No handle attached to the inner outlet → nearest boundary is still the wrapper's.
    expect(findNearestBoundary(innerOutlet)).toBe(findNearestBoundary(wrapper))
  })
})
