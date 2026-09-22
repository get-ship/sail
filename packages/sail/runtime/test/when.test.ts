import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest'
import {
  bindWhen,
  acquireMediaQuery,
  __resetMediaQueryCacheForTests,
} from '../src/when'

type Mql = MediaQueryList & {
  /** test helper */
  _setMatches(v: boolean): void
}

function installMatchMedia(initial: boolean): {
  mql: Mql
  listeners: Set<() => void>
} {
  const listeners = new Set<() => void>()
  let matches = initial
  const mql = {
    get matches() {
      return matches
    },
    media: '',
    onchange: null,
    addEventListener(_type: string, cb: EventListenerOrEventListenerObject) {
      listeners.add(cb as () => void)
    },
    removeEventListener(_type: string, cb: EventListenerOrEventListenerObject) {
      listeners.delete(cb as () => void)
    },
    addListener() {},
    removeListener() {},
    dispatchEvent() {
      return false
    },
    _setMatches(v: boolean) {
      matches = v
      for (const l of listeners) l()
    },
  } as Mql
  const fn = vi.fn(() => mql)
  // jsdom resolves window.matchMedia; stub both surfaces.
  vi.stubGlobal('matchMedia', fn)
  Object.defineProperty(window, 'matchMedia', {
    configurable: true,
    writable: true,
    value: fn,
  })
  return { mql, listeners }
}

function setup() {
  const host = document.createElement('div')
  document.body.appendChild(host)
  const anchor = document.createComment('when')
  host.appendChild(anchor)
  return { host, anchor }
}

describe('bindWhen', () => {
  afterEach(() => {
    __resetMediaQueryCacheForTests()
    vi.unstubAllGlobals()
    document.body.innerHTML = ''
  })

  it('renders then-branch when media matches', () => {
    installMatchMedia(true)
    const { host, anchor } = setup()
    bindWhen({
      anchor,
      query: '(min-width: 768px)',
      then_: () => ({ nodes: [document.createTextNode('WIDE')] }),
      else_: () => ({ nodes: [document.createTextNode('NARROW')] }),
    })
    expect(host.textContent).toBe('WIDE')
  })

  it('renders else-branch when media does not match (and SSR-style false)', () => {
    installMatchMedia(false)
    const { host, anchor } = setup()
    bindWhen({
      anchor,
      query: '(min-width: 768px)',
      then_: () => ({ nodes: [document.createTextNode('WIDE')] }),
      else_: () => ({ nodes: [document.createTextNode('NARROW')] }),
    })
    expect(host.textContent).toBe('NARROW')
  })

  it('swaps branches on matchMedia change (branch swap, not dual hidden)', () => {
    const { mql } = installMatchMedia(false)
    const { host, anchor } = setup()
    const thenFactory = vi.fn(() => ({ nodes: [document.createTextNode('WIDE')] }))
    const elseFactory = vi.fn(() => ({ nodes: [document.createTextNode('NARROW')] }))
    bindWhen({
      anchor,
      query: '(min-width: 768px)',
      then_: thenFactory,
      else_: elseFactory,
    })
    expect(host.textContent).toBe('NARROW')
    expect(host.querySelectorAll('[hidden]').length).toBe(0)
    mql._setMatches(true)
    expect(host.textContent).toBe('WIDE')
    // Only the active branch is in the tree — not two siblings with hidden.
    expect(host.childNodes.length).toBe(2) // text + comment anchor
    mql._setMatches(false)
    expect(host.textContent).toBe('NARROW')
  })

  it('defaults to else when matchMedia is unavailable (SSR)', () => {
    Object.defineProperty(window, 'matchMedia', {
      configurable: true,
      writable: true,
      value: undefined,
    })
    vi.stubGlobal('matchMedia', undefined)
    const { host, anchor } = setup()
    bindWhen({
      anchor,
      query: '(min-width: 1280px)',
      then_: () => ({ nodes: [document.createTextNode('DESKTOP')] }),
      else_: () => ({ nodes: [document.createTextNode('DEFAULT')] }),
    })
    expect(host.textContent).toBe('DEFAULT')
  })

  it('shares one listener across concurrent bindWhen on the same query', () => {
    const { listeners } = installMatchMedia(false)
    const a1 = setup()
    const a2 = setup()
    const d1 = bindWhen({
      anchor: a1.anchor,
      query: '(min-width: 768px)',
      then_: () => ({ nodes: [document.createTextNode('A')] }),
      else_: () => ({ nodes: [document.createTextNode('a')] }),
    })
    const d2 = bindWhen({
      anchor: a2.anchor,
      query: '(min-width: 768px)',
      then_: () => ({ nodes: [document.createTextNode('B')] }),
      else_: () => ({ nodes: [document.createTextNode('b')] }),
    })
    expect(listeners.size).toBe(1)
    d1()
    expect(listeners.size).toBe(1)
    d2()
    expect(listeners.size).toBe(0)
  })
})

describe('acquireMediaQuery', () => {
  beforeEach(() => {
    __resetMediaQueryCacheForTests()
    installMatchMedia(true)
  })
  afterEach(() => {
    __resetMediaQueryCacheForTests()
    vi.unstubAllGlobals()
  })

  it('refcounts and tears down on last release', () => {
    const { listeners } = installMatchMedia(true)
    const a = acquireMediaQuery('(min-width: 390px)')
    const b = acquireMediaQuery('(min-width: 390px)')
    expect(a.matches()).toBe(true)
    expect(listeners.size).toBe(1)
    a.release()
    expect(listeners.size).toBe(1)
    b.release()
    expect(listeners.size).toBe(0)
  })
})
