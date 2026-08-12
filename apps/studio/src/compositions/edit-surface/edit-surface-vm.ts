/** Edit surface VM — doc tabs + source editor (VS Code baseline). */
import { type SailHost, mountAmbient } from '../../vendor/runtime'
import { getShellState, pushSourceEditorDiagnostics } from '../../vendor/studio-shell'
import { activeDoc, getEditState, pathToLiveRoute, querySourceEditor, saveActiveSource, scrollSourceEditor, setSourceEditorHost, setSourceTextFromEditor, setView, subscribeEdit, uiCheckDiags } from '../../edit/index.ts'
import { loadFindings, sourceForSave } from '../../checks/index.ts'
import { signal, effect } from '../../vendor/runtime'
import type { UiEvent } from '../ui-event'
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

/**
 * Ops HTTP base for <sail-code-editor ops-url>.
 * Matches `createOpsClient()` / studio-client default: same-origin (empty base).
 * The editor treats `"/"` as same-origin after normalize (trailing slash strip).
 */
export const STUDIO_OPS_URL = '/'

export function createEditSurface() {
  const sourceText = signal('')
  const sourceStatus = signal('—')
  const editError = signal('')
  const hasEditError = signal(false)
  const sourceScrollLine = signal(0)
  const sourceDirty = signal(false)
  const isSourceReady = signal(false)
  const isSourceEmpty = signal(false)
  const isSourceLoading = signal(false)
  const liveRouteLabel = signal('')
  const docTabs = signal<TabRow[]>([])
  const hasOpenDoc = signal(false)
  const activePathLabel = signal('')
  /** Host ref via `use:captureEditor` — no ambient document.querySelector. */
  const editorEl = signal<SailHost | null>(null)
  /** Same base as createOpsClient() — relative same-origin SailAuthoring. */
  const opsUrl = signal(STUDIO_OPS_URL)
  /** Active document path for Hover/Complete/diagnostics. */
  const opsPath = signal('')
  let lastScrollNonce = -1
  let lastView: string | null = null

  const trySourceScroll = (): boolean =>
    getEditState().sourceScrollLine == null ? false : scrollSourceEditor(querySourceEditor())

  const pushSourceDiagnostics = () => {
    pushSourceEditorDiagnostics(
      querySourceEditor(),
      uiCheckDiags().map((c) => ({
        line: c.line,
        ...(c.column != null ? { column: c.column } : {}),
        ...(c.endLine != null ? { endLine: c.endLine } : {}),
        ...(c.endColumn != null ? { endColumn: c.endColumn } : {}),
        severity:
          c.severity === 'warning' || c.severity === 'info'
            ? (c.severity as 'warning' | 'info')
            : 'error',
        message: c.message,
        source: 'check',
      })),
    )
  }

  const syncSourceFields = (doc: ReturnType<typeof activeDoc>, loading: boolean) => {
    const text = doc?.source.text ?? ''
    sourceText.set(text)
    sourceStatus.set(loading ? 'Loading…' : (doc?.source.status ?? '—'))
    sourceDirty.set(!!doc?.source.dirty)
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
    opsPath.set(es.activePath ?? '')

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
    const stopEd = effect(() => {
      setSourceEditorHost(editorEl())
    })
    return () => {
      unsub()
      stopEd()
      setSourceEditorHost(null)
    }
  })

  const onSourceSailChange = (detail: { value?: string }) => {
    const v = typeof detail?.value === 'string' ? detail.value : ''
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

  const onTabClick = (e: UiEvent) => {
    // Tab-close control — structural closest only (not a SailHost mount).
    if (e.target?.closest?.('[tab-close]')) {
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

  const onTabMiddle = (e: UiEvent) => {
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
    sourceScrollLine,
    sourceDirty,
    isSourceReady,
    isSourceEmpty,
    isSourceLoading,
    liveRouteLabel,
    docTabs,
    hasOpenDoc,
    activePathLabel,
    opsUrl,
    opsPath,
    editorEl,
    onSourceSailChange,
    onSaveSource,
    onTabClick,
    onTabMiddle,
  }
}
