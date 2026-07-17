// Slice 8 — harden-safety audit (#133). The confined-view endowment surface is
// `harden()`-ed, which transitively freezes everything reachable. This asserts
// the REAL `@ship/sail-runtime` reactive core survives that freeze and stays
// functional — the precondition for endowing the real runtime to a confined
// view.
//
// deck-host does NOT depend on the runtime in production (the host INJECTS it);
// this test reaches over by relative path purely to exercise the real symbols,
// without creating a package-level dependency edge.

import { beforeAll, describe, expect, it } from 'vitest'

import { buildViewEndowments, lockdownRealm } from '../src/index.ts'
// The real reactive core that every confined controller calls.
import {
  signal,
  derived,
  effect,
  batch,
  untrack,
  isSignal,
} from '../../../sail/runtime/src/signal.ts'

const fakeDoc = {
  createElement: (t: string) => ({ tagName: t.toUpperCase() }),
  createElementNS: (_ns: string, t: string) => ({ tagName: t }),
  createTextNode: (t: string) => ({ nodeValue: t }),
  createComment: (t: string) => ({ nodeValue: t }),
  createDocumentFragment: () => ({ kind: 'fragment' }),
} as unknown as Document

describe('harden-safety of the real @ship/sail-runtime reactive core', () => {
  beforeAll(() => lockdownRealm())

  it('hardens the real reactive surface without throwing', () => {
    // buildViewEndowments runs harden() over the spread runtime namespace. If
    // any real export were harden-hostile, this constructor would throw.
    const surface = buildViewEndowments({
      runtime: { signal, derived, effect, batch, untrack, isSignal },
      ship: {},
      doc: fakeDoc,
    }) as Record<string, unknown>
    expect(Object.isFrozen(surface)).toBe(true)
    expect(surface.signal).toBe(signal)
  })

  it('signals stay readable + writable + derivable through the hardened surface', () => {
    const surface = buildViewEndowments({
      runtime: { signal, derived, effect, batch, untrack, isSignal },
      ship: {},
      doc: fakeDoc,
    }) as { signal: typeof signal; derived: typeof derived }
    // The factory is frozen; its OUTPUT (a fresh signal) is not — and needn't
    // be: `.set` mutates internal closure state, not a frozen own-property.
    const count = surface.signal(0)
    count.set(2)
    expect(count()).toBe(2)
    const doubled = surface.derived(() => count() * 2)
    expect(doubled()).toBe(4)
    count.set(5)
    expect(doubled()).toBe(10) // reactivity intact through the frozen surface
  })

  it('effects re-run on dep change through the hardened surface', () => {
    const surface = buildViewEndowments({
      runtime: { signal, derived, effect, batch, untrack, isSignal },
      ship: {},
      doc: fakeDoc,
    }) as { signal: typeof signal; effect: typeof effect }
    const s = surface.signal(1)
    const seen: number[] = []
    // Return void — a non-void return is interpreted by the effect system as a
    // cleanup function.
    surface.effect(() => {
      seen.push(s())
    })
    s.set(2)
    s.set(3)
    expect(seen).toEqual([1, 2, 3])
  })
})
