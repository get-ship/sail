import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
  defineRoutes,
  isEditMode,
  mergeRoutes,
  setEditMode,
} from '../src/index'

// Module-level SHAPES + STATIC_HASH persist across tests. We don't
// need to reset them for the merge tests (they only assert on
// `mergeRoutes`'s return value, not the registry). The gate tests
// use unique pattern names so their baselines don't collide with
// each other, and they set edit mode ON for the first call (to
// establish the baseline past the gate) then OFF for the second
// (to verify the gate rejects).
beforeEach(() => {
  setEditMode(true)
})

afterEach(() => {
  setEditMode(false)
})

describe('mergeRoutes', () => {
  it('returns the union when there are no collisions', () => {
    const staticR = [{ pattern: '/agents/:id' }]
    const runtimeR = [{ pattern: '/repos/:owner/:name' }]
    const { merged, collisions } = mergeRoutes(staticR, runtimeR)
    expect(merged).toHaveLength(2)
    expect(merged[0]).toEqual({ pattern: '/agents/:id' })
    expect(merged[1]).toEqual({ pattern: '/repos/:owner/:name' })
    expect(collisions).toEqual([])
  })

  it('drops runtime duplicates silently under first-wins (default)', () => {
    const staticR = [{ pattern: '/agents/:id' }]
    const runtimeR = [{ pattern: '/agents/:id' }, { pattern: '/runs/:id' }]
    const { merged, collisions } = mergeRoutes(staticR, runtimeR)
    expect(merged.map((r) => r.pattern)).toEqual(['/agents/:id', '/runs/:id'])
    // first-wins: collisions array is empty (the caller didn't ask
    // to be told).
    expect(collisions).toEqual([])
  })

  it('logs and tracks collisions under onCollision=log', () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    const staticR = [{ pattern: '/agents/:id' }, { pattern: '/runs/:id' }]
    const runtimeR = [{ pattern: '/agents/:id' }, { pattern: '/runs/:id' }]
    const { merged, collisions } = mergeRoutes(staticR, runtimeR, {
      onCollision: 'log',
    })
    expect(merged).toHaveLength(2)
    expect(collisions).toEqual(['/agents/:id', '/runs/:id'])
    expect(warn).toHaveBeenCalledTimes(2)
    warn.mockRestore()
  })

  it('throws on the first collision under onCollision=error', () => {
    const staticR = [{ pattern: '/agents/:id' }]
    const runtimeR = [{ pattern: '/agents/:id' }, { pattern: '/runs/:id' }]
    expect(() => mergeRoutes(staticR, runtimeR, { onCollision: 'error' })).toThrow(
      /runtime pattern '\/agents\/:id' collides/,
    )
  })

  it('returns empty merged set when both halves are empty', () => {
    const { merged, collisions } = mergeRoutes([], [])
    expect(merged).toEqual([])
    expect(collisions).toEqual([])
  })

  it('treats a runtime pattern that equals a static pattern as a collision (dropped)', () => {
    const staticR = [{ pattern: '/agents/:id' }]
    const runtimeR = [{ pattern: '/agents/:id' }]
    const { merged, collisions } = mergeRoutes(staticR, runtimeR)
    expect(merged).toHaveLength(1)
    expect(collisions).toEqual([])
    expect(merged[0]?.pattern).toBe('/agents/:id')
  })

  it('treats a runtime pattern that overlaps but is not equal as a non-collision', () => {
    const staticR = [{ pattern: '/agents/me' }]
    const runtimeR = [{ pattern: '/agents/:id' }]
    const { merged, collisions } = mergeRoutes(staticR, runtimeR)
    expect(merged).toHaveLength(2)
    expect(collisions).toEqual([])
  })

  it('appends merged into opts.target without registering globally', () => {
    const target: { pattern: string }[] = []
    const staticR = [{ pattern: '/a' }]
    const runtimeR = [{ pattern: '/b' }, { pattern: '/c' }]
    const { merged, collisions } = mergeRoutes(staticR, runtimeR, { target })
    expect(merged).toEqual([...staticR, ...runtimeR])
    expect(collisions).toEqual([])
    expect(target).toEqual([...staticR, ...runtimeR])
  })
})

describe('edit-mode toggle', () => {
  it('isEditMode() reflects the current state', () => {
    setEditMode(true)
    expect(isEditMode()).toBe(true)
    setEditMode(false)
    expect(isEditMode()).toBe(false)
  })

  it('setEditMode(true) allows defineRoutes to grow the registry', () => {
    setEditMode(true)
    expect(() => {
      defineRoutes([{ pattern: '/edit-mode-test-a' }])
      defineRoutes([{ pattern: '/edit-mode-test-a' }, { pattern: '/edit-mode-test-b' }])
    }).not.toThrow()
  })

  it('setEditMode(false) rejects defineRoutes additions after the baseline', () => {
    // Establish baseline with the gate open, then close it and
    // attempt a different pattern — must throw.
    setEditMode(true)
    defineRoutes([{ pattern: '/gate-uniq-a' }])
    setEditMode(false)
    expect(() => defineRoutes([{ pattern: '/gate-uniq-b' }])).toThrow(/outside edit mode/)
  })
})
