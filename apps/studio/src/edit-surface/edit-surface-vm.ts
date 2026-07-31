/**
 * Edit surface VM — editor tabs + source + live preview + opt-in debug rail.
 * Explorer/element-tree are sibling components. Shell owns ops seed.
 */
import { mountAmbient, signal } from '@ship/sail-runtime'
import {
  closeTab,
  getShellState,
  setActiveTab,
  type DocTab,
} from '@ship/sail-studio-shell'
import {
  activeDoc,
  clearHoverTip,
  getEditState,
  hoverPreviewAt,
  pathToLiveRoute,
  refreshPreview,
  saveActiveSource,
  scrollSourceEditor,
  selectEditNode,
  selectFromPreviewClick,
  setSourceTextFromEditor,
  subscribeEdit,
  uiCheckDiags,
  uiChildren,
  uiStateCards,
  uiStructuredOps,
  openDocument,
} from '@ship/studio/edit'
import { loadFindings, sourceForSave } from '@ship/studio/checks'

type StateCardUi = {
  id: string
  name: string
  kind: string
  ssrTier: string
  tierLabel: string
  tierWhy: string
  readersLabel: string
  writersLabel: string
  key: string
}
type ChildRowUi = { id: string; tag: string; layer: string; key: string }
type StructOpUi = { id: string; label: string; disabled: boolean; key: string }
type CheckUi = { line: number; severity: string; message: string; key: string }
type TabRow = DocTab & {
  active: boolean
  dirtyDot: string
  canClose: boolean
  showClose: boolean
  key: string
}

/** Close tab; if active, return promoted id to open (else null). */
export function closeDocTab(closedId: string): string | null {
  const wasActive = getShellState().activeTabId === closedId
  closeTab(closedId)
  return wasActive ? getShellState().activeTabId : null
}

/** Activate a doc tab and load source when the id is a .sail path. */
export async function openShellTabDocument(id: string): Promise<void> {
  setActiveTab(id)
  if (!id.endsWith('.sail')) return
  const s = getShellState()
  await openDocument(id, { loadPreview: false, theme: s.themeId, mode: s.modeId })
}

export function createEditSurface() {
  const stateCards = signal<StateCardUi[]>([])
  const childRows = signal<ChildRowUi[]>([])
  const structOps = signal<StructOpUi[]>([])
  const sourceText = signal('')
  const sourceStatus = signal('—')
  const livePreviewSrc = signal('')
  const hasLivePreview = signal(false)
  const auditSrc = signal('')
  const hasAudit = signal(false)
  const showAudit = signal(false)
  const showDebug = signal(false)
  const editError = signal('')
  const hasEditError = signal(false)
  const checkRows = signal<CheckUi[]>([])
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
  /** Tab id under pointer — × only on hover (or always when active). */
  let hoverTabId: string | null = null
  let lastScrollNonce = -1

  const sync = () => {
    const es = getEditState()
    const shell = getShellState()
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
        key: c.id,
      })),
    )
    childRows.set(
      uiChildren().map((c) => ({
        id: c.id,
        tag: `<${c.tag}>`,
        layer: c.layer,
        key: c.id,
      })),
    )
    structOps.set(
      uiStructuredOps().map((o) => ({
        id: o.id,
        label: o.label,
        disabled: o.disabled,
        key: o.id,
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
    const doc = activeDoc()
    sourceText.set(doc?.source.text ?? '')
    sourceStatus.set(doc?.source.status ?? '—')
    sourceDirty.set(!!doc?.source.dirty)
    checksLabel.set(doc?.source.checksLabel ?? '')
    const route = pathToLiveRoute(es.activePath)
    // Never iframe the studio app into itself (dogfood nesting).
    const root = (es.projectRoot || '').replace(/\/$/, '')
    const isStudioDogfood =
      root === 'apps/studio' || root.endsWith('/apps/studio')
    if (route && !isStudioDogfood) {
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
    // Editor tabs: only real .sail docs (drop the bootstrap "shell" placeholder).
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
          // VSCode: × on hover; always visible on the active tab when closable.
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
  }

  sync()

  mountAmbient(() => {
    const unsub = subscribeEdit(() => sync())
    return () => unsub()
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
      if (changed && text !== doc.source.text) {
        setSourceTextFromEditor(text)
      }
    }
    void saveActiveSource().then(() => {
      if (path) void loadFindings(path).then(() => sync())
      else sync()
    })
  }

  const onAuditSnapshot = () => {
    const shell = getShellState()
    showAudit.set(true)
    const w = shell.viewport
    void refreshPreview({
      theme: shell.themeId,
      mode: shell.modeId,
      viewport: { width: w, height: Math.round((w * 9) / 16) },
    }).then(() => sync())
  }

  const onToggleDebug = (e: Event) => {
    const el = e.target as HTMLInputElement
    showDebug.set(!!el.checked)
  }

  const previewCoords = (e: MouseEvent, img: HTMLElement) => {
    const rect = img.getBoundingClientRect()
    const vp = getShellState().viewport
    const vw = vp
    const vh = Math.round((vp * 9) / 16)
    const sx = rect.width > 0 ? vw / rect.width : 1
    const sy = rect.height > 0 ? vh / rect.height : 1
    return {
      x: (e.clientX - rect.left) * sx,
      y: (e.clientY - rect.top) * sy,
    }
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

  const onElementClick = (e: Event) => {
    const t = (e.target as Element | null)?.closest?.('[data-node-id]') as HTMLElement | null
    if (!t?.dataset.nodeId) return
    selectEditNode(t.dataset.nodeId)
    sync()
    const ta =
      typeof document !== 'undefined'
        ? (document.querySelector('[data-source-editor]') as HTMLTextAreaElement | null)
        : null
    scrollSourceEditor(ta)
  }

  const onHideAudit = () => {
    showAudit.set(false)
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
      // Only clear if leaving the tab strip host for this id.
      if (tabEl?.dataset.tabId === hoverTabId || !tabEl) {
        hoverTabId = null
        sync()
      }
      return
    }
    if (!tabEl?.dataset.tabId) return
    if (hoverTabId === tabEl.dataset.tabId) return
    hoverTabId = tabEl.dataset.tabId
    sync()
  }

  return {
    stateCards,
    childRows,
    structOps,
    sourceText,
    sourceStatus,
    livePreviewSrc,
    hasLivePreview,
    auditSrc,
    hasAudit,
    showAudit,
    showDebug,
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
    onSourceSailChange,
    onSaveSource,
    onAuditSnapshot,
    onToggleDebug,
    onPreviewClick,
    onPreviewMove,
    onPreviewLeave,
    onElementClick,
    onHideAudit,
    onTabClick,
    onTabPointer,
  }
}
