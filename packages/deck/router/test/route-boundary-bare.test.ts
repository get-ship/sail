import { describe, it, expect, beforeEach, vi } from 'vitest'

// Isolated file (vitest runs each file in its own jsdom realm) so that
// `sui-error-state` is NEVER registered here — exercising the registration-free
// bare-card fallback (the same guard pattern mountRouter's buildNotFound uses).
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

import { installRouteBoundary } from '../src/route-boundary'
import { runProtected } from '@ship/sui-runtime'

let outlet: HTMLElement

beforeEach(() => {
  document.body.innerHTML = '<div id="outlet"></div>'
  outlet = document.getElementById('outlet')!
})

describe('installRouteBoundary — bare fallback when no primitive is registered', () => {
  it('renders a self-contained card with detail in dev', () => {
    expect(customElements.get('sui-error-state')).toBeUndefined()
    installRouteBoundary(outlet, { onReset: () => {}, dev: true })
    expect(() =>
      runProtected(outlet, 'render', () => { throw new Error('bare-detail') }),
    ).not.toThrow()
    const bare = outlet.querySelector('.deck-route-error')
    expect(bare).not.toBeNull()
    expect(bare!.getAttribute('role')).toBe('alert')
    expect(bare!.textContent).toContain('bare-detail')
    expect(bare!.querySelector('button')).not.toBeNull()
  })

  it('redacts the detail in prod', () => {
    installRouteBoundary(outlet, { onReset: () => {}, dev: false })
    runProtected(outlet, 'render', () => { throw new Error('prod-secret') })
    const bare = outlet.querySelector('.deck-route-error')!
    expect(bare.textContent ?? '').not.toContain('prod-secret')
    // Still graceful: a card with a retry, never a blank/dead subtree.
    expect(bare.querySelector('button')).not.toBeNull()
  })
})
