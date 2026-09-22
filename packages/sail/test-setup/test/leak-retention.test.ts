// Self-tests for the WeakRef/FinalizationRegistry retention signal added to the
// leak-budget harness — the effect/subscription-retention class the original
// bodyDelta + define-count signals could NOT see: a node that IS removed from
// the DOM but is still pinned alive by a closure (an undisposed effect, an
// orphaned subscription callback, a listener never removed).
//
// These tests prove the new `measureRetention` catches a DELIBERATE
// closure-retain and clears a clean cycle. They are gated on `global.gc`
// (run with `node --expose-gc` / `--node-options=--expose-gc`); without it the
// measure reports `gcUnavailable: true` and the assertions adapt — the harness
// documents (does not silently pass over) the limitation.

import { describe, expect, it } from 'vitest'
import { measureRetention, assertNoRetention, gcAvailable } from '../src/leak-retention'

describe('measureRetention — closure-retention class', () => {
  it('reports NO retention for a cycle that lets its node go', async () => {
    const r = await measureRetention(() => {
      // A node created, used, and dropped — nothing outside the closure holds it.
      // Yield the node as the probe so the harness can prove it collects.
      const el = document.createElement('div')
      document.body.appendChild(el)
      return { teardown: () => el.remove(), probe: el }
    }, { iterations: 8 })

    expect(r.gcUnavailable).toBe(!gcAvailable())
    expect(r.retainedProbed).toBe(8) // every cycle handed back a probe
    if (!r.gcUnavailable) {
      // Every probed node collected after teardown.
      expect(r.retainedAfterTeardown).toBe(0)
      expect(r.ok).toBe(true)
    }
  })

  it('detects RETENTION — a teardown that removes the node but a closure still holds it', async () => {
    // The leak class #258-adjacent: an effect/subscription that survives unmount.
    const trap: object[] = []
    const r = await measureRetention(() => {
      const el = document.createElement('div')
      document.body.appendChild(el)
      // A "subscription" closure that captures the node and is never disposed.
      const handler = () => void el.tagName
      trap.push(handler) // <- the leak: the closure (and via it `el`) outlives teardown
      return {
        teardown: () => el.remove(), // node IS removed from the DOM…
        // …but `handler` (still in `trap`) keeps `el` reachable forever.
        probe: el,
      }
    }, { iterations: 8 })

    expect(r.retainedProbed).toBe(8)
    if (!r.gcUnavailable) {
      // bodyDelta sees nothing (the node was removed); retention catches it.
      expect(r.bodyDelta).toBe(0)
      expect(r.retainedAfterTeardown).toBeGreaterThan(0)
      expect(r.ok).toBe(false)
    }
  })
})

describe('assertNoRetention', () => {
  it('returns the report for a clean cycle', async () => {
    const r = await assertNoRetention(() => {
      const el = document.createElement('div')
      document.body.appendChild(el)
      return { teardown: () => el.remove(), probe: el }
    }, { iterations: 6 })
    // ok is true when gc collected everything, OR gc was unavailable (can't fail
    // a measure we couldn't run — but it's reported, not hidden).
    expect(r.ok || r.gcUnavailable).toBe(true)
  })

  it('throws a descriptive error when retention is detected (gc only)', async () => {
    if (!gcAvailable()) return // limitation documented; nothing to assert without gc
    const trap: object[] = []
    await expect(
      assertNoRetention(() => {
        const el = document.createElement('div')
        document.body.appendChild(el)
        const handler = () => void el.tagName
        trap.push(handler)
        return { teardown: () => el.remove(), probe: el }
      }, { iterations: 6 }),
    ).rejects.toThrow(/retention/)
  })
})

describe('measureMountLeak — retention folded into the existing report', () => {
  it('exposes the retention fields on the unified async report', async () => {
    const { measureMountLeakAsync } = await import('../src/leak-retention')
    const r = await measureMountLeakAsync(() => {
      const el = document.createElement('div')
      document.body.appendChild(el)
      return () => el.remove()
    }, { iterations: 6 })
    expect(r).toHaveProperty('retainedAfterTeardown')
    expect(r).toHaveProperty('gcUnavailable')
    expect(typeof r.definesPerIteration).toBe('number')
  })
})
