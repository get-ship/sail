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
  refreshPreview,
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
import { paintRegionOverlay } from '../edit/region-overlay'
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
  let regionBoxes: import('../edit/region-overlay').RegionBoxUi[] = []
  let hoverTabId: string | null = null
  let lastScrollNonce = -1
  let lastPreviewPath: string | null = null
  const applyViewFlags = (view: DocViewId) => {
    activeView.set(view)
    isPreview.set(view === 'preview')
    isSource.set(view === 'source')
    isState.set(view === 'state')
    isLayout.set(view === 'layout')
    isData.set(view === 'data')
    isA11y.set(view === 'a11y')
  }
  const ensurePreview = () => {
    const es = getEditState()
    const doc = activeDoc()
    if (!es.activePath || !doc) return
    if (es.view !== 'preview' && es.view !== 'layout') return
    if (doc.preview.imagePng && lastPreviewPath === es.activePath) return
    lastPreviewPath = es.activePath
    const shell = getShellState()
    const w = shell.viewport
    void refreshPreview({
      theme: shell.themeId,
      mode: shell.modeId,
      viewport: { width: w, height: Math.round((w * 9) / 16) },
    }).then(() => sync())
  }
  const sync = () => {
    const es = getEditState()
    const shell = getShellState()
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
    layoutRows.set(uiLayoutRows(shell.viewport).map((r) => ({ id: r.id, line: r.line, selected: r.selected, key: r.id })))
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
    const vw = shell.viewport || 1440
    const vh = Math.round((vw * 9) / 16)
    const selected = es.selectionNodeId
    const hover = es.hoverTip.visible ? es.hoverTip.nodeId : null
    regionBoxes = (doc?.preview.regions ?? [])
      .filter((r) => r.w > 0 && r.h > 0)
      .map((r) => ({
        id: r.nodeId,
        left: `${((r.x / vw) * 100).toFixed(2)}%`,
        top: `${((r.y / vh) * 100).toFixed(2)}%`,
        width: `${((r.w / vw) * 100).toFixed(2)}%`,
        height: `${((r.h / vh) * 100).toFixed(2)}%`,
        selected: r.nodeId === selected || r.nodeId === hover,
        key: r.nodeId,
      }))
    queueMicrotask(() => paintRegionOverlay(regionBoxes))
    const realTabs = shell.tabs.filter((t) => t.id.endsWith('.sail') || t.id !== 'shell')
    const tabs = realTabs.length
      ? realTabs
      : es.activePath
        ? [{ id: es.activePath, label: es.activePath.split('/').pop() ?? es.activePath, dirty: !!doc?.source.dirty }]
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
    if (es.sourceScrollNonce !== lastScrollNonce) {
      lastScrollNonce = es.sourceScrollNonce
      const ta =
        typeof document !== 'undefined'
          ? (document.querySelector('[data-source-editor]') as HTMLTextAreaElement | null)
          : null
      if (ta) scrollSourceEditor(ta)
    }
    ensurePreview()
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
  const onPreviewMove = (e: Event) => {
    const img = (e.target as Element | null)?.closest?.('[data-preview-img]') as HTMLElement | null
    if (!img) return
    const { x, y } = previewCoords(e as MouseEvent, img)
    hoverPreviewAt(x, y)
    sync()
  }
  const onPreviewLeave = () => {
    clearHoverTip()
    sync()
  }
  const onLayoutClick = (e: Event) => {
    const t = (e.target as Element | null)?.closest?.('[data-layout-id]') as HTMLElement | null
    if (!t?.dataset.layoutId) return
    selectEditNode(t.dataset.layoutId)
    sync()
    const ta =
      typeof document !== 'undefined'
        ? (document.querySelector('[data-source-editor]') as HTMLTextAreaElement | null)
        : null
    scrollSourceEditor(ta)
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
