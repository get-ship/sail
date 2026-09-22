import { describe, expect, it } from 'vitest'

import { hostDispatch } from '../src/host-property'

describe('hostDispatch', () => {
  it('dispatches a bubbling composed Event without a detail', () => {
    const node = document.createElement('div')
    const seen: Event[] = []
    node.addEventListener('input', (e) => seen.push(e))
    expect(hostDispatch(node, 'input', { bubbles: true, composed: true })).toBe(true)
    expect(seen).toHaveLength(1)
    expect(seen[0]!.bubbles).toBe(true)
    expect(seen[0]!.composed).toBe(true)
    expect(seen[0]!).not.toBeInstanceOf(CustomEvent)
  })

  it('uses CustomEvent when detail is present', () => {
    const node = document.createElement('div')
    const seen: CustomEvent[] = []
    node.addEventListener('sail-x', (e) => seen.push(e as CustomEvent))
    hostDispatch(node, 'sail-x', { bubbles: true, detail: { id: 1 } })
    expect(seen).toHaveLength(1)
    expect(seen[0]!.detail).toEqual({ id: 1 })
  })

  it('returns false for a null node', () => {
    expect(hostDispatch(null, 'input')).toBe(false)
  })
})
