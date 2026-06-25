import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { assertNoMountLeak, measureMountLeak } from '../src/leak-budget'

// Keep the document clean so bodyDelta is measured against an empty baseline.
beforeEach(() => {
  document.body.innerHTML = ''
})
afterEach(() => {
  document.body.innerHTML = ''
})

describe('measureMountLeak', () => {
  it('reports ok for a clean mount/unmount cycle', () => {
    const r = measureMountLeak(
      () => {
        const el = document.createElement('div')
        document.body.appendChild(el)
        return () => el.remove()
      },
      { iterations: 50 },
    )
    expect(r.bodyDelta).toBe(0)
    expect(r.newDefines).toBe(0)
    expect(r.ok).toBe(true)
  })

  it('detects NODE RETENTION — a teardown that leaves detached roots', () => {
    const r = measureMountLeak(
      () => {
        const el = document.createElement('div')
        document.body.appendChild(el)
        return () => {
          /* deliberately do NOT remove — the leak */
        }
      },
      { iterations: 20 },
    )
    expect(r.bodyDelta).toBe(20)
    expect(r.ok).toBe(false)
  })

  it('detects REGISTRY GROWTH — a new custom element minted per cycle (#258 class)', () => {
    let n = 0
    const r = measureMountLeak(
      () => {
        // Mint a DISTINCT tag every cycle — the append-only-registry leak.
        const tag = `leak-probe-${n++}`
        if (!customElements.get(tag)) {
          customElements.define(tag, class extends HTMLElement {})
        }
        const el = document.createElement(tag)
        document.body.appendChild(el)
        return () => el.remove()
      },
      { iterations: 30 },
    )
    expect(r.bodyDelta).toBe(0) // nodes ARE cleaned…
    expect(r.definesPerIteration).toBeGreaterThanOrEqual(1) // …but the registry grows
    expect(r.ok).toBe(false)
  })

  it('treats a define-once component as bounded (warmup define not counted)', () => {
    const tag = 'leak-probe-once'
    customElements.define(tag, class extends HTMLElement {})
    const r = measureMountLeak(
      () => {
        const el = document.createElement(tag)
        document.body.appendChild(el)
        return () => el.remove()
      },
      { iterations: 40 },
    )
    expect(r.newDefines).toBe(0)
    expect(r.ok).toBe(true)
  })
})

describe('assertNoMountLeak', () => {
  it('returns the report when bounded', () => {
    const r = assertNoMountLeak(
      () => {
        const el = document.createElement('div')
        document.body.appendChild(el)
        return () => el.remove()
      },
      { iterations: 25 },
    )
    expect(r.ok).toBe(true)
  })

  it('throws a descriptive error when a leak is detected', () => {
    expect(() =>
      assertNoMountLeak(
        () => {
          const el = document.createElement('div')
          document.body.appendChild(el)
          return () => {
            /* leak */
          }
        },
        { iterations: 10 },
      ),
    ).toThrow(/mount leak/)
  })
})
