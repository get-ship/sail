import { describe, expect, it } from 'vitest'
import {
  filteredList,
  signal,
  type LiveSignal,
  type LiveState,
  type Signal,
} from '../../src/index'

describe('filteredList()', () => {
  it('filters a plain Signal<T[]> by query', () => {
    const items: Signal<string[]> = signal(['apple', 'banana', 'cherry'])
    const list = filteredList(items, (s, q) => s.includes(q))

    expect(list.filtered()).toEqual(['apple', 'banana', 'cherry'])

    list.setQuery('an')
    expect(list.filtered()).toEqual(['banana'])

    list.clear()
    expect(list.query()).toBe('')
    expect(list.filtered()).toHaveLength(3)
  })

  it('unwraps a LiveSignal<{items: T[]}> envelope', () => {
    const live: LiveSignal<{ items: { id: string; name: string }[] }> = signal({
      kind: 'ok',
      value: { items: [{ id: '1', name: 'Atlas' }, { id: '2', name: 'Beacon' }] },
    } as LiveState<{ items: { id: string; name: string }[] }>)

    const list = filteredList(live, (a, q) => a.name.toLowerCase().includes(q))
    expect(list.filtered().map((i) => i.id)).toEqual(['1', '2'])

    list.setQuery('Atlas')
    expect(list.filtered().map((i) => i.id)).toEqual(['1'])
  })

  it('hands `matches` a trimmed, lowercased query', () => {
    const seen: string[] = []
    const list = filteredList(signal(['x']), (_i, q) => {
      seen.push(q)
      return true
    })

    // One place normalizes, so every caller gives the same answer to "is search
    // case-sensitive". An empty query short-circuits — `matches` is never called.
    list.setQuery('  AtLaS  ')
    list.filtered()
    expect(seen).toEqual(['atlas'])

    list.setQuery('   ')
    expect(list.filtered()).toEqual(['x'])
    expect(seen).toEqual(['atlas'])
  })

  it('is empty while live is loading or errored', () => {
    const live: LiveSignal<{ items: number[] }> = signal({
      kind: 'loading',
    } as LiveState<{ items: number[] }>)
    const list = filteredList(live, () => true)
    expect(list.filtered()).toEqual([])

    live.set({ kind: 'error', error: new Error('x') } as LiveState<{ items: number[] }>)
    expect(list.filtered()).toEqual([])

    live.set({ kind: 'ok', value: { items: [1, 2, 3] } } as LiveState<{ items: number[] }>)
    expect(list.filtered()).toEqual([1, 2, 3])
  })
})
