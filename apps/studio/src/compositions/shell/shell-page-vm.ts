/** Studio chrome VM — activity rail (real links) + status bar + ops boot. */
import { hotkeys, mountAmbient } from '@ship/sail-runtime'
import {
  attachShellHotkeys,
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
  subscribeSailThemeChange,
  subscribeShell,
  subscribeWindowMessages,
  toggleChecksPanel,
} from '@ship/sail-studio-shell'
import { bindEditSelection, createOpsClient, getEditState, openDocument, saveActiveSource, seedFilesFromPaths, setOpsClient, setProjectRoot, subscribeEdit } from '../../edit/index.ts'
import { getChecksState, loadFindings, setChecksOpsClient, setEditorSetting, setProjectRoot as setChecksProjectRoot, subscribeChecks } from '../../checks/index.ts'

import { signal, effect, navigate } from '@ship/sail-runtime'
function getChecksStateSafe(): ReturnType<typeof getChecksState> | null {
  try {
    return getChecksState()
  } catch {
    return null
  }
}
import { loadIconGlyphs, loadThemes, setDesignOpsClient, setDesignProjectRoot, setDesignMode } from '../../design/index.ts'
import { getTasksState, loadThreads, newTaskFromBatch, sendBatchToExisting, setTasksOpsClient, setTasksProjectRoot, subscribeTasks, uiBatchSummary, uiThreadRows } from '../../tasks/index.ts'
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
  const hasChecksFindings = signal(false)
  const hasFindingsErrors = signal(false)
  const hasFindingsWarnings = signal(false)
  const findingsErrorLabel = signal('0')
  const findingsWarningLabel = signal('0')
  const findingsStatusTone = signal('muted')
  const findingsStatusAria = signal('Findings · clean')
  const mod = hotkeyMod()

  const readPath = () => opts?.pathname?.() ?? '/edit'

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
            ? `Shot · ${Math.round(it.rect.width)}px`
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
    // Prefer checks store severities when mounted; shell badge is error floor
    const cs = getChecksStateSafe()
    const err = cs?.errorCount ?? s.checksBadge
    const warn = cs?.warningCount ?? 0
    hasFindingsErrors.set(err > 0)
    hasFindingsWarnings.set(warn > 0)
    hasChecksFindings.set(err + warn > 0)
    findingsErrorLabel.set(String(err))
    findingsWarningLabel.set(String(warn))
    findingsStatusTone.set(err > 0 ? 'destructive' : warn > 0 ? 'warning' : 'muted')
    findingsStatusAria.set(
      err + warn > 0
        ? `Findings · ${err} error${err === 1 ? '' : 's'} · ${warn} warning${warn === 1 ? '' : 's'}`
        : 'Findings · clean',
    )
    syncTasks()
  }

  const go = (href: string) => {
    navigate(href)
    sync(href)
  }

  sync()

  mountAmbient(() => {
    // Theme/mode via setThemeMode + host chrome (no documentElement writes).
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
    const unsubChecks = subscribeChecks(() => sync())
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
    const onBridgeData = (data: unknown) => {
      if (!isStudioMessage(data)) return
      const d = data
      if (d.type === 'select') selectFromBridge(d.nodeId ?? null, d.selector ?? null)
      if (d.type === 'annotate' && d.rect) void handleBridgeRegion('annotate', d.rect)
      if (d.type === 'screenshot' && d.rect) void handleBridgeRegion('screenshot', d.rect)
    }
    // Declarative shell hotkeys — framework owns document root target.
    const digitMap: Record<string, () => void> = {}
    for (const d of Object.keys(ACTIVITY_BY_DIGIT)) {
      digitMap[`mod+${d}`] = () => go(hrefForActivity(ACTIVITY_BY_DIGIT[d]!))
    }
    const stopHotkeys = attachShellHotkeys(
      {
        'mod+s': () => {
          if (getEditState().activePath) void saveActiveSource().then(() => sync())
        },
        'mod+shift+k': () => {
          toggleChecksPanel()
          sync()
        },
        ...digitMap,
        Escape: () => {
          if (getShellState().checksPanelOpen) {
            setChecksPanelOpen(false)
            sync()
            return
          }
          if (canExitStudio()) exitStudio()
        },
      },
      hotkeys,
    )
    const stopMessages = subscribeWindowMessages(onBridgeData)
    const stopThemeBus = subscribeSailThemeChange((d) => {
      setThemeMode(d.theme || 'studio', d.mode || 'dark')
      setDesignMode((d.mode as 'light' | 'dark') || 'dark')
      sync()
    })
    return () => {
      stopRoute()
      unsubShell()
      unsubSel()
      unsubBatch()
      unsubEditSel()
      unsubEdit()
      unsubTasks()
      unsubChecks()
      stopHotkeys()
      stopMessages()
      stopThemeBus()
    }
  })

  const onThemeChange = (detail?: { theme?: string; mode?: string }) => {
    setThemeMode(detail?.theme || 'studio', detail?.mode || 'dark')
    setDesignMode((detail?.mode as 'light' | 'dark') || 'dark')
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
    hasChecksFindings,
    hasFindingsErrors,
    hasFindingsWarnings,
    findingsErrorLabel,
    findingsWarningLabel,
    findingsStatusTone,
    findingsStatusAria,
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
    onRailClick: (e: {
      preventDefault(): void
      button?: number
      metaKey?: boolean
      ctrlKey?: boolean
      shiftKey?: boolean
      altKey?: boolean
      target: { closest?: (s: string) => { getAttribute?: (n: string) => string | null; href?: string } | null } | null
    }) => {
      if ((e.button ?? 0) !== 0 || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return
      const a = e.target?.closest?.('a[activity]')
      const href = a?.getAttribute?.('href') ?? a?.href
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
