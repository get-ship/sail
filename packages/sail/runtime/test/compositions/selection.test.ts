import { describe, expect, it } from 'vitest'
import { selection, signal, type Signal } from '../../src/index'

describe('selection()', () => {
  type Item = { id: string; name: string }

  it('selects, clears, toggles by key', () => {
    const items: Signal<Item[]> = signal([
      { id: 'a', name: 'Atlas' },
      { id: 'b', name: 'Beacon' },
      { id: 'c', name: 'Comet' },
    ])
    const sel = selection(items, (i) => i.id)

    expect(sel.selected()).toBeNull()
    expect(sel.selectedId()).toBeNull()

    sel.select('b')
    expect(sel.selectedId()).toBe('b')
    expect(sel.selected()?.name).toBe('Beacon')
    expect(sel.isSelected('b')).toBe(true)
    expect(sel.isSelected('a')).toBe(false)

    sel.clear()
    expect(sel.selectedId()).toBeNull()
    expect(sel.selected()).toBeNull()

    // toggle selects if absent, clears if it matches, replaces if it differs.
    sel.toggle('c')
    expect(sel.selectedId()).toBe('c')
    sel.toggle('c')
    expect(sel.selectedId()).toBeNull()
    sel.toggle('a')
    expect(sel.selectedId()).toBe('a')
    sel.toggle('b')
    expect(sel.selectedId()).toBe('b')
  })

  it('selected() is null once the id leaves the list', () => {
    const items: Signal<Item[]> = signal([{ id: 'a', name: 'Atlas' }])
    const sel = selection(items, (i) => i.id)
    sel.select('a')
    expect(sel.selected()?.id).toBe('a')

    // The id survives; the item it pointed at does not. A stale row must never
    // resolve to a stale object.
    items.set([])
    expect(sel.selected()).toBeNull()
    expect(sel.selectedId()).toBe('a')
  })
})
