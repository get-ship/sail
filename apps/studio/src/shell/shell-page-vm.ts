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

/** Digit → activity; Ctrl/Cmd+digit navigates. */
export const ACTIVITY_BY_DIGIT: Record<string, ActivityId> = {
  '1': 'edit',
  '2': 'checks',
  '3': 'design',
  '4': 'tasks',
}

const RAIL: Record<string, { label: string; icon: string; digit: string }> = {
  edit: { label: 'Edit', icon: 'square-pen', digit: '1' },
  checks: { label: 'Checks', icon: 'list-checks', digit: '2' },
  design: { label: 'Design', icon: 'palette', digit: '3' },
  tasks: { label: 'Tasks', icon: 'list-todo', digit: '4' },
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

/** Platform mod glyph for tooltips — ⌘ on Apple, Ctrl+ elsewhere. */
export function hotkeyMod(): string {
  if (typeof navigator === 'undefined') return 'Ctrl+'
  const p =
    (navigator as { userAgentData?: { platform?: string } }).userAgentData?.platform ||
    navigator.platform ||
    ''
  return /mac|iphone|ipad|ipod/i.test(p) ? '⌘' : 'Ctrl+'
}

export function railLabel(id: string, mod = hotkeyMod()): string {
  const r = RAIL[id]
  if (r) return `${r.label} ${mod}${r.digit}`
  return id.charAt(0).toUpperCase() + id.slice(1)
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
export function railRowsForPath(pathname: string, mod = hotkeyMod()): RailRow[] {
  const active = activityFromPath(pathname)
  return (['edit', 'checks', 'design', 'tasks'] as ActivityId[]).map((id) => ({
    id,
    label: railLabel(id, mod),
    icon: RAIL[id]?.icon || 'box',
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
    syncTasks()
  }

  sync()

  mountAmbient(() => {
    if (typeof document !== 'undefined') {
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
      // Dual-pane edit: do not gate save on es.view (default is still 'preview').
      if (isMod && e.key === 's' && getEditState().activePath) {
        e.preventDefault()
        void saveActiveSource().then(() => sync())
        return
      }
      if (isMod && ACTIVITY_BY_DIGIT[e.key]) {
        e.preventDefault()
        const href = hrefForActivity(ACTIVITY_BY_DIGIT[e.key])
        navigate(href)
        sync(href) // target path — avoid stale route.pathname
        return
      }
      if (e.altKey && (e.key === 'p' || e.key === 'P') && activityFromPath(readPath()) === 'design') {
        e.preventDefault()
        previewInApp()
        sync()
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
    showExit: canExitStudio(),
    onExitStudio: () => exitStudio(),
    onThemeChange,
    onBatchNewTask: () => {
      void newTaskFromBatch().then(() => {
        const href = hrefForActivity('tasks')
        navigate(href)
        sync(href)
      })
    },
    onBatchSendExisting: () => {
      const id = getTasksState().selectedId
      if (id) void sendBatchToExisting(id).then(() => sync())
    },
    setRailPath: (pathname: string) => syncRail(pathname),
    syncRail,
  }
}
