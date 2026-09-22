import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'

vi.mock('@ship/sail-runtime', () => {
  const signal = <T>(initial: T) => {
    let v = initial
    const s = (() => v) as { (): T; set(n: T): void; peek(): T }
    s.set = (n: T) => { v = n }
    s.peek = () => v
    return s
  }
  return { signal }
})

import { route, navigate, setEditMode, setRouterPort } from '../src/index'
import { mountRouter } from '../src/mount-router'

describe('probe pathname', () => {
  beforeEach(() => {
    setEditMode(true)
    history.replaceState({}, '', '/')
    document.body.innerHTML = '<div id="root"></div>'
    // clear any leftover port
    ;(globalThis as any).__shipRouterPort = undefined
  })
  afterEach(() => {
    ;(globalThis as any).__shipRouterPort = undefined
  })
  it('navigate no-port updates route.pathname sync', () => {
    const r = route()
    expect(r.pathname()).toBe('/')
    navigate('/a/b')
    expect(r.pathname()).toBe('/a/b')
  })
  it('navigate with mountRouter port updates shared pathname sync (chrome reuse)', async () => {
    const handle = mountRouter({
      root: '#root',
      routes: [{ pattern: '/' }, { pattern: '/a/b' }, { pattern: '/a/c' }],
      layouts: [{ pathPrefix: '/a', tag: 'x-chrome' }],
      modules: { '/': { __id: 'i' }, '/a/b': { __id: 'b' }, '/a/c': { __id: 'c' } },
    })
    // layout-style consumer: only route.pathname, no location wrap
    const layoutView = route()
    expect(layoutView.pathname()).toBe('/')
    navigate('/a/b')
    expect(layoutView.pathname()).toBe('/a/b')
    navigate('/a/c')
    expect(layoutView.pathname()).toBe('/a/c')
    handle.dispose()
  })
})
