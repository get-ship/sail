import { describe, expect, it } from 'vitest'
import { signal } from '../src/signal'
import {
  shared,
  isShared,
  provide,
  useShared,
  pickShared,
} from '../src/context'

describe('shared()', () => {
  it('marks a signal and returns it unchanged behaviorally', () => {
    const s = signal(0)
    const marked = shared(s)
    expect(marked).toBe(s)
    expect(isShared(marked)).toBe(true)

    marked.set(1)
    expect(s()).toBe(1)
  })

  it('isShared returns false on unmarked signals', () => {
    const s = signal(0)
    expect(isShared(s)).toBe(false)
  })
})

describe('provide / useShared', () => {
  it('child reads parent-published context via DOM walk', () => {
    const parent = document.createElement('div')
    const child = document.createElement('span')
    parent.appendChild(child)
    document.body.appendChild(parent)

    const ctx = { count: signal(42) }
    provide(parent, 'cmd', ctx)

    const found = useShared<typeof ctx>(child, 'cmd')
    expect(found).toBe(ctx)
    expect(found?.count()).toBe(42)

    document.body.removeChild(parent)
  })

  it('returns null when no harness in ancestry', () => {
    const orphan = document.createElement('span')
    document.body.appendChild(orphan)
    expect(useShared(orphan, 'unknown')).toBeNull()
    document.body.removeChild(orphan)
  })

  it('crosses shadow boundaries to find harness', () => {
    const grandparent = document.createElement('div')
    document.body.appendChild(grandparent)
    const root = grandparent.attachShadow({ mode: 'open' })

    const innerHost = document.createElement('section')
    root.appendChild(innerHost)
    const innerRoot = innerHost.attachShadow({ mode: 'open' })

    const child = document.createElement('span')
    innerRoot.appendChild(child)

    const ctx = { x: 1 }
    provide(grandparent, 'g', ctx)

    expect(useShared<typeof ctx>(child, 'g')).toBe(ctx)
    document.body.removeChild(grandparent)
  })

  it('nearest harness wins', () => {
    const outer = document.createElement('div')
    const inner = document.createElement('div')
    const leaf = document.createElement('span')
    outer.appendChild(inner)
    inner.appendChild(leaf)
    document.body.appendChild(outer)

    provide(outer, 'k', { who: 'outer' })
    provide(inner, 'k', { who: 'inner' })

    const found = useShared<{ who: string }>(leaf, 'k')
    expect(found?.who).toBe('inner')

    document.body.removeChild(outer)
  })
})

describe('pickShared()', () => {
  it('returns only entries whose values were marked shared', () => {
    const scope = {
      mode: shared(signal('find')),
      value: shared(signal('')),
      selectedIdx: signal(0),  // not shared
      label: 'plain string',   // not a signal at all
    }
    const sharedOnly = pickShared(scope)
    expect(Object.keys(sharedOnly).sort()).toEqual(['mode', 'value'])
  })

  it('writes through shared signals propagate (same identity)', () => {
    const s = shared(signal(0))
    const scope = { x: s }
    const picked = pickShared(scope)
    expect(picked.x).toBe(s)
    picked.x?.set(7)
    expect(s()).toBe(7)
  })
})
