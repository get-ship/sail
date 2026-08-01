/** Edit surface VM — tabs + view switcher + panels. */
import { mountAmbient, signal } from '@ship/sail-runtime'
import {
  closeTab,
  getShellState,
  setActiveTab,
  type DocTab,
} from '@ship/sail-studio-shell'
import {
  DOC_VIEWS,
  activeDoc,
  clearHoverTip,
  getEditState,
  hoverPreviewAt,
  openDocument,
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
  uiStructuredOps,
  writeSignalValue,
} from '@ship/studio/edit'
import { loadFindings, sourceForSave } from '@ship/studio/checks'
import { ensurePreviewOnce, paintRegions } from './preview-helpers'
type TabRow = DocTab & {
  active: boolean
  dirtyDot: string
  canClose: boolean
  showClose: boolean
  key: string
}
export function closeDocTab(closedId: string): string | null {
  const wasActive = getShellState().activeTabId === closedId
  closeTab(closedId)
  return wasActive ? getShellState().activeTabId : null
}
export async function openShellTabDocument(id: string): Promise<void> {
  setActiveTab(id)
  if (!id.endsWith('.sail')) return
  const s = getShellState()
  await openDocument(id, { loadPreview: false, theme: s.themeId, mode: s.modeId })
}
export function createEditSurface() {
  const stateCards = signal<
    {
      id: string
      name: string
      kind: string
      ssrTier: string
      tierLabel: string
      tierWhy: string
      readersLabel: string
      writersLabel: string
      value: string
      key: string
    }[]
  >([])
  const structOps = signal<{ id: string; label: string; disabled: boolean; key: string }[]>([])
  const sourceText = signal('')
  const sourceStatus = signal('—')
  const livePreviewSrc = signal('')
  const hasLivePreview = signal(false)
  const auditSrc = signal('')
  const hasAudit = signal(false)
  const editError = signal('')
  const hasEditError = signal(false)
  const checkRows = signal<{ line: number; severity: string; message: string; key: string }[]>([])
  const hasChecks = signal(false)
  const checksLabel = signal('')
  const hoverLabel = signal('')
  const hasHover = signal(false)
  const sourceScrollLine = signal(0)
  const sourceDirty = signal(false)
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
  let hoverTabId: string | null = null
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
  const trySourceScroll = (): boolean => {
    const es = getEditState()
    if (es.sourceScrollLine == null) return false
    return scrollSourceEditor(querySourceEditor())
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
    structOps.set(uiStructuredOps().map((o) => ({ ...o, key: o.id })))
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
    sourceText.set(doc?.source.text ?? '')
    sourceStatus.set(doc?.source.status ?? '—')
    sourceDirty.set(!!doc?.source.dirty)
    checksLabel.set(doc?.source.checksLabel ?? '')
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
    const png = doc?.preview.imagePng
    if (png) {
      auditSrc.set(png.startsWith('data:') ? png : `data:image/png;base64,${png}`)
      hasAudit.set(true)
    } else {
      auditSrc.set('')
      hasAudit.set(false)
    }
    hoverLabel.set(es.hoverTip.visible ? es.hoverTip.label : '')
    hasHover.set(es.hoverTip.visible)
    sourceScrollLine.set(es.sourceScrollLine ?? 0)
    editError.set(es.error ?? '')
    hasEditError.set(!!es.error)
    paintRegions()
    const realTabs = shell.tabs.filter((t) => t.id.endsWith('.sail') || t.id !== 'shell')
    const tabs = realTabs.length
      ? realTabs
      : es.activePath
        ? [
            {
              id: es.activePath,
              label: es.activePath.split('/').pop() ?? es.activePath,
              dirty: !!doc?.source.dirty,
            },
          ]
        : []
    docTabs.set(
      tabs.map((t) => {
        const active = t.id === (shell.activeTabId ?? es.activePath)
        const canClose = tabs.length > 1
        return {
          ...t,
          active,
          dirtyDot: t.dirty ? '●' : '',
          canClose,
          showClose: canClose && (active || hoverTabId === t.id),
          key: t.id,
        }
      }),
    )
    hasOpenDoc.set(!!es.activePath && !!doc)
    activePathLabel.set(es.activePath ?? '')
    // Consume scroll nonce only when the editor is mounted and scroll applied.
    if (es.sourceScrollNonce !== lastScrollNonce) {
      if (trySourceScroll()) lastScrollNonce = es.sourceScrollNonce
    } else if (viewChanged && es.view === 'source') {
      // Entering Source with a pending span (nonce already consumed or missed).
      queueMicrotask(() => {
        trySourceScroll()
        lastScrollNonce = getEditState().sourceScrollNonce
      })
    }
    ensurePreviewOnce(flight, () => sync())
  }
  sync()
  mountAmbient(() => {
    const unsub = subscribeEdit(() => sync())
    return () => unsub()
  })
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
  const onPreviewClick = (e: Event) => {
    const img = (e.target as Element | null)?.closest?.('[data-preview-img]') as HTMLElement | null
    if (!img) return
    const { x, y } = previewCoords(e as MouseEvent, img)
    if (selectFromPreviewClick(x, y)) sync()
  }
  /** Light hover path: tip + overlay only (no full surface rebuild). */
  const onPreviewMove = (e: Event) => {
    const img = (e.target as Element | null)?.closest?.('[data-preview-img]') as HTMLElement | null
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
  const onTabClick = (e: Event) => {
    const closeEl = (e.target as Element | null)?.closest?.('[data-tab-close]') as HTMLElement | null
    if (closeEl) {
      const host = closeEl.closest?.('[data-tab-id]') as HTMLElement | null
      if (!host?.dataset.tabId) return
      e.preventDefault()
      e.stopPropagation()
      const next = closeDocTab(host.dataset.tabId)
      if (next) void openShellTabDocument(next).then(() => sync())
      else sync()
      return
    }
    const tabEl = (e.target as Element | null)?.closest?.('[data-tab-id]') as HTMLElement | null
    if (!tabEl?.dataset.tabId) return
    void openShellTabDocument(tabEl.dataset.tabId).then(() => sync())
  }
  const onTabPointer = (e: Event) => {
    const tabEl = (e.target as Element | null)?.closest?.('[data-tab-id]') as HTMLElement | null
    if (e.type === 'mouseleave' || e.type === 'pointerleave') {
      if (tabEl?.dataset.tabId === hoverTabId || !tabEl) {
        hoverTabId = null
        sync()
      }
      return
    }
    if (!tabEl?.dataset.tabId || hoverTabId === tabEl.dataset.tabId) return
    hoverTabId = tabEl.dataset.tabId
    sync()
  }
  return {
    stateCards,
    structOps,
    sourceText,
    sourceStatus,
    livePreviewSrc,
    hasLivePreview,
    auditSrc,
    hasAudit,
    editError,
    hasEditError,
    checkRows,
    hasChecks,
    checksLabel,
    hoverLabel,
    hasHover,
    sourceScrollLine,
    sourceDirty,
    liveRouteLabel,
    docTabs,
    hasOpenDoc,
    activePathLabel,
    activeView,
    isPreview,
    isSource,
    isState,
    isLayout,
    isData,
    isA11y,
    stateJson,
    dataJson,
    layoutRows,
    a11yRows,
    onViewChange,
    onSourceSailChange,
    onSaveSource,
    onPreviewClick,
    onPreviewMove,
    onPreviewLeave,
    onLayoutClick,
    onSignalWrite,
    onTabClick,
    onTabPointer,
  }
}
