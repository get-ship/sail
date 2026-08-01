/** Edit surface VM — tabs + view switcher + panels. */
import { mountAmbient, signal } from '@ship/sail-runtime'
import { getShellState } from '@ship/sail-studio-shell'
import {
  DOC_VIEWS,
  activeDoc,
  clearHoverTip,
  getEditState,
  hoverPreviewAt,
  pathToLiveRoute,
  querySourceEditor,
  saveActiveSource,
  scrollSourceEditor,
  selectEditNode,
  selectFromPreviewClick,
  setSourceTextFromEditor,
  setView,
  subscribeEdit,
  type DocViewId,
  uiA11yRows,
  uiCheckDiags,
  uiDataJson,
  uiLayoutRows,
  uiStateCards,
  uiStateJson,
  writeSignalValue,
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
import {
  ensurePreviewOnce,
  paintRegions,
  regionMetaLabel,
  retryPreviewOnce,
} from './preview-helpers'

export { closeDocTab, openShellTabDocument, mapDocTabs, resolveDocTabs } from './edit-surface-tabs'

export function createEditSurface() {
  type Card = {
    id: string; name: string; kind: string; ssrTier: string; tierLabel: string
    tierWhy: string; readersLabel: string; writersLabel: string; value: string; key: string
  }
  const stateCards = signal<Card[]>([])
  const sourceText = signal('')
  const sourceStatus = signal('—')
  const livePreviewSrc = signal('')
  const hasLivePreview = signal(false)
  const auditSrc = signal('')
  const hasAudit = signal(false)
  const isPreviewReady = signal(false)
  const isPreviewLoading = signal(false)
  const isPreviewEmpty = signal(false)
  const previewError = signal('')
  const hasPreviewError = signal(false)
  const previewProvenance = signal('Audit snapshot')
  const previewRegionsLabel = signal('')
  const hasPreviewRegions = signal(false)
  const editError = signal('')
  const hasEditError = signal(false)
  const checkRows = signal<{ line: number; severity: string; message: string; key: string }[]>([])
  const hasChecks = signal(false)
  const checksLabel = signal('')
  const hoverLabel = signal('')
  const hasHover = signal(false)
  const sourceScrollLine = signal(0)
  const sourceDirty = signal(false)
  const isSourceReady = signal(false)
  const isSourceEmpty = signal(false)
  const liveRouteLabel = signal('')
  const docTabs = signal<TabRow[]>([])
  const hasOpenDoc = signal(false)
  const activePathLabel = signal('')
  const activeView = signal<DocViewId>('preview')
  const isPreview = signal(true)
  const isSource = signal(false)
  const isState = signal(false)
  const isLayout = signal(false)
  const isData = signal(false)
  const isA11y = signal(false)
  const stateJson = signal('{}')
  const dataJson = signal('{}')
  const layoutRows = signal<{ id: string; line: string; selected: boolean; key: string }[]>([])
  const a11yRows = signal<
    { id: string; check: string; status: string; detail: string; tone: string; key: string }[]
  >([])
  let lastScrollNonce = -1
  const flight = { inflight: null as string | null, failedPath: null as string | null }
  let lastHoverNodeId = ''
  let lastView: DocViewId | null = null

  const applyViewFlags = (view: DocViewId) => {
    activeView.set(view)
    isPreview.set(view === 'preview')
    isSource.set(view === 'source')
    isState.set(view === 'state')
    isLayout.set(view === 'layout')
    isData.set(view === 'data')
    isA11y.set(view === 'a11y')
  }
  const trySourceScroll = (): boolean =>
    getEditState().sourceScrollLine == null ? false : scrollSourceEditor(querySourceEditor())
  /** Push authoring checks into CodeMirror lint gutter when the host is mounted. */
  const pushSourceDiagnostics = () => {
    if (typeof document === 'undefined') return
    if (getEditState().view !== 'source') return
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
  const syncSourceFields = (doc: ReturnType<typeof activeDoc>) => {
    const text = doc?.source.text ?? ''
    sourceText.set(text)
    sourceStatus.set(doc?.source.status ?? '—')
    sourceDirty.set(!!doc?.source.dirty)
    checksLabel.set(doc?.source.checksLabel ?? '')
    // Ready = has bytes; empty = open doc with no text (load miss). Dirty overlays ready.
    isSourceReady.set(!!text)
    isSourceEmpty.set(!!doc && !text)
  }
  const syncPreviewFields = (doc: ReturnType<typeof activeDoc>) => {
    const png = doc?.preview.imagePng
    const pErr = doc?.preview.error ?? ''
    const loading = !!doc?.preview.loading
    if (png) {
      auditSrc.set(png.startsWith('data:') ? png : `data:image/png;base64,${png}`)
      hasAudit.set(true)
      isPreviewReady.set(true)
    } else {
      auditSrc.set('')
      hasAudit.set(false)
      isPreviewReady.set(false)
    }
    previewError.set(pErr)
    hasPreviewError.set(!!pErr && !png)
    // Loading only when no frame yet (ready keeps prior frame if reloading).
    isPreviewLoading.set(loading && !png && !pErr)
    // Idle empty: open doc, nothing loading/errored/ready — distinct from loading.
    isPreviewEmpty.set(!!doc && !png && !pErr && !loading)
    previewProvenance.set(doc?.preview.provenance || 'Audit snapshot')
    const n = doc?.preview.regions?.length ?? 0
    const meta = regionMetaLabel(n)
    previewRegionsLabel.set(meta)
    hasPreviewRegions.set(!!meta)
  }

  const sync = () => {
    const es = getEditState()
    const shell = getShellState()
    const viewChanged = lastView !== es.view
    lastView = es.view
    applyViewFlags(es.view)
    stateCards.set(
      uiStateCards().map((c) => ({
        id: c.id,
        name: c.name,
        kind: c.kind,
        ssrTier: c.ssrTier,
        tierLabel: c.tierLabel,
        tierWhy: c.tierWhy,
        readersLabel: c.readersLabel,
        writersLabel: c.writersLabel,
        value: c.value ?? '',
        key: c.id,
      })),
    )
    checkRows.set(
      uiCheckDiags().map((c, i) => ({
        line: c.line,
        severity: c.severity,
        message: c.message,
        key: `${c.line}-${i}`,
      })),
    )
    hasChecks.set(uiCheckDiags().length > 0)
    layoutRows.set(
      uiLayoutRows(shell.viewport).map((r) => ({
        id: r.id,
        line: r.line,
        selected: r.selected,
        key: r.id,
      })),
    )
    a11yRows.set(
      uiA11yRows(shell.themeId, shell.modeId).map((r) => ({
        id: r.id,
        check: r.check,
        status: r.status,
        detail: r.detail,
        tone: r.status === 'pass' ? 'success' : r.status === 'warn' ? 'warning' : 'muted',
        key: r.id,
      })),
    )
    stateJson.set(uiStateJson())
    dataJson.set(uiDataJson())
    const doc = activeDoc()
    syncSourceFields(doc)
    const route = pathToLiveRoute(es.activePath)
    const root = (es.projectRoot || '').replace(/\/$/, '')
    const dogfood = root === 'apps/studio' || root.endsWith('/apps/studio')
    if (route && !dogfood) {
      livePreviewSrc.set(route)
      hasLivePreview.set(true)
      liveRouteLabel.set(route === '/' ? 'Live · /' : `Live · ${route}`)
    } else {
      livePreviewSrc.set('')
      hasLivePreview.set(false)
      liveRouteLabel.set('No live preview for this file')
    }
    syncPreviewFields(doc)
    hoverLabel.set(es.hoverTip.visible ? es.hoverTip.label : '')
    hasHover.set(es.hoverTip.visible)
    sourceScrollLine.set(es.sourceScrollLine ?? 0)
    editError.set(es.error ?? '')
    hasEditError.set(!!es.error)
    paintRegions()
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
    } else if (viewChanged && es.view === 'source') {
      queueMicrotask(() => {
        trySourceScroll()
        pushSourceDiagnostics()
        lastScrollNonce = getEditState().sourceScrollNonce
      })
    } else if (es.view === 'source') {
      pushSourceDiagnostics()
    }
    ensurePreviewOnce(flight, () => sync())
  }

  sync()
  mountAmbient(() => {
    const unsub = subscribeEdit(() => sync())
    return () => unsub()
  })

  const onRetryPreview = () => retryPreviewOnce(flight, () => sync())
  const onViewChange = (e: Event) => {
    const name = (e as CustomEvent<{ name?: string }>).detail?.name
    if (!name || !DOC_VIEWS.includes(name as DocViewId)) return
    setView(name as DocViewId)
    sync()
  }
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
  const previewCoords = (e: MouseEvent, img: HTMLElement) => {
    const rect = img.getBoundingClientRect()
    const vp = getShellState().viewport
    const sx = rect.width > 0 ? vp / rect.width : 1
    const sy = rect.height > 0 ? Math.round((vp * 9) / 16) / rect.height : 1
    return { x: (e.clientX - rect.left) * sx, y: (e.clientY - rect.top) * sy }
  }
  const previewImg = (e: Event) =>
    (e.target as Element | null)?.closest?.('[data-preview-img]') as HTMLElement | null
  const onPreviewClick = (e: Event) => {
    const img = previewImg(e)
    if (!img) return
    const { x, y } = previewCoords(e as MouseEvent, img)
    if (selectFromPreviewClick(x, y)) sync()
  }
  const onPreviewMove = (e: Event) => {
    const img = previewImg(e)
    if (!img) return
    const { x, y } = previewCoords(e as MouseEvent, img)
    const tip = hoverPreviewAt(x, y)
    if (tip.nodeId === lastHoverNodeId && tip.visible === hasHover()) return
    lastHoverNodeId = tip.nodeId
    hoverLabel.set(tip.visible ? tip.label : '')
    hasHover.set(tip.visible)
    paintRegions()
  }
  const onPreviewLeave = () => {
    lastHoverNodeId = ''
    clearHoverTip()
    hoverLabel.set('')
    hasHover.set(false)
    paintRegions()
  }
  const onLayoutClick = (e: Event) => {
    const t = (e.target as Element | null)?.closest?.('[data-layout-id]') as HTMLElement | null
    if (!t?.dataset.layoutId) return
    selectEditNode(t.dataset.layoutId)
    sync()
    scrollSourceEditor(querySourceEditor())
  }
  const onSignalWrite = (e: Event) => {
    const host = (e.target as Element | null)?.closest?.('[data-binding]') as HTMLElement | null
    const name = host?.dataset.binding
    if (!name) return
    const input = host.querySelector('input') as HTMLInputElement | null
    writeSignalValue(name, input?.value ?? (e.target as HTMLInputElement).value ?? '')
    sync()
  }
  const promoteAfterClose = (next: string | null) => {
    if (next) void openShellTabDocument(next).then(() => sync())
    else sync()
  }
  const onTabClick = (e: Event) => {
    const closeEl = (e.target as Element | null)?.closest?.('[data-tab-close]') as HTMLElement | null
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
  /** Middle-click closes a doc tab (STUDIO-0). */
  const onTabMiddle = (e: Event) => {
    if (!isMiddleButton(e)) return
    const id = tabIdFromEvent(e)
    if (!id) return
    e.preventDefault()
    promoteAfterClose(closeDocTab(id))
  }

  return {
    stateCards, sourceText, sourceStatus, livePreviewSrc, hasLivePreview, auditSrc, hasAudit,
    isPreviewReady, isPreviewLoading, isPreviewEmpty, previewError, hasPreviewError,
    previewProvenance, previewRegionsLabel, hasPreviewRegions,
    editError, hasEditError, checkRows,
    hasChecks, checksLabel, hoverLabel, hasHover, sourceScrollLine, sourceDirty,
    isSourceReady, isSourceEmpty, liveRouteLabel,
    docTabs, hasOpenDoc, activePathLabel, activeView, isPreview, isSource, isState, isLayout,
    isData, isA11y, stateJson, dataJson, layoutRows, a11yRows, onViewChange, onSourceSailChange,
    onSaveSource, onRetryPreview, onPreviewClick, onPreviewMove, onPreviewLeave, onLayoutClick,
    onSignalWrite, onTabClick, onTabMiddle,
  }
}
