/** Studio chrome VM — activity rail (real links) + status bar + ops boot. */
import { effect, mountAmbient, signal } from '@ship/sail-runtime'
import { navigate } from '@ship/deck/router'
import {
  activities,
  bindBatchToShell,
  bindSelectionToShell,
  getBatchItems,
  getShellState,
  handleBridgeRegion,
  isStudioMessage,
  selectFromBridge,
  setBadges,
  setSailPath,
  setThemeMode,
  subscribeShell,
  type ActivityId,
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
  previewInApp,
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

const CORE_LABELS: Record<string, string> = {
  edit: 'Edit',
  checks: 'Checks',
  design: 'Design',
  tasks: 'Tasks',
}
const RAIL_ICONS: Record<string, string> = {
  edit: 'square-pen',
  checks: 'list-checks',
  design: 'palette',
  tasks: 'list-todo',
}

export type RailRow = {
  id: ActivityId
  label: string
  icon: string
  href: string
  active: boolean
  key: string
}
type BatchChipUi = { id: string; label: string; key: string }

function railLabel(id: string): string {
  return CORE_LABELS[id] ?? id.charAt(0).toUpperCase() + id.slice(1)
}

/** Multi-app mount base (e.g. `/_studio`) so rail links stay in-app. */
export function appRouteBase(): string {
  if (typeof document === 'undefined') return ''
  const raw = document
    .querySelector('meta[name="deck-app-route"]')
    ?.getAttribute('content')
    ?.trim()
  if (!raw || raw === '/') return ''
  return raw.replace(/\/$/, '')
}

/** Pathname relative to app base for rail active state. */
export function pathForActivity(pathname: string): string {
  const base = appRouteBase()
  if (!base) return pathname || '/'
  if (pathname === base) return '/'
  if (pathname.startsWith(base + '/')) {
    const rest = pathname.slice(base.length)
    return rest.startsWith('/') ? rest : `/${rest}`
  }
  return pathname || '/'
}

/** Map path → activity id. `/` and `/edit` both mean edit. */
export function activityFromPath(pathname: string): ActivityId {
  const p = pathForActivity(pathname)
  if (p === '/checks' || p.startsWith('/checks/')) return 'checks'
  if (p === '/design' || p.startsWith('/design/')) return 'design'
  if (p === '/tasks' || p.startsWith('/tasks/')) return 'tasks'
  return 'edit'
}

export function hrefForActivity(id: ActivityId): string {
  const base = appRouteBase()
  const leaf = id === 'edit' ? '/edit' : `/${id}`
  return `${base}${leaf}`
}

/** Pure rail rows for a pathname — unit-tested without DOM navigation. */
export function railRowsForPath(pathname: string): RailRow[] {
  const active = activityFromPath(pathname)
  return (['edit', 'checks', 'design', 'tasks'] as ActivityId[]).map((id) => ({
    id,
    label: railLabel(id),
    icon: RAIL_ICONS[id] || 'box',
    href: hrefForActivity(id),
    active: active === id,
    key: id,
  }))
}

function healthTone(h: 'ok' | 'degraded' | 'down'): string {
  return h === 'ok' ? 'success' : h === 'degraded' ? 'warning' : 'destructive'
}
const SERVICE_OK = { ok: 'Service', degraded: 'Service slow', down: 'Service down' } as const
const LIVE_OK = { ok: 'Live', degraded: 'Lagging', down: 'Offline' } as const

export type ShellPageOpts = {
  /**
   * Live layout pathname (from deck-injected `route.pathname`).
   * When chrome is reused across leaf navigations, this must be reactive —
   * `popstate` alone never fires for SPA pushState.
   */
  pathname?: () => string
}

export function createShellPage(opts?: ShellPageOpts) {
  const rail = signal<RailRow[]>([])
  const statusLine = signal('Ready')
  const hasBatch = signal(false)
  const batchChips = signal<BatchChipUi[]>([])
  const batchNewLabel = signal('New task (0)')
  const opsHealthLabel = signal('Service')
  const watchHealthLabel = signal('Live')
  const opsHealthTone = signal('success')
  const watchHealthTone = signal('success')

  const readPath = () =>
    opts?.pathname?.() ??
    (typeof location !== 'undefined' ? location.pathname : '/edit')

  const syncRail = (pathname?: string) => {
    rail.set(railRowsForPath(pathname ?? readPath()))
  }

  const syncTasks = () => {
    const items = getBatchItems()
    batchChips.set(
      items.map((it) => ({
        id: it.id,
        label:
          it.kind === 'screenshot'
            ? `Screenshot · ${Math.round(it.rect.w)}px`
            : it.nodes[0]?.label ?? `Selection · ${it.id.slice(0, 8)}`,
        key: it.id,
      })),
    )
    batchNewLabel.set(uiBatchSummary().label)
    setBadges(getShellState().checksBadge, uiThreadRows().length)
  }

  const sync = () => {
    const s = getShellState()
    syncRail()
    const sel = s.selection.nodeId
    const selShort = sel ? sel.split('#').pop() || sel : null
    statusLine.set(
      [s.devUrl || 'Studio', selShort ? `selected ${selShort}` : null]
        .filter(Boolean)
        .join(' · ') || 'Ready',
    )
    opsHealthLabel.set(SERVICE_OK[s.opsHealth])
    watchHealthLabel.set(LIVE_OK[s.watchHealth])
    opsHealthTone.set(healthTone(s.opsHealth))
    watchHealthTone.set(healthTone(s.watchHealth))
    hasBatch.set(s.batchCount > 0)
    syncTasks()
  }

  sync()

  mountAmbient(() => {
    if (typeof document !== 'undefined') {
      // Studio workbench theme (warm charcoal + coral) — dogfood via design surface.
      document.documentElement.setAttribute('data-theme', 'studio')
      document.documentElement.setAttribute('data-mode', 'dark')
    }
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
    // Seed immediately so the explorer is never a blank panel.
    seedFilesFromPaths(projectPaths(root))
    setBadges(0, 0)
    const unsubShell = subscribeShell(sync)
    const unsubSel = bindSelectionToShell()
    const unsubBatch = bindBatchToShell()
    const unsubEditSel = bindEditSelection()
    const unsubTasks = subscribeTasks(() => sync())
    // Track live layout pathname so rail active flips on SPA leaf navigations
    // (pushState does not fire popstate; chrome is reused, not remounted).
    const stopRoute = effect(() => {
      readPath()
      syncRail()
    })
    const shell = getShellState()
    // Live preview for page routes — dogfood studio nests itself; skip
    // iframe when the target project is studio (use audit / host site instead).
    const loadPreview =
      root !== 'apps/studio' && !root.endsWith('/apps/studio')
    void (async () => {
      let files = projectPaths(root)
      try {
        if (ops.sailAuthoringListFiles) {
          const listed = await ops.sailAuthoringListFiles({ project: root })
          if (listed.ok && listed.files?.length) files = listed.files
        }
      } catch {
        // Keep seed paths.
      }
      seedFilesFromPaths(files)
      sync()
      // Prefer a mid-size doc over a huge marketing home (Model can hang).
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
        // Explorer still has files; user can open another path.
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
      if ((e.metaKey || e.ctrlKey) && e.key === 's') {
        const es = getEditState()
        if (es.view === 'source' && es.activePath) {
          e.preventDefault()
          void saveActiveSource().then(() => sync())
        }
      }
      if (e.altKey && (e.key === 'p' || e.key === 'P')) {
        if (activityFromPath(readPath()) === 'design') {
          e.preventDefault()
          previewInApp()
          sync()
        }
      }
      // Escape leaves Studio — no dead-end when overlay or embedded.
      if (e.key === 'Escape' && canExitStudio() && !e.metaKey && !e.ctrlKey && !e.altKey) {
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
      unsubTasks()
      if (typeof window !== 'undefined') {
        window.removeEventListener('message', onMsg)
        window.removeEventListener('keydown', onKey)
        window.removeEventListener('sail-theme-change', onThemeChange)
      }
    }
  })

  const onBatchNewTask = () => {
    void newTaskFromBatch().then(() => {
      navigate(hrefForActivity('tasks'))
      sync()
    })
  }

  const onBatchSendExisting = () => {
    const id = getTasksState().selectedId
    if (!id) return
    void sendBatchToExisting(id).then(() => sync())
  }

  const onThemeChange = (e: Event) => {
    const detail = (e as CustomEvent).detail as { theme?: string; mode?: string } | undefined
    const themeId = detail?.theme || 'studio'
    const modeId = detail?.mode || 'dark'
    setThemeMode(themeId, modeId)
    setDesignMode((modeId as 'light' | 'dark') || 'dark')
    sync()
  }

  /** Test/helper: force rail resync for a path (also used when pathname opts change). */
  const setRailPath = (pathname: string) => {
    syncRail(pathname)
  }

  return {
    rail,
    statusLine,
    hasBatch,
    batchChips,
    batchNewLabel,
    opsHealthLabel,
    watchHealthLabel,
    opsHealthTone,
    watchHealthTone,
    showExit: canExitStudio(),
    onExitStudio: () => exitStudio(),
    onThemeChange,
    onBatchNewTask,
    onBatchSendExisting,
    setRailPath,
    syncRail,
  }
}
