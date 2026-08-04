/** Studio chrome VM — activity rail (real links) + status bar + ops boot. */
import { effect, mountAmbient, signal } from '@ship/sail-runtime'
import { navigate } from '@ship/deck/router'
import {
  bindBatchToShell,
  bindSelectionToShell,
  getBatchItems,
  getShellState,
  handleBridgeRegion,
  isStudioMessage,
  selectFromBridge,
  setBadges,
  setChecksPanelOpen,
  setSailPath,
  setThemeMode,
  subscribeShell,
  toggleChecksPanel,
} from '@ship/sail-studio-shell'
import {
  bindEditSelection,
  createOpsClient,
  getEditState,
  openDocument,
  saveActiveSource,
  seedFilesFromPaths,
  setOpsClient,
  setProjectRoot,
  subscribeEdit,
} from '@ship/studio/edit'
import {
  loadFindings,
  setChecksOpsClient,
  setEditorSetting,
  setProjectRoot as setChecksProjectRoot,
} from '@ship/studio/checks'
import {
  loadIconGlyphs,
  loadThemes,
  setDesignOpsClient,
  setDesignProjectRoot,
  setDesignMode,
} from '@ship/studio/design'
import {
  getTasksState,
  loadThreads,
  newTaskFromBatch,
  sendBatchToExisting,
  setTasksOpsClient,
  setTasksProjectRoot,
  subscribeTasks,
  uiBatchSummary,
  uiThreadRows,
} from '@ship/studio/tasks'
import { canExitStudio, exitStudio } from './exit'
import { projectPaths, resolveProjectRoot } from './project'
export {
  ACTIVITY_BY_DIGIT,
  activityFromPath,
  appRouteBase,
  hotkeyMod,
  hrefForActivity,
  pathForActivity,
  railLabel,
  railRowsForPath,
  type RailRow,
} from './shell-rail'
import {
  ACTIVITY_BY_DIGIT,
  hrefForActivity,
  hotkeyMod,
  pathForActivity,
  railRowsForPath,
  type RailRow,
} from './shell-rail'

type BatchChipUi = { id: string; label: string; key: string }

function healthTone(h: 'ok' | 'degraded' | 'down'): string {
  return h === 'ok' ? 'success' : h === 'degraded' ? 'warning' : 'destructive'
}
const SERVICE_OK = { ok: 'Service', degraded: 'Service slow', down: 'Service down' } as const
const LIVE_OK = { ok: 'Live', degraded: 'Lagging', down: 'Offline' } as const

/** Live layout pathname (deck `route.pathname`) — reactive across SPA leaf nav. */
export type ShellPageOpts = { pathname?: () => string }

export function createShellPage(opts?: ShellPageOpts) {
  const rail = signal<RailRow[]>([])
  const statusLine = signal('Ready')
  const themeStatusLabel = signal('studio · dark')
  const saveStatusLabel = signal('Saved')
  const hasBatch = signal(false)
  const batchCountLabel = signal('0')
  const batchChips = signal<BatchChipUi[]>([])
  const batchNewLabel = signal('New task (0)')
  const opsHealthLabel = signal('Service')
  const watchHealthLabel = signal('Live')
  const opsHealthTone = signal('success')
  const watchHealthTone = signal('success')
  const checksPanelOpen = signal(false)
  const checksBadgeLabel = signal('Checks')
  const checksBadgeTone = signal('muted')
  const hasChecksFindings = signal(false)
  const mod = hotkeyMod()

  const readPath = () =>
    opts?.pathname?.() ??
    (typeof location !== 'undefined' ? location.pathname : '/edit')

  const syncRail = (pathname?: string) => {
    rail.set(railRowsForPath(pathname ?? readPath(), mod))
  }

  const syncTasks = () => {
    const items = getBatchItems()
    batchChips.set(
      items.map((it) => ({
        id: it.id,
        label:
          it.kind === 'screenshot'
            ? `Shot · ${Math.round(it.rect.w)}px`
            : it.nodes[0]?.label ?? `Pick · ${it.id.slice(0, 6)}`,
        key: it.id,
      })),
    )
    batchCountLabel.set(String(items.length))
    batchNewLabel.set(uiBatchSummary().label)
    setBadges(getShellState().checksBadge, uiThreadRows().length)
  }

  const sync = (pathOverride?: string) => {
    const path = pathOverride ?? readPath()
    syncRail(path)
    const s = getShellState()
    const sel = s.selection.nodeId
    const selShort = sel ? sel.split('#').pop() || sel : null
    const es = getEditState()
    const dirty =
      es.activePath && es.docs.get(es.activePath)?.source.dirty ? 'Unsaved' : 'Saved'
    const route = pathForActivity(path).replace(/^\//, '') || 'edit'
    statusLine.set(
      [s.devUrl || 'Studio', route, selShort ? `sel ${selShort}` : null]
        .filter(Boolean)
        .join(' · ') || 'Ready',
    )
    themeStatusLabel.set(`${s.themeId} · ${s.modeId}`)
    saveStatusLabel.set(dirty)
    opsHealthLabel.set(SERVICE_OK[s.opsHealth])
    watchHealthLabel.set(LIVE_OK[s.watchHealth])
    opsHealthTone.set(healthTone(s.opsHealth))
    watchHealthTone.set(healthTone(s.watchHealth))
    hasBatch.set(s.batchCount > 0)
    checksPanelOpen.set(s.checksPanelOpen)
    const n = s.checksBadge
    hasChecksFindings.set(n > 0)
    checksBadgeLabel.set(n > 0 ? `Checks · ${n}` : 'Checks')
    checksBadgeTone.set(n > 0 ? 'destructive' : 'muted')
    syncTasks()
  }

  const go = (href: string) => {
    navigate(href)
    sync(href)
  }

  sync()

  mountAmbient(() => {
    // Theme/mode attrs: setThemeMode below + host chrome — no documentElement writes
    // (sail-no-ambient-dom). SAIL-LINT-CHAT if shell still needs data-theme on :root
    // without theme-select / tokens path.
    const ops = createOpsClient()
    setOpsClient(ops)
    setChecksOpsClient(ops)
    setDesignOpsClient(ops)
    setTasksOpsClient(ops)
    const root = resolveProjectRoot()
    setProjectRoot(root)
    setThemeMode('studio', 'dark')
    setChecksProjectRoot(root)
    setDesignProjectRoot(root)
    setDesignMode(getShellState().modeId === 'light' ? 'light' : 'dark')
    setTasksProjectRoot(root)
    setEditorSetting('vscode')
    seedFilesFromPaths(projectPaths(root))
    setBadges(0, 0)
    const unsubShell = subscribeShell(() => sync())
    const unsubSel = bindSelectionToShell()
    const unsubBatch = bindBatchToShell()
    const unsubEditSel = bindEditSelection()
    const unsubEdit = subscribeEdit(() => sync())
    const unsubTasks = subscribeTasks(() => sync())
    // Live pathname: full sync so status route + rail stay aligned with SPA leaf.
    const stopRoute = effect(() => {
      const p = readPath()
      sync(p)
    })
    const shell = getShellState()
    const loadPreview = root !== 'apps/studio' && !root.endsWith('/apps/studio')
    void (async () => {
      let files = projectPaths(root)
      try {
        if (ops.sailAuthoringListFiles) {
          const listed = await ops.sailAuthoringListFiles({ project: root })
          if (listed.ok && listed.files?.length) files = listed.files
        }
      } catch {
        /* seed */
      }
      seedFilesFromPaths(files)
      sync()
      const openPath =
        files.find((f) => /\/getting-started\/page\.sail$/.test(f)) ||
        files.find((f) => /\/routes\/page\.sail$/.test(f)) ||
        files[0] ||
        `${root}/routes/page.sail`
      setSailPath(openPath)
      try {
        await Promise.race([
          openDocument(openPath, {
            loadPreview,
            theme: shell.themeId,
            mode: shell.modeId,
          }),
          new Promise<null>((_, rej) =>
            setTimeout(() => rej(new Error('open timeout')), 20000),
          ),
        ])
        void loadFindings(openPath).then(() => sync())
      } catch {
        /* seeded explorer */
      }
      sync()
    })()
    void loadThemes().then(() => {
      void loadIconGlyphs().then(() => sync())
      sync()
    })
    void loadThreads().then(() => sync())
    sync()
    const onMsg = (e: MessageEvent) => {
      if (!isStudioMessage(e.data)) return
      const d = e.data
      if (d.type === 'select') selectFromBridge(d.nodeId ?? null, d.selector ?? null)
      if (d.type === 'annotate' && d.rect) void handleBridgeRegion('annotate', d.rect)
      if (d.type === 'screenshot' && d.rect) void handleBridgeRegion('screenshot', d.rect)
    }
    const onKey = (e: KeyboardEvent) => {
      const isMod = e.metaKey || e.ctrlKey
      // Always swallow browser Save-page; save only when a doc is open.
      if (isMod && e.key === 's') {
        e.preventDefault()
        if (getEditState().activePath) void saveActiveSource().then(() => sync())
        return
      }
      // Ctrl/⌘+Shift+K — not Shift+C (Chromium Inspect Element).
      if (isMod && e.shiftKey && (e.key === 'k' || e.key === 'K')) {
        e.preventDefault()
        toggleChecksPanel()
        sync()
        return
      }
      if (isMod && ACTIVITY_BY_DIGIT[e.key]) {
        e.preventDefault()
        go(hrefForActivity(ACTIVITY_BY_DIGIT[e.key]))
        return
      }
      if (e.key === 'Escape' && getShellState().checksPanelOpen) {
        e.preventDefault()
        setChecksPanelOpen(false)
        sync()
        return
      }
      if (e.key === 'Escape' && canExitStudio() && !isMod && !e.altKey) {
        e.preventDefault()
        exitStudio()
      }
    }
    if (typeof window !== 'undefined') {
      window.addEventListener('message', onMsg)
      window.addEventListener('keydown', onKey)
      window.addEventListener('sail-theme-change', onThemeChange)
    }
    return () => {
      stopRoute()
      unsubShell()
      unsubSel()
      unsubBatch()
      unsubEditSel()
      unsubEdit()
      unsubTasks()
      if (typeof window !== 'undefined') {
        window.removeEventListener('message', onMsg)
        window.removeEventListener('keydown', onKey)
        window.removeEventListener('sail-theme-change', onThemeChange)
      }
    }
  })

  const onThemeChange = (e: Event) => {
    const d = (e as CustomEvent).detail as { theme?: string; mode?: string } | undefined
    setThemeMode(d?.theme || 'studio', d?.mode || 'dark')
    setDesignMode((d?.mode as 'light' | 'dark') || 'dark')
    sync()
  }

  return {
    rail,
    statusLine,
    themeStatusLabel,
    saveStatusLabel,
    hasBatch,
    batchCountLabel,
    batchChips,
    batchNewLabel,
    opsHealthLabel,
    watchHealthLabel,
    opsHealthTone,
    watchHealthTone,
    checksPanelOpen,
    checksBadgeLabel,
    checksBadgeTone,
    hasChecksFindings,
    showExit: canExitStudio(),
    onExitStudio: () => exitStudio(),
    onThemeChange,
    onToggleChecksPanel: () => {
      toggleChecksPanel()
      sync()
    },
    onCloseChecksPanel: () => {
      setChecksPanelOpen(false)
      sync()
    },
    onBatchNewTask: () => {
      void newTaskFromBatch().then(() => go(hrefForActivity('tasks')))
    },
    onBatchSendExisting: () => {
      const id = getTasksState().selectedId
      if (id) void sendBatchToExisting(id).then(() => sync())
    },
    onRailClick: (e: Event) => {
      const me = e as MouseEvent
      if (me.button !== 0 || me.metaKey || me.ctrlKey || me.shiftKey || me.altKey) return
      const a = (e.target as Element | null)?.closest?.('a[data-activity]') as HTMLAnchorElement | null
      const href = a?.getAttribute('href')
      if (!href) return
      e.preventDefault()
      // Themes dirty leave guard (session registers while a facet is open).
      void import('../theme-session/theme-leave-bridge').then((m) => {
        const proceed = m.requestThemeLeaveNavigate(href)
        if (proceed) go(href)
      })
    },
    setRailPath: (pathname: string) => syncRail(pathname),
    syncRail,
  }
}
