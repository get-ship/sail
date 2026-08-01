/**
 * Explorer VM — rail tabs (Routes / Files / Components) + sail-tree file nav.
 * One createTable identity; dataSig swaps per tab. Components host is separate.
 */
import { createTable } from '@ship/sail-primitives'
import { mountAmbient, signal } from '@ship/sail-runtime'
import { getShellState } from '@ship/sail-studio-shell'
import {
  fileSailTreeNodes,
  getEditState,
  openDocument,
  routeSailTreeNodes,
  setFilter,
  subscribeEdit,
  type SailTreeNode,
} from '@ship/studio/edit'
import { assignTreeHost } from '../edit/assign-tree'

type RailTab = 'routes' | 'files' | 'components'

function previewVp() {
  const shell = getShellState()
  return {
    width: shell.viewport,
    height: Math.round((shell.viewport * 9) / 16),
  }
}

/** Non-file tree rows (group/dir headers / route intermediates) — expand only. */
function isTreeMetaId(id: string): boolean {
  return (
    id.startsWith('group:') || id.startsWith('route:') || id.startsWith('dir:')
  )
}

export function createEditExplorer() {
  const filterText = signal('')
  const hasTree = signal(false)
  const dataSig = signal<SailTreeNode[]>([])
  const railTab = signal<RailTab>('routes')
  /** Last Routes/Files tab — keeps sail-tree data mounted while Components is up. */
  let lastTreeTab: 'routes' | 'files' = 'routes'
  const treeLabel = signal('Routes')
  const emptyTitle = signal('No routes')
  const emptyBody = signal('Open a project to list route sources.')
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

  const applyTreeCopy = (tab: 'routes' | 'files') => {
    if (tab === 'routes') {
      treeLabel.set('Routes')
      emptyTitle.set('No routes')
      emptyBody.set('Open a project to list route sources.')
    } else {
      treeLabel.set('Project files')
      emptyTitle.set('No files')
      emptyBody.set('Open a project to list Sail sources.')
    }
  }

  const sync = () => {
    const es = getEditState()
    const dirty = new Set<string>()
    for (const [p, d] of es.docs) {
      if (d.source.dirty) dirty.add(p)
    }
    const tab = railTab()
    if (tab === 'routes' || tab === 'files') lastTreeTab = tab
    applyTreeCopy(lastTreeTab)
    // Routes: hierarchy of URL paths. Files: group → source paths (R3).
    const shown =
      lastTreeTab === 'routes'
        ? routeSailTreeNodes(es.files, es.filter, { dirtyPaths: dirty })
        : fileSailTreeNodes(es.files, es.filter, es.activePath, {
            dirtyPaths: dirty,
          })
    // Keep tree identity + collapse state while Components panel is visible.
    dataSig.set(shown)
    hasTree.set(
      shown.some((g) => (g.children?.length ?? 0) > 0) || shown.length > 0,
    )
    filterText.set(es.filter)
    syncSelection(es.activePath)
    // Tree host stays under data-rail-panel=tree (h:0 when Components active).
    queueMicrotask(() => assignTreeHost('[data-file-tree-host]', tree))
  }

  sync()

  mountAmbient(() => {
    const unsub = subscribeEdit(() => sync())
    queueMicrotask(() => assignTreeHost('[data-file-tree-host]', tree))
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
    // Group / intermediate route rows expand via sail-tree; restore active file.
    if (isTreeMetaId(id)) {
      syncSelection(getEditState().activePath)
      return
    }
    void openPath(id)
  }

  const onRailTabChange = (e: Event) => {
    const name = (e as CustomEvent<{ name?: string }>).detail?.name
    if (name === 'routes' || name === 'files' || name === 'components') {
      railTab.set(name)
      sync()
    }
  }

  return {
    filterText,
    hasTree,
    railTab,
    treeLabel,
    emptyTitle,
    emptyBody,
    isTreeView: () => railTab() !== 'components',
    isComponents: () => railTab() === 'components',
    onRailTabChange,
    onFilterInput,
    onTreeSelect,
  }
}
