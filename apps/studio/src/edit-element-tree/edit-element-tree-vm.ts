/**
 * Element tree VM — sail-tree hierarchy + L1/L2/L3 badges in labels.
 * Click/select → shared selection + source scroll.
 */
import { createTable, type Table } from '@ship/sail-primitives'
import { mountAmbient, signal } from '@ship/sail-runtime'
import {
  activeDoc,
  elementSailTreeNodes,
  getEditState,
  scrollSourceEditor,
  selectEditNode,
  subscribeEdit,
  type SailTreeNode,
} from '@ship/studio/edit'

function assignTree(sel: string, table: Table<SailTreeNode>): void {
  if (typeof document === 'undefined') return
  const el = document.querySelector(sel) as (HTMLElement & { tree?: Table<SailTreeNode> }) | null
  if (el && el.tree !== table) el.tree = table
}

export function createEditElementTree() {
  const hasElements = signal(false)
  const activePathLabel = signal('—')
  const dataSig = signal<SailTreeNode[]>([])
  const tree = createTable<SailTreeNode>({
    data: dataSig,
    columns: [],
    getRowId: (r) => r.id,
    getSubRows: (r) => r.children,
    enableExpanding: true,
    enableRowSelection: true,
    filterFromLeafRows: true,
    navMode: 'row',
    initialState: { expanded: true },
  })

  const sync = () => {
    const es = getEditState()
    const doc = activeDoc()
    const nodes = doc
      ? elementSailTreeNodes(doc.tree.nodes, doc.tree.roots)
      : []
    dataSig.set(nodes)
    hasElements.set(nodes.length > 0)
    activePathLabel.set(es.activePath?.split('/').pop() ?? '—')
    if (es.selectionNodeId) {
      tree.setRowSelection({
        mode: 'include',
        ids: new Set([es.selectionNodeId]),
      })
    }
    queueMicrotask(() => assignTree('[data-element-tree-host]', tree))
  }

  sync()

  mountAmbient(() => {
    const unsub = subscribeEdit(() => sync())
    queueMicrotask(() => assignTree('[data-element-tree-host]', tree))
    return () => unsub()
  })

  const selectAndScroll = (id: string) => {
    selectEditNode(id)
    sync()
    const ta =
      typeof document !== 'undefined'
        ? (document.querySelector('[data-source-editor]') as HTMLTextAreaElement | null)
        : null
    scrollSourceEditor(ta)
  }

  const onTreeSelect = (e: Event) => {
    const ce = e as CustomEvent<{ id?: string }>
    if (!ce.detail?.id) return
    selectAndScroll(ce.detail.id)
  }

  return {
    hasElements,
    activePathLabel,
    onTreeSelect,
  }
}
