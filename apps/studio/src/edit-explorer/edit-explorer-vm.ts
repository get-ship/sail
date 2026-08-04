/**
 * Explorer VM — project file tree only (filter + open document).
 * Live Table identity is returned for `tree={tree}` on `<sail-tree>`.
 */
import { createTable } from '@ship/sail-primitives'
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

/** Group/dir rows expand only — not openable files. */
function isTreeMetaId(id: string): boolean {
  return id.startsWith('group:') || id.startsWith('dir:')
}

export function createEditExplorer() {
  const filterText = signal('')
  const hasTree = signal(false)
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

  const onTreeSelect = (e: Event) => {
    const ce = e as CustomEvent<{ id?: string }>
    const id = ce.detail?.id
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
    onFilterInput,
    onTreeSelect,
  }
}
