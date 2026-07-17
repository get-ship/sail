// TESTS-FIRST — Backbone #1, Slice 4: keyboard navigation model.
//
// Status: RED. `./create-table` (which wires `./nav`) does not exist yet;
// import-not-found is the expected red. DO NOT add the impl here.
//
// Authority: build-spec §Features #14 ("Keyboard-nav hooks": navMode 'row'|'cell';
// nav.onKeydown(e) returns true when handled; keymap = arrows move within the
// grid, Tab/Shift+Tab EXIT the grid, Home/End row ends, Ctrl+Home/Ctrl+End grid
// corners, PageUp/PageDown page the viewport, typeahead first-letter, tree
// Right/Left expand/collapse when getCanExpand else cell move, Space toggles
// selection, Enter/F2 enter edit + Escape cancels; nav SUSPENDED during edit;
// engine never touches the DOM), Slice 4.

import { describe, expect, it } from 'vitest'
// RED: greenfield module.
import { createTable } from './create-table'

interface Row {
  id: string
  name: string
  age: number
}
const COLUMNS = [
  { id: 'name', accessorKey: 'name' as const },
  { id: 'age', accessorKey: 'age' as const, enableEditing: true },
]
const ROWS: Row[] = [
  { id: 'a', name: 'Alpha', age: 1 },
  { id: 'b', name: 'Bravo', age: 2 },
  { id: 'c', name: 'Carol', age: 3 },
]

function mk(opts: Record<string, unknown> = {}) {
  return createTable<Row>({
    data: ROWS,
    columns: COLUMNS,
    getRowId: (r) => r.id,
    navMode: 'cell',
    ...opts,
  })
}

// A minimal KeyboardEvent-shaped stub — the engine reads key/ctrlKey/shiftKey
// and never touches the DOM, so a plain object is sufficient for the model.
function key(k: string, mods: Partial<{ ctrlKey: boolean; shiftKey: boolean; metaKey: boolean }> = {}) {
  return { key: k, ctrlKey: false, shiftKey: false, metaKey: false, ...mods, preventDefault() {} } as unknown as KeyboardEvent
}

describe('nav — navMode + active model', () => {
  it("navMode reflects the configured 'cell' | 'row' mode", () => {
    expect(mk({ navMode: 'cell' }).nav.mode()).toBe('cell')
    expect(mk({ navMode: 'row' }).nav.mode()).toBe('row')
  })

  it('focusCell sets nav.active to that row/column', () => {
    const table = mk()
    table.nav.focusCell('b', 'age')
    expect(table.nav.active()).toEqual({ rowId: 'b', columnId: 'age' })
  })
})

describe('nav — onKeydown returns true ONLY when it handles the key', () => {
  it('an arrow within the grid is handled (returns true)', () => {
    const table = mk()
    table.nav.focusCell('a', 'name')
    expect(table.nav.onKeydown(key('ArrowDown'))).toBe(true)
    expect(table.nav.active().rowId).toBe('b')
  })

  it('Tab EXITS the grid → NOT handled (returns false so the browser tabs away)', () => {
    const table = mk()
    table.nav.focusCell('a', 'name')
    expect(table.nav.onKeydown(key('Tab'))).toBe(false)
    expect(table.nav.onKeydown(key('Tab', { shiftKey: true }))).toBe(false)
  })

  it('an unmapped key is NOT handled (returns false)', () => {
    const table = mk()
    table.nav.focusCell('a', 'name')
    expect(table.nav.onKeydown(key('q'))).toBe(false)
  })
})

describe('nav — keymap (cell mode)', () => {
  it('ArrowRight / ArrowLeft move across columns', () => {
    const table = mk()
    table.nav.focusCell('a', 'name')
    table.nav.onKeydown(key('ArrowRight'))
    expect(table.nav.active().columnId).toBe('age')
    table.nav.onKeydown(key('ArrowLeft'))
    expect(table.nav.active().columnId).toBe('name')
  })

  it('Home / End jump to the row ends; Ctrl+Home / Ctrl+End jump to grid corners', () => {
    const table = mk()
    table.nav.focusCell('b', 'age')
    table.nav.onKeydown(key('Home'))
    expect(table.nav.active().columnId).toBe('name') // first column, same row
    table.nav.onKeydown(key('End'))
    expect(table.nav.active().columnId).toBe('age') // last column, same row
    table.nav.onKeydown(key('Home', { ctrlKey: true }))
    expect(table.nav.active()).toEqual({ rowId: 'a', columnId: 'name' }) // top-left
    table.nav.onKeydown(key('End', { ctrlKey: true }))
    expect(table.nav.active()).toEqual({ rowId: 'c', columnId: 'age' }) // bottom-right
  })

  it('Space toggles selection at the active cell', () => {
    const table = mk({ enableRowSelection: true })
    table.nav.focusCell('b', 'name')
    expect(table.nav.onKeydown(key(' '))).toBe(true)
    expect(table.getRow('b')!.getIsSelected()).toBe(true)
  })

  it('typeahead jumps to the next row whose active column starts with the char', () => {
    const table = mk()
    table.nav.focusCell('a', 'name')
    table.nav.typeahead('C') // Carol
    expect(table.nav.active().rowId).toBe('c')
  })
})

describe('nav — tree rows: Right/Left expand/collapse when getCanExpand, else move', () => {
  interface Node {
    id: string
    label: string
    children?: Node[]
  }
  const TREE: Node[] = [
    { id: 'p', label: 'Parent', children: [{ id: 'k', label: 'Kid' }] },
  ]
  function mkTree() {
    return createTable<Node>({
      data: TREE,
      columns: [{ id: 'label', accessorKey: 'label' }],
      getRowId: (r) => r.id,
      getSubRows: (r) => r.children,
      enableExpanding: true,
      navMode: 'row',
    })
  }

  it('ArrowRight on a collapsible row expands it (instead of moving columns)', () => {
    const table = mkTree()
    table.nav.focusCell('p', 'label')
    expect(table.nav.onKeydown(key('ArrowRight'))).toBe(true)
    expect(table.getRow('p')!.getIsExpanded()).toBe(true)
  })

  it('ArrowLeft on an expanded row collapses it', () => {
    const table = mkTree()
    table.toggleExpanded('p')
    table.nav.focusCell('p', 'label')
    expect(table.nav.onKeydown(key('ArrowLeft'))).toBe(true)
    expect(table.getRow('p')!.getIsExpanded()).toBe(false)
  })
})

describe('nav — ARIA-tree nav in navMode:row (#240 follow-up 3)', () => {
  interface Node {
    id: string
    label: string
    children?: Node[]
  }
  // Root ─ c1 ─ (g1, g2) , Root ─ c2
  const TREE: Node[] = [
    {
      id: 'root',
      label: 'Root',
      children: [
        { id: 'c1', label: 'C1', children: [{ id: 'g1', label: 'G1' }, { id: 'g2', label: 'G2' }] },
        { id: 'c2', label: 'C2' },
      ],
    },
  ]
  // Explicit per-id expansion map (NOT the expand-all sentinel) so collapsing one
  // node leaves the others expanded — the expand-all `true` demotes to a sparse map
  // on the first explicit collapse, which would collapse every unlisted node.
  function mkTree(expanded: Record<string, boolean> = { root: true, c1: true }) {
    return createTable<Node>({
      data: TREE,
      columns: [{ id: 'label', accessorKey: 'label' }],
      getRowId: (r) => r.id,
      getSubRows: (r) => r.children,
      enableExpanding: true,
      navMode: 'row',
      initialState: { expanded },
    })
  }

  it('ArrowLeft on an EXPANDED node collapses it (does not move)', () => {
    const t = mkTree()
    t.nav.focusCell('c1', 'label')
    expect(t.nav.onKeydown(key('ArrowLeft'))).toBe(true)
    expect(t.getRow('c1')!.getIsExpanded()).toBe(false)
    expect(t.nav.active().rowId).toBe('c1') // stayed put
  })

  it('ArrowLeft on a COLLAPSED node moves focus to its PARENT', () => {
    const t = mkTree({ root: true, c1: false }) // root open, c1 collapsed
    expect(t.getRow('c1')!.getIsExpanded()).toBe(false)
    t.nav.focusCell('c1', 'label')
    expect(t.nav.onKeydown(key('ArrowLeft'))).toBe(true)
    expect(t.nav.active().rowId).toBe('root') // moved to parent
  })

  it('ArrowLeft on a LEAF moves focus to its PARENT', () => {
    const t = mkTree()
    t.nav.focusCell('g1', 'label')
    expect(t.nav.onKeydown(key('ArrowLeft'))).toBe(true)
    expect(t.nav.active().rowId).toBe('c1') // leaf → parent
  })

  it('ArrowLeft on a depth-0 collapsed/leaf row with no parent is a no-op (stays put)', () => {
    const t = mkTree({ root: false }) // root collapsed → only root visible, depth 0, no parent
    t.nav.focusCell('root', 'label')
    expect(t.nav.onKeydown(key('ArrowLeft'))).toBe(true)
    expect(t.nav.active().rowId).toBe('root')
  })

  it('ArrowRight on a COLLAPSED node expands it (does not move)', () => {
    const t = mkTree({ root: true, c1: false }) // c1 collapsed, root open
    expect(t.getRow('c1')!.getIsExpanded()).toBe(false)
    t.nav.focusCell('c1', 'label')
    expect(t.nav.onKeydown(key('ArrowRight'))).toBe(true)
    expect(t.getRow('c1')!.getIsExpanded()).toBe(true)
    expect(t.nav.active().rowId).toBe('c1') // stayed put
  })

  it('ArrowRight on an EXPANDED node moves focus to its FIRST CHILD', () => {
    const t = mkTree() // root + c1 expanded
    t.nav.focusCell('c1', 'label')
    expect(t.nav.onKeydown(key('ArrowRight'))).toBe(true)
    expect(t.nav.active().rowId).toBe('g1') // first child
  })

  it('ArrowRight on a LEAF is a no-op (no children, cannot expand)', () => {
    const t = mkTree()
    t.nav.focusCell('g1', 'label')
    expect(t.nav.onKeydown(key('ArrowRight'))).toBe(true)
    expect(t.nav.active().rowId).toBe('g1')
    expect(t.getRow('g1')!.getCanExpand()).toBe(false)
  })

  it('Home jumps to the FIRST visible row, End to the LAST visible row', () => {
    const t = mkTree() // visible: root, c1, g1, g2, c2
    t.nav.focusCell('g1', 'label')
    t.nav.onKeydown(key('Home'))
    expect(t.nav.active().rowId).toBe('root')
    t.nav.onKeydown(key('End'))
    expect(t.nav.active().rowId).toBe('c2')
  })
})

describe('nav — FLAT table is UNAFFECTED by the ARIA-tree nav change', () => {
  function mkFlat() {
    return createTable<Row>({
      data: ROWS,
      columns: COLUMNS,
      getRowId: (r) => r.id,
      navMode: 'row',
    })
  }

  it('ArrowRight / ArrowLeft on a flat row still move across columns (no getCanExpand)', () => {
    const t = mkFlat()
    t.nav.focusCell('a', 'name')
    t.nav.onKeydown(key('ArrowRight'))
    expect(t.nav.active().columnId).toBe('age') // moved columns, not expand
    t.nav.onKeydown(key('ArrowLeft'))
    expect(t.nav.active().columnId).toBe('name')
    expect(t.nav.active().rowId).toBe('a') // never jumped to a parent
  })

  it('Home / End on a flat row jump to the column ends (depth-0 leaves, no tree)', () => {
    const t = mkFlat()
    t.nav.focusCell('b', 'age')
    t.nav.onKeydown(key('Home'))
    expect(t.nav.active().columnId).toBe('name')
    expect(t.nav.active().rowId).toBe('b') // same row
    t.nav.onKeydown(key('End'))
    expect(t.nav.active().columnId).toBe('age')
    expect(t.nav.active().rowId).toBe('b')
  })
})

describe('nav — suspended during edit', () => {
  it('while editing, arrow keys do NOT move the active cell (nav suspended)', () => {
    const table = mk()
    table.nav.focusCell('a', 'age')
    table.startEditing('a', 'age')
    const before = table.nav.active()
    const handled = table.nav.onKeydown(key('ArrowDown'))
    // The engine does not move focus during an edit (the input owns the keys).
    expect(table.nav.active()).toEqual(before)
    // ArrowDown is not consumed as navigation while editing.
    expect(handled).toBe(false)
  })
})
