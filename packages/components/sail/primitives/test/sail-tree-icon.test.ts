// <sail-tree> — per-node icon (`node.icon`).
//
// A file tree shows a glyph per file type: a node carrying `icon` renders a
// `<sail-icon name=…>` ahead of its label; a node without one falls back to the
// `node-icon` slot. The icon rides the same `prop:tree` IDENTITY engine — it is
// node data, not a separate channel.

import { afterEach, describe, expect, it } from 'vitest'
import '../src/sail-tree/sail-tree.compiled.js'
import { createTable, type Table } from '../src/sail-table/engine/create-table'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

interface FileNode extends Record<string, unknown> {
  id: string
  label: string
  icon?: string
  children?: FileNode[]
}

const TREE: FileNode[] = [
  { id: 'readme', label: 'README.md', icon: 'file-text' },
  { id: 'main', label: 'main.rs', icon: 'file-code' },
  { id: 'plain', label: 'no-icon' }, // no icon → slot fallback path
]

function mountTree() {
  const tree = createTable<FileNode>({
    data: TREE,
    columns: [],
    getRowId: (r) => r.id,
    getSubRows: (r) => r.children,
    enableExpanding: true,
    initialState: { expanded: {} },
  }) as Table<FileNode>
  const m = mount(reg, 'sail-tree', {})
  ;(m.el as unknown as { tree: Table<FileNode> }).tree = tree
  return m
}

const rowFor = (shadow: ShadowRoot | HTMLElement, id: string) =>
  shadow.querySelector(`[role="treeitem"][data-node-id="${id}"]`) as HTMLElement

describe('<sail-tree> — per-node icon', () => {
  it('renders <sail-icon> with the node icon name ahead of the label', async () => {
    const { shadow } = mountTree()
    await flush()
    const readme = rowFor(shadow!, 'readme')
    const icon = readme.querySelector('.sail-icon[name]') as HTMLElement
    expect(icon).not.toBeNull()
    expect(icon.getAttribute('name')).toBe('file-text')

    const main = rowFor(shadow!, 'main')
    expect(main.querySelector('.sail-icon[name]')?.getAttribute('name')).toBe('file-code')
  })

  it('falls back to the node-icon slot when a node has no icon', async () => {
    const { shadow } = mountTree()
    await flush()
    const plain = rowFor(shadow!, 'plain')
    // No data-driven icon for this node…
    expect(plain.querySelector('.sail-icon[name]')).toBeNull()
    // …the slot fallback is present instead.
    expect(plain.querySelector('slot[name="node-icon"]')).not.toBeNull()
  })
})
