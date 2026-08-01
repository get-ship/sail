/**
 * Explorer VM — sail-tree file nav (createTable identity) + filter.
 * Ops openDocument on leaf activate; shell seeds ops + paths.
 */
import { createTable, type Table } from '@ship/sail-primitives'
import { mountAmbient, signal } from '@ship/sail-runtime'
import { getShellState } from '@ship/sail-studio-shell'
import {
  fileSailTreeNodes,
  getEditState,
  openDocument,
  setFilter,
  subscribeEdit,
  type SailTreeNode,
} from '@ship/studio/edit'

function previewVp() {
  const shell = getShellState()
  return {
    width: shell.viewport,
    height: Math.round((shell.viewport * 9) / 16),
  }
}

function assignTree(
  sel: string,
  table: Table<SailTreeNode>,
): void {
  if (typeof document === 'undefined') return
  const el = document.querySelector(sel) as (HTMLElement & { tree?: Table<SailTreeNode> }) | null
  if (el && el.tree !== table) el.tree = table
}

export function createEditExplorer() {
  const filterText = signal('')
  const hasFiles = signal(false)
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
    const dirty = new Set<string>()
    for (const [p, d] of es.docs) {
      if (d.source.dirty) dirty.add(p)
    }
    const nodes = fileSailTreeNodes(es.files, es.filter, es.activePath, {
      dirtyPaths: dirty,
      sourceLen: (p) => es.docs.get(p)?.source?.text?.length ?? null,
    })
    dataSig.set(nodes)
    hasFiles.set(nodes.some((g) => (g.children?.length ?? 0) > 0) || nodes.length > 0)
    filterText.set(es.filter)
    if (es.activePath) {
      tree.setRowSelection({ mode: 'include', ids: new Set([es.activePath]) })
    }
    queueMicrotask(() => assignTree('[data-file-tree-host]', tree))
  }

  sync()

  mountAmbient(() => {
    const unsub = subscribeEdit(() => sync())
    queueMicrotask(() => assignTree('[data-file-tree-host]', tree))
    return () => unsub()
  })

  const openPath = (path: string) => {
    if (!path || path.startsWith('group:')) return Promise.resolve()
    const shell = getShellState()
    return openDocument(path, {
      loadPreview: false,
      theme: shell.themeId,
      mode: shell.modeId,
      viewport: previewVp(),
    }).then(() => sync())
  }

  const onFilterInput = (e: Event) => {
    setFilter((e.target as HTMLInputElement).value)
    sync()
  }

  const onTreeSelect = (e: Event) => {
    const ce = e as CustomEvent<{ id?: string }>
    const id = ce.detail?.id
    if (!id || id.startsWith('group:')) return
    void openPath(id)
  }

  return {
    filterText,
    hasFiles,
    onFilterInput,
    onTreeSelect,
  }
}
