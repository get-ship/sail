import { describe, expect, it } from 'vitest'
import {
  resolveController,
  signal,
  useSelection,
  type Signal,
} from '../../src/index'

const noShip = { ship: undefined }

describe('useSelection', () => {
  type Item = { id: string; name: string }

  it('selects, clears, toggles by key', () => {
    const items: Signal<Item[]> = signal([
      { id: 'a', name: 'Atlas' },
      { id: 'b', name: 'Beacon' },
      { id: 'c', name: 'Comet' },
    ])
    const b = resolveController(
      { use: { sel: useSelection(items, (i) => i.id) } },
      noShip,
    ) as Record<string, Record<string, unknown>>

    // Initial: no selection
    expect((b.sel!.selected as () => Item | null)()).toBeNull()
    expect((b.sel!.selectedId as () => string | null)()).toBeNull()

    // select(id) → derived.selected resolves through the source
    ;(b.sel!.select as (id: string) => void)('b')
    expect((b.sel!.selectedId as () => string | null)()).toBe('b')
    expect((b.sel!.selected as () => Item | null)()?.name).toBe('Beacon')

    // clear()
    ;(b.sel!.clear as () => void)()
    expect((b.sel!.selectedId as () => string | null)()).toBeNull()
    expect((b.sel!.selected as () => Item | null)()).toBeNull()

    // toggle(id) — selects if absent, clears if matches
    ;(b.sel!.toggle as (id: string) => void)('c')
    expect((b.sel!.selectedId as () => string | null)()).toBe('c')
    ;(b.sel!.toggle as (id: string) => void)('c')
    expect((b.sel!.selectedId as () => string | null)()).toBeNull()
    ;(b.sel!.toggle as (id: string) => void)('a')
    expect((b.sel!.selectedId as () => string | null)()).toBe('a')
    ;(b.sel!.toggle as (id: string) => void)('b') // different id — replaces
    expect((b.sel!.selectedId as () => string | null)()).toBe('b')
  })

  it('derived.selected returns null when the id is no longer in the list', () => {
    const items: Signal<Item[]> = signal([{ id: 'a', name: 'Atlas' }])
    const b = resolveController(
      { use: { sel: useSelection(items, (i) => i.id) } },
      noShip,
    ) as Record<string, Record<string, unknown>>
    ;(b.sel!.select as (id: string) => void)('a')
    expect((b.sel!.selected as () => Item | null)()?.id).toBe('a')

    // Item gone — derived returns null even though selectedId is still 'a'
    items.set([])
    expect((b.sel!.selected as () => Item | null)()).toBeNull()
    expect((b.sel!.selectedId as () => string | null)()).toBe('a')
  })
})
