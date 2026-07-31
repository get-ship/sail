/**
 * Explorer VM — collapsible dir groups + keyboard file nav (aria-activedescendant).
 * Ops openDocument on file activate; shell seeds ops + paths.
 */
import { mountAmbient, signal } from '@ship/sail-runtime'
import { getShellState } from '@ship/sail-studio-shell'
import {
  fileTreeGroups,
  getEditState,
  nextTreeFocus,
  openDocument,
  pathDomId,
  setFilter,
  subscribeEdit,
} from '@ship/studio/edit'

type FileRowUi = {
  path: string
  name: string
  /** Right-aligned meta (extension chip). */
  metaLabel: string
  checksLabel: string
  hasChecks: boolean
  selected: boolean
  focused: boolean
  domId: string
  tabIndex: number
  key: string
}

function fileExt(name: string): string {
  const i = name.lastIndexOf('.')
  if (i <= 0) return 'file'
  return name.slice(i + 1).toLowerCase()
}

/** Human size for explorer meta (reference: `index.html · 13.1 KB`). */
function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  return `${(bytes / 1024).toFixed(1)} KB`
}

/**
 * Right-aligned row meta: extension, plus source size when the doc is open.
 * Age/mtime not available without a list API — size from open source is best-effort.
 */
function fileMetaLabel(name: string, sourceLen: number | null): string {
  const ext = fileExt(name)
  if (sourceLen == null || sourceLen <= 0) return ext
  return `${ext} · ${formatSize(sourceLen)}`
}

type TreeGroupUi = {
  id: string
  label: string
  key: string
  files: FileRowUi[]
}

function previewVp() {
  const shell = getShellState()
  return {
    width: shell.viewport,
    height: Math.round((shell.viewport * 9) / 16),
  }
}

export function createEditExplorer() {
  const treeGroups = signal<TreeGroupUi[]>([])
  const hasFiles = signal(false)
  const filterText = signal('')
  /** aria-activedescendant id for the focused treeitem. */
  const activeDescendant = signal('')
  /** Keyboard focus path (may differ from active open file). */
  let kbdPath: string | null = null

  const flatPaths = (): string[] =>
    treeGroups()
      .flatMap((g) => g.files)
      .map((f) => f.path)

  const sync = () => {
    const es = getEditState()
    if (!kbdPath || !es.files.some((f) => f.path === kbdPath)) {
      kbdPath = es.activePath
    }
    const groups = fileTreeGroups(es.files, es.filter, es.activePath).map((g) => ({
      id: g.id,
      label: g.label,
      key: g.id,
      files: g.files.map((f) => {
        const focused = f.path === kbdPath
        const doc = es.docs.get(f.path)
        const sourceLen = doc?.source?.text?.length ?? null
        return {
          path: f.path,
          name: f.name,
          metaLabel: fileMetaLabel(f.name, sourceLen),
          checksLabel: f.checksLabel,
          hasChecks: f.hasChecks,
          selected: f.selected,
          focused,
          domId: pathDomId(f.path),
          // Roving tabindex: focused item is tabbable; others -1.
          tabIndex: focused ? 0 : -1,
          key: f.path,
        }
      }),
    }))
    treeGroups.set(groups)
    hasFiles.set(groups.some((g) => g.files.length > 0))
    filterText.set(es.filter)
    activeDescendant.set(kbdPath ? pathDomId(kbdPath) : '')
  }

  sync()

  mountAmbient(() => {
    const unsub = subscribeEdit(() => sync())
    return () => unsub()
  })

  const openPath = (path: string) => {
    kbdPath = path
    const shell = getShellState()
    return openDocument(path, {
      loadPreview: false,
      theme: shell.themeId,
      mode: shell.modeId,
      viewport: previewVp(),
    }).then(() => sync())
  }

  const focusItemDom = (path: string | null) => {
    if (!path || typeof document === 'undefined') return
    const el = document.getElementById(pathDomId(path)) as HTMLElement | null
    el?.focus?.()
  }

  const onFilterInput = (e: Event) => {
    setFilter((e.target as HTMLInputElement).value)
    sync()
  }

  const onFileClick = (e: Event) => {
    const t = (e.target as Element | null)?.closest?.('[data-file-path]') as HTMLElement | null
    if (!t?.dataset.filePath) return
    void openPath(t.dataset.filePath)
  }

  const onTreeKeydown = (e: Event) => {
    const ke = e as KeyboardEvent
    const paths = flatPaths()
    if (!paths.length) return
    if (ke.key === 'Enter' || ke.key === ' ') {
      ke.preventDefault()
      if (kbdPath) void openPath(kbdPath)
      return
    }
    const next = nextTreeFocus(paths, kbdPath, ke.key)
    if (next == null || next === kbdPath) {
      if (ke.key === 'ArrowDown' || ke.key === 'ArrowUp' || ke.key === 'Home' || ke.key === 'End') {
        ke.preventDefault()
      }
      return
    }
    if (ke.key === 'ArrowDown' || ke.key === 'ArrowUp' || ke.key === 'Home' || ke.key === 'End') {
      ke.preventDefault()
      kbdPath = next
      sync()
      // Roving tabindex: move DOM focus onto the focused treeitem.
      queueMicrotask(() => focusItemDom(next))
    }
  }

  return {
    treeGroups,
    hasFiles,
    filterText,
    activeDescendant,
    onFilterInput,
    onFileClick,
    onTreeKeydown,
  }
}
