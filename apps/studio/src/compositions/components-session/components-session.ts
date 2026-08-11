/**
 * Components page session — mode, selection, toast, open-edit, thread, brief.
 * Domain modules register via onSync; no hand-rolled listener Sets as product stores.
 */
import {
  mountAmbient,
  navigate,
  signal,
} from '../../../../../packages/sail/runtime/src/index.ts'
import { getEditState, openDocument, requestSourceScroll } from '../../edit/index.ts'
import { createThread } from '../../tasks/index.ts'
import { getShellState } from '../../../../../packages/sail/studio-shell/src/index.ts'
import { buildAgentBrief, buildNewComponentBrief } from '../../components/brief'
import type {
  ComponentMode,
  ComponentRecord,
  DetailTabId,
} from '../../components/types'
import {
  readComponentsActivity,
  writeComponentsActivity,
} from './components-activity'
import {
  componentHref,
  componentIdFromLocation,
  componentsHomeHref,
  editActivityHref,
  modeFromLocation,
  tasksActivityHref,
  themesComponentsHref,
} from './components-nav'

export type ToastSlot = { key: string; message: string; tone: string }

export type ComponentsSession = ReturnType<typeof createComponentsSession>

export type ComponentsSessionOpts = {
  /** Surface-fixed mode when library/inspect are separate route mounts. */
  mode?: ComponentMode
}

/**
 * Programmatic clipboard write for session actions (copy brief).
 * Prefer `use:clipboard` on click hosts; this path is for non-click handlers.
 * Returns false when the Clipboard API is unavailable.
 */
async function writeClipboard(text: string): Promise<boolean> {
  try {
    // Clipboard API is the platform primitive behind use:clipboard.
    const clip = globalThis.navigator?.clipboard
    if (clip?.writeText) {
      await clip.writeText(text)
      return true
    }
  } catch {
    /* offline / denied */
  }
  return false
}

export function createComponentsSession(opts?: ComponentsSessionOpts) {
  const fixedMode = opts?.mode
  const bag = readComponentsActivity()
  const mode = signal<ComponentMode>(fixedMode ?? 'library')
  // Preserve selection/filter across Library ↔ Inspect remounts (SPEC §6.1).
  const selectedId = signal(bag.selectedId)
  const filterQuery = signal(bag.filterQuery)
  const detailTab = signal<DetailTabId>('overview')
  const isLibrary = signal((fixedMode ?? 'library') === 'library')
  const isInspect = signal((fixedMode ?? 'library') === 'inspect')

  const toastItems = signal<ToastSlot[]>([])
  let toastSeq = 0

  const showNewForm = signal(false)
  const newName = signal('')
  const newIntent = signal('')
  const newLayer = signal('app')

  // Domain registration (session fan-out only — not a product store)
  const domainSyncs = new Set<() => void>()

  const setNote = (msg: string, tone = 'muted') => {
    if (!msg) {
      toastItems.set([])
      return
    }
    toastSeq += 1
    toastItems.set([{ key: `toast-${toastSeq}`, message: msg, tone }])
  }

  const onToastDismiss = () => {
    toastItems.set([])
  }

  const notifyDomain = () => {
    for (const fn of domainSyncs) fn()
  }

  const onSync = (fn: () => void) => {
    domainSyncs.add(fn)
    return () => {
      domainSyncs.delete(fn)
    }
  }

  const applyMode = (m: ComponentMode, opts?: { navigate?: boolean }) => {
    mode.set(m)
    isLibrary.set(m === 'library')
    isInspect.set(m === 'inspect')
    if (opts?.navigate !== false) {
      const id = selectedId()
      navigate(id ? componentHref(id, m) : componentsHomeHref(m))
    }
    notifyDomain()
  }

  const setSelectedId = (id: string, opts?: { navigate?: boolean; replace?: boolean }) => {
    selectedId.set(id)
    writeComponentsActivity({ selectedId: id })
    if (opts?.navigate !== false) {
      const m = mode()
      if (id) navigate(componentHref(id, m))
      else navigate(componentsHomeHref(m))
    }
    notifyDomain()
  }

  const syncFromRoute = () => {
    const m = fixedMode ?? modeFromLocation()
    const id = componentIdFromLocation()
    mode.set(m)
    isLibrary.set(m === 'library')
    isInspect.set(m === 'inspect')
    // Route :id wins; else keep bag/session selection (mode switch preserve).
    if (id && id !== 'inspect') {
      selectedId.set(id)
      writeComponentsActivity({ selectedId: id })
    }
    notifyDomain()
  }

  const openInEdit = async (path: string, span?: { line?: number; col?: number }) => {
    if (!path || path === '—' || path.includes('@ship/')) {
      setNote('No project path to open in Edit.', 'warning')
      return false
    }
    const shell = getShellState()
    try {
      const doc = await openDocument(path, {
        loadPreview: false,
        theme: shell.themeId,
        mode: shell.modeId,
      })
      if (!doc) {
        setNote('Could not open in Edit — try again.', 'destructive')
        return false
      }
      if (span?.line && span.line > 0) {
        requestSourceScroll(span.line, span.col ?? 1)
      }
      navigate(editActivityHref())
      return true
    } catch {
      setNote('Could not open in Edit — try again.', 'destructive')
      return false
    }
  }

  const startThread = async (record: ComponentRecord | null) => {
    if (!record) {
      setNote('Select a component first.', 'warning')
      return null
    }
    const title = `Component: ${record.displayName || record.id}`
    const body = buildAgentBrief({ record })
    const id = await createThread({
      title,
      body,
      anchor: {
        componentId: record.id,
        path: record.path,
        origin: record.origin,
        package: record.package,
      },
    })
    if (!id) {
      setNote('Could not start a thread — tasks may be offline.', 'warning')
      return null
    }
    setNote('Thread started.', 'success')
    navigate(tasksActivityHref())
    return id
  }

  const copyAgentBrief = async (record: ComponentRecord | null) => {
    if (!record) {
      setNote('Select a component first.', 'warning')
      return false
    }
    const text = buildAgentBrief({ record })
    const ok = await writeClipboard(text)
    setNote(ok ? 'Brief copied' : 'Could not copy brief', ok ? 'success' : 'warning')
    return ok
  }

  const onFilterInput = (e: Event) => {
    const q = (e.target as HTMLInputElement).value ?? ''
    filterQuery.set(q)
    writeComponentsActivity({ filterQuery: q })
    notifyDomain()
  }

  const onModeLibrary = () => applyMode('library')
  const onModeInspect = () => applyMode('inspect')

  /** Closed-over mode — no dataset bus (plain `mode=` is not data-*). */
  const onModeToggle = (m: 'library' | 'inspect') => {
    if (m === 'library' || m === 'inspect') applyMode(m)
  }

  const setDetailTab = (tab: DetailTabId) => {
    detailTab.set(tab)
  }

  const onDetailTab = (e: Event) => {
    const name = (e as CustomEvent<{ value?: string; name?: string }>).detail?.value
      ?? (e as CustomEvent<{ name?: string }>).detail?.name
      ?? (e.target as HTMLElement | null)?.getAttribute?.('value')
      ?? (e.target as HTMLElement | null)?.getAttribute?.('name')
    if (name === 'overview' || name === 'contract' || name === 'health' || name === 'theme') {
      detailTab.set(name)
    }
  }

  const onNewOpen = () => {
    showNewForm.set(true)
    detailTab.set('overview')
    notifyDomain()
  }

  const onNewNameInput = (e: Event) => {
    newName.set((e.target as HTMLInputElement).value ?? '')
  }
  const onNewIntentInput = (e: Event) => {
    newIntent.set((e.target as HTMLTextAreaElement).value ?? '')
  }
  const onNewLayerChange = (e: Event) => {
    newLayer.set((e.target as HTMLSelectElement).value ?? 'app')
  }

  const onCreateBrief = async () => {
    const text = buildNewComponentBrief({
      name: newName(),
      intent: newIntent(),
      layer: newLayer(),
    })
    const ok = await writeClipboard(text)
    setNote(
      ok ? 'Draft brief copied — scaffold is not written to disk yet.' : 'Could not copy brief',
      ok ? 'success' : 'warning',
    )
  }

  const onStartThreadFromNew = async () => {
    const name = newName().trim() || 'new-component'
    const record: ComponentRecord = {
      id: name,
      displayName: name,
      origin: 'project',
      package: '—',
      path: '',
      layer: newLayer() === 'L2' ? 'L2' : 'app',
      description: newIntent(),
      props: [],
      slots: [],
      issueCount: 0,
      ssrTierSummary: 'unknown',
      hasPreview: false,
      threadCount: 0,
      canOpenInEdit: false,
    }
    await startThread(record)
  }

  const goThemesComponents = () => {
    navigate(themesComponentsHref(getShellState().themeId || 'studio'))
  }

  const activeEditPath = () => getEditState().activePath

  syncFromRoute()

  mountAmbient(() => {
    // Route sync on mount; popstate needs a deck route signal (DOMDEC gap).
    syncFromRoute()
    return () => {}
  })

  return {
    mode,
    selectedId,
    filterQuery,
    detailTab,
    isLibrary,
    isInspect,
    toastItems,
    showNewForm,
    newName,
    newIntent,
    newLayer,
    setNote,
    onToastDismiss,
    onSync,
    notifyDomain,
    applyMode,
    setSelectedId,
    syncFromRoute,
    openInEdit,
    startThread,
    copyAgentBrief,
    onFilterInput,
    onModeLibrary,
    onModeInspect,
    onModeToggle,
    setDetailTab,
    onDetailTab,
    onNewOpen,
    onNewNameInput,
    onNewIntentInput,
    onNewLayerChange,
    onCreateBrief,
    onStartThreadFromNew,
    goThemesComponents,
    activeEditPath,
  }
}
