/**
 * Explorer VM — project file tree only (filter + open document).
 * Live Table identity is returned for `tree={tree}` on `<sail-tree>`.
 */
import { createTable } from '../../vendor/primitives'
import { mountAmbient } from '../../vendor/runtime'
import { getShellState } from '../../vendor/studio-shell'
import { fileSailTreeNodes, getEditState, openDocument, setFilter, subscribeEdit, type SailTreeNode } from '../../edit/index.ts'

import { signal } from '../../vendor/runtime'
function previewVp() {
  const shell = getShellState()
  return {
    width: shell.viewport,
    height: Math.round((shell.viewport * 9) / 16),
  }
}

/** Group/dir rows expand only — not openable files. */
function isTreeMetaId(id: string): boolean {
  return id.startsWith('group:') || id.startsWith('dir:')
}

function countLeaves(nodes: SailTreeNode[]): number {
  let n = 0
  const walk = (list: SailTreeNode[]) => {
    for (const node of list) {
      if (node.children?.length) walk(node.children)
      else n += 1
    }
  }
  walk(nodes)
  return n
}

export function createEditExplorer() {
  const filterText = signal('')
  const hasTree = signal(false)
  const dirtyCountLabel = signal('')
  const fileCountLabel = signal('0 files')
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

  const syncSelection = (path: string | null) => {
    if (path) {
      tree.setRowSelection({ mode: 'include', ids: new Set([path]) })
    } else {
      tree.resetRowSelection()
    }
  }

  const sync = () => {
    const es = getEditState()
    const dirty = new Set<string>()
    for (const [p, d] of es.docs) {
      if (d.source.dirty) dirty.add(p)
    }
    const shown = fileSailTreeNodes(es.files, es.filter, es.activePath, {
      dirtyPaths: dirty,
    })
    dataSig.set(shown)
    hasTree.set(
      shown.some((g) => (g.children?.length ?? 0) > 0) || shown.length > 0,
    )
    filterText.set(es.filter)
    const leaves = countLeaves(shown)
    fileCountLabel.set(leaves === 1 ? '1 file' : `${leaves} files`)
    dirtyCountLabel.set(
      dirty.size > 0 ? `${dirty.size} dirty` : '',
    )
    syncSelection(es.activePath)
  }

  sync()

  mountAmbient(() => {
    const unsub = subscribeEdit(() => sync())
    return () => unsub()
  })

  const openPath = (path: string) => {
    if (!path || isTreeMetaId(path)) return Promise.resolve()
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

  const onTreeSelect = (detail: { id?: string }) => {
    const id = detail?.id
    if (!id) return
    if (isTreeMetaId(id)) {
      syncSelection(getEditState().activePath)
      return
    }
    void openPath(id)
  }

  return {
    /** Live Table identity for `<sail-tree tree={tree}>` (bindDynamic → host.tree). */
    tree,
    filterText,
    hasTree,
    dirtyCountLabel,
    fileCountLabel,
    onFilterInput,
    onTreeSelect,
  }
}
