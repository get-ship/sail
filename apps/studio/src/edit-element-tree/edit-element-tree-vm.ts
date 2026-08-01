/**
 * Element tree VM — sail-tree hierarchy + L1/L2/L3 badges in labels.
 * Click/select → shared selection + source scroll (when Source mounted).
 */
import { createTable, type Table } from '@ship/sail-primitives'
import { mountAmbient, signal } from '@ship/sail-runtime'
import {
  activeDoc,
  elementSailTreeNodes,
  getEditState,
  querySourceEditor,
  scrollSourceEditor,
  selectEditNode,
  subscribeEdit,
  type SailTreeNode,
} from '@ship/studio/edit'
import { assignTreeHost } from '../edit/assign-tree'

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
    } else {
      tree.resetRowSelection()
    }
    queueMicrotask(() => assignTreeHost('[data-element-tree-host]', tree))
  }

  sync()

  mountAmbient(() => {
    const unsub = subscribeEdit(() => sync())
    queueMicrotask(() => assignTreeHost('[data-element-tree-host]', tree))
    return () => unsub()
  })

  const selectAndScroll = (id: string) => {
    selectEditNode(id)
    sync()
    // Only applies when Source view mounts the editor; surface re-applies on enter.
    scrollSourceEditor(querySourceEditor())
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
