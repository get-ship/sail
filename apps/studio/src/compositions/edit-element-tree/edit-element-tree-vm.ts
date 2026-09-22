/**
 * Components tab element tree — active-doc sail-tree, L1/L2/L3 in trailing.
 * Select → shared selection + Source view + span scroll (when editor mounted).
 * Live Table identity returned for `tree={tree}` on `<sail-tree>`.
 */
import { createTable } from '@ship/sail-components/primitives'
import { mountAmbient } from '@ship/sail-runtime'
import { activeDoc, elementSailTreeNodes, getEditState, querySourceEditor, scrollSourceEditor, selectEditNode, setView, subscribeEdit, type SailTreeNode } from '../../edit/index.ts'

import { signal } from '@ship/sail-runtime'
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
    // Active-doc tree only — no project-wide element index in the store.
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
  }

  sync()

  mountAmbient(() => {
    const unsub = subscribeEdit(() => sync())
    return () => unsub()
  })

  const selectAndScroll = (id: string) => {
    selectEditNode(id)
    // Components → source at span (surface re-scrolls when editor mounts).
    setView('source')
    sync()
    queueMicrotask(() => scrollSourceEditor(querySourceEditor()))
  }

  const onTreeSelect = (detail: { id?: string }) => {
    if (!detail?.id) return
    selectAndScroll(detail.id)
  }

  return {
    /** Live Table identity for `<sail-tree tree={tree}>`. */
    tree,
    hasElements,
    activePathLabel,
    onTreeSelect,
  }
}
