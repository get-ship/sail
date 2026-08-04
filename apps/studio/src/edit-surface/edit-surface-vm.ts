/** Edit surface VM — doc tabs + source editor (VS Code baseline). */
import { effect, mountAmbient, signal } from '@ship/sail-runtime'
import { getShellState } from '@ship/sail-studio-shell'
import {
  activeDoc,
  getEditState,
  pathToLiveRoute,
  querySourceEditor,
  saveActiveSource,
  scrollSourceEditor,
  setSourceEditorHost,
  setSourceTextFromEditor,
  setView,
  subscribeEdit,
  uiCheckDiags,
} from '@ship/studio/edit'
import { loadFindings, sourceForSave } from '@ship/studio/checks'
import {
  closeDocTab,
  isMiddleButton,
  mapDocTabs,
  openShellTabDocument,
  resolveDocTabs,
  tabIdFromEvent,
  type TabRow,
} from './edit-surface-tabs'

export { closeDocTab, openShellTabDocument, mapDocTabs, resolveDocTabs } from './edit-surface-tabs'

export function createEditSurface() {
  const sourceText = signal('')
  const sourceStatus = signal('—')
  const editError = signal('')
  const hasEditError = signal(false)
  const checkRows = signal<{ line: number; severity: string; message: string; key: string }[]>([])
  const hasChecks = signal(false)
  const checksLabel = signal('')
  const sourceScrollLine = signal(0)
  const sourceDirty = signal(false)
  const isSourceReady = signal(false)
  const isSourceEmpty = signal(false)
  const isSourceLoading = signal(false)
  const liveRouteLabel = signal('')
  const docTabs = signal<TabRow[]>([])
  const hasOpenDoc = signal(false)
  const activePathLabel = signal('')
  /** Host ref via `bind:this` — no ambient document.querySelector. */
  const editorEl = signal<HTMLElement | null>(null)
  let lastScrollNonce = -1
  let lastView: string | null = null

  const trySourceScroll = (): boolean =>
    getEditState().sourceScrollLine == null ? false : scrollSourceEditor(querySourceEditor())

  const pushSourceDiagnostics = () => {
    if (typeof document === 'undefined') return
    const host = querySourceEditor() as
      | (HTMLElement & {
          setDiagnostics?: (
            d: { line: number; severity: 'error' | 'warning' | 'info'; message: string }[],
          ) => void
        })
      | null
    if (!host || typeof host.setDiagnostics !== 'function') return
    host.setDiagnostics(
      uiCheckDiags().map((c) => ({
        line: c.line,
        severity:
          c.severity === 'warning' || c.severity === 'info'
            ? (c.severity as 'warning' | 'info')
            : 'error',
        message: c.message,
      })),
    )
  }

  const syncSourceFields = (doc: ReturnType<typeof activeDoc>, loading: boolean) => {
    const text = doc?.source.text ?? ''
    sourceText.set(text)
    sourceStatus.set(loading ? 'Loading…' : (doc?.source.status ?? '—'))
    sourceDirty.set(!!doc?.source.dirty)
    checksLabel.set(doc?.source.checksLabel ?? '')
    // Ready only when we have text and are not mid-open — avoids empty/error flash.
    isSourceLoading.set(loading && !text)
    isSourceReady.set(!!text && !loading)
    isSourceEmpty.set(!!doc && !text && !loading)
  }

  const sync = () => {
    const es = getEditState()
    const shell = getShellState()
    if (es.view !== 'source') setView('source')
    const viewChanged = lastView !== 'source'
    lastView = 'source'

    checkRows.set(
      uiCheckDiags().map((c, i) => ({
        line: c.line,
        severity: c.severity,
        message: c.message,
        key: `${c.line}-${i}`,
      })),
    )
    hasChecks.set(uiCheckDiags().length > 0)
    const doc = activeDoc()
    syncSourceFields(doc, !!es.loading)
    const route = pathToLiveRoute(es.activePath)
    liveRouteLabel.set(route ? (route === '/' ? '/' : route) : '')
    sourceScrollLine.set(es.sourceScrollLine ?? 0)
    editError.set(es.error ?? '')
    hasEditError.set(!!es.error)

    const dirtyById = new Map<string, boolean>()
    for (const [p, d] of es.docs) dirtyById.set(p, !!d.source.dirty)
    const tabs = resolveDocTabs(
      shell.tabs,
      shell.activeTabId,
      es.activePath,
      !!doc?.source.dirty,
    )
    docTabs.set(mapDocTabs(tabs, shell.activeTabId ?? es.activePath, dirtyById))
    hasOpenDoc.set(!!es.activePath && !!doc)
    activePathLabel.set(es.activePath ?? '')

    if (es.sourceScrollNonce !== lastScrollNonce) {
      if (trySourceScroll()) lastScrollNonce = es.sourceScrollNonce
      pushSourceDiagnostics()
    } else if (viewChanged) {
      queueMicrotask(() => {
        if (trySourceScroll()) lastScrollNonce = getEditState().sourceScrollNonce
        pushSourceDiagnostics()
      })
    } else {
      pushSourceDiagnostics()
    }
  }

  sync()
  mountAmbient(() => {
    const unsub = subscribeEdit(() => sync())
    // Register source editor host from template bind:this — legal ref path.
    const stopEd = effect(() => {
      setSourceEditorHost(editorEl())
    })
    return () => {
      unsub()
      stopEd()
      setSourceEditorHost(null)
    }
  })

  const onSourceSailChange = (e: Event) => {
    const ce = e as CustomEvent<{ value?: string }>
    const v =
      typeof ce.detail?.value === 'string'
        ? ce.detail.value
        : String((e.target as { value?: string })?.value ?? '')
    setSourceTextFromEditor(v)
    sync()
  }

  const onSaveSource = () => {
    const path = getEditState().activePath
    const doc = activeDoc()
    if (path && doc) {
      const { text, changed } = sourceForSave(path, doc.source.text)
      if (changed && text !== doc.source.text) setSourceTextFromEditor(text)
    }
    void saveActiveSource().then(() => {
      if (path) void loadFindings(path).then(() => sync())
      else sync()
    })
  }

  const promoteAfterClose = (next: string | null) => {
    if (next) void openShellTabDocument(next).then(() => sync())
    else sync()
  }

  const onTabClick = (e: Event) => {
    const closeEl = (e.target as Element | null)?.closest?.(
      '[data-tab-close]',
    ) as HTMLElement | null
    if (closeEl) {
      const id = tabIdFromEvent(e)
      if (!id) return
      e.preventDefault()
      e.stopPropagation()
      promoteAfterClose(closeDocTab(id))
      return
    }
    const id = tabIdFromEvent(e)
    if (!id) return
    void openShellTabDocument(id).then(() => sync())
  }

  const onTabMiddle = (e: Event) => {
    if (!isMiddleButton(e)) return
    const id = tabIdFromEvent(e)
    if (!id) return
    e.preventDefault()
    promoteAfterClose(closeDocTab(id))
  }

  return {
    sourceText,
    sourceStatus,
    editError,
    hasEditError,
    checkRows,
    hasChecks,
    checksLabel,
    sourceScrollLine,
    sourceDirty,
    isSourceReady,
    isSourceEmpty,
    isSourceLoading,
    liveRouteLabel,
    docTabs,
    hasOpenDoc,
    activePathLabel,
    editorEl,
    onSourceSailChange,
    onSaveSource,
    onTabClick,
    onTabMiddle,
  }
}
