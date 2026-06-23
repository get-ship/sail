import { describe, expect, it } from 'vitest'
import {
  resolveController,
  signal,
  useFilteredList,
  type LiveState,
  type LiveSignal,
  type Signal,
} from '../../src/index'

const noShip = { ship: undefined }

describe('useFilteredList', () => {
  it('filters a plain Signal<T[]> by query', () => {
    const items: Signal<string[]> = signal(['apple', 'banana', 'cherry'])
    const b = resolveController(
      { use: { list: useFilteredList(items, (s, q) => s.includes(q)) } },
      noShip,
    ) as Record<string, Record<string, unknown>>

    expect((b.list!.filtered as () => string[])()).toEqual([
      'apple',
      'banana',
      'cherry',
    ])
    ;(b.list!.setQuery as (q: string) => void)('an')
    expect((b.list!.filtered as () => string[])()).toEqual(['banana'])
    ;(b.list!.clear as () => void)()
    expect((b.list!.query as () => string)()).toBe('')
    expect((b.list!.filtered as () => string[])()).toHaveLength(3)
  })

  it('unwraps a LiveSignal<{items: T[]}> envelope', () => {
    const live: LiveSignal<{ items: { id: string; name: string }[] }> = signal({
      kind: 'ok',
      value: { items: [{ id: '1', name: 'Atlas' }, { id: '2', name: 'Beacon' }] },
    } as LiveState<{ items: { id: string; name: string }[] }>)

    const b = resolveController(
      { use: { list: useFilteredList(live, (a, q) => a.name.includes(q)) } },
      noShip,
    ) as Record<string, Record<string, unknown>>

    const filtered = b.list!.filtered as () => { id: string; name: string }[]
    expect(filtered().map((i) => i.id)).toEqual(['1', '2'])
    ;(b.list!.setQuery as (q: string) => void)('Atlas')
    expect(filtered().map((i) => i.id)).toEqual(['1'])
  })

  it('returns empty array while live is loading or errored', () => {
    const live: LiveSignal<{ items: number[] }> = signal({
      kind: 'loading',
    } as LiveState<{ items: number[] }>)
    const b = resolveController(
      { use: { list: useFilteredList(live, () => true) } },
      noShip,
    ) as Record<string, Record<string, unknown>>
    expect((b.list!.filtered as () => number[])()).toEqual([])

    live.set({ kind: 'error', error: new Error('x') } as LiveState<{ items: number[] }>)
    expect((b.list!.filtered as () => number[])()).toEqual([])

    live.set({ kind: 'ok', value: { items: [1, 2, 3] } } as LiveState<{ items: number[] }>)
    expect((b.list!.filtered as () => number[])()).toEqual([1, 2, 3])
  })
})
