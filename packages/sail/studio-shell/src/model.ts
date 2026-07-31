
import {
  clearSelection,
  getSelection,
  selectFromBridge,
  selectNode,
  subscribeSelection,
  type Selection,
} from './selection'
import { publishFabVisible, readFabVisible } from './channel'
import {
  addRegionToBatch,
  getBatchItems,
  subscribeBatch,
  type BatchItem,
  type BatchKind,
} from './batch'
import type { BridgeRect } from './channel'

/** Built-in Studio activities (core). */
export type CoreActivityId = 'edit' | 'checks' | 'design' | 'tasks'

/**
 * Activity id on the rail: core ids or module-provided ids.
 * Modules never replace core ids.
 */
export type ActivityId = CoreActivityId | (string & {})

/** Additive rail entry from a loaded sail-module manifest. */
export type ModuleActivity = {
  id: string
  label: string
  icon?: string
  /** Served via shell, e.g. `/_studio/x/hello`. */
  route: string
  module: string
  /** True when loaded unsigned under Dev mode. */
  devUnsigned?: boolean
}

export type DocTab = {
  id: string
  label: string
  dirty: boolean
}

export type ViewportId = 390 | 768 | 1440

export type ShellState = {
  activity: ActivityId
  tabs: DocTab[]
  activeTabId: string | null
  viewport: ViewportId
  themeId: string
  modeId: string
  modelRev: number
  opsHealth: 'ok' | 'degraded' | 'down'
  watchHealth: 'ok' | 'degraded' | 'down'
  devUrl: string
  fabVisible: boolean
  autoFix: boolean
  selection: Selection
  checksBadge: number
  tasksBadge: number
  /** Last annotate/screenshot region nodes (chips under selection) */
  regionNodes: { nodeId: string; label: string }[]
  batchCount: number
  /** Active sail path for SailRender join (optional) */
  sailPath: string | null
  moduleActivities: ModuleActivity[]
  /** Active module surface route when a module activity is selected. */
  moduleRoute: string | null
  /** Loud banner when any loaded module is unsigned (dev-only). */
  moduleUnsignedBanner: boolean
}

type Listener = (s: ShellState) => void

const CORE_ACTIVITIES: CoreActivityId[] = ['edit', 'checks', 'design', 'tasks']

const CORE_SET = new Set<string>(CORE_ACTIVITIES)

let moduleActivities: ModuleActivity[] = []

// ── Chrome state <-> URL hash ────────────────────────────────────────────
// The shell's location IS state: `#/<activity>` or `#/<activity>/<tabId>`.
// Deep-linkable (an agent can hand a user `#/checks`), survives SPA HMR
// remounts and full reloads — a remount re-reads the hash instead of
// resetting to the default surface. Browser-only; tests run hashless.
const hasDom = () => typeof location !== 'undefined' && typeof history !== 'undefined'

function chromeFromHash(): { activity?: ActivityId; tabId?: string } {
  if (!hasDom()) return {}
  const m = /^#\/([^/]+)(?:\/(.+))?$/.exec(location.hash)
  if (!m) return {}
  const activity = decodeURIComponent(m[1]) as ActivityId
  const tabId = m[2] ? decodeURIComponent(m[2]) : undefined
  return { activity, tabId }
}

function writeChromeHash(): void {
  if (!hasDom()) return
  const parts = [state.activity]
  if (state.activeTabId && state.activeTabId !== 'shell') parts.push(state.activeTabId)
  const hash = `#/${parts.map(encodeURIComponent).join('/')}`
  if (location.hash !== hash) history.replaceState(null, '', hash)
}

const initialChrome = chromeFromHash()

let state: ShellState = {
  activity:
    initialChrome.activity && CORE_SET.has(initialChrome.activity)
      ? (initialChrome.activity as CoreActivityId)
      : 'edit',
  tabs: [{ id: 'shell', label: 'shell', dirty: false }],
  activeTabId: initialChrome.tabId ?? 'shell',
  viewport: 1440,
  themeId: 'ship',
  modeId: 'dark',
  modelRev: 0,
  opsHealth: 'ok',
  watchHealth: 'ok',
  devUrl: 'sail dev',
  fabVisible: true,
  autoFix: false,
  selection: getSelection(),
  checksBadge: 0,
  tasksBadge: 0,
  regionNodes: [],
  batchCount: 0,
  sailPath: null,
  moduleActivities: [],
  moduleRoute: null,
  moduleUnsignedBanner: false,
}

const listeners = new Set<Listener>()

function emit(): void {
  for (const fn of listeners) fn(state)
}

export function getShellState(): ShellState {
  return state
}

export function subscribeShell(fn: Listener): () => void {
  listeners.add(fn)
  return () => {
    listeners.delete(fn)
  }
}

/** Core + module activity ids currently on the rail. */
export function activities(): ActivityId[] {
  return [...CORE_ACTIVITIES, ...moduleActivities.map((a) => a.id)]
}

export function coreActivities(): CoreActivityId[] {
  return CORE_ACTIVITIES.slice()
}

export function getModuleActivities(): ModuleActivity[] {
  return moduleActivities.slice()
}

/**
 * Replace module rail entries (from `/sail.ops.v1.modules`).
 * Does not touch core activities. If the current activity was a removed
 * module, fall back to `edit` so Studio stays usable (containment).
 */
export function setModuleActivities(list: ModuleActivity[]): void {
  // Reject attempts to override core ids.
  moduleActivities = list.filter((a) => a.id && !CORE_SET.has(a.id))
  const banner = moduleActivities.some((a) => a.devUnsigned)
  const still =
    CORE_SET.has(state.activity) ||
    moduleActivities.some((a) => a.id === state.activity)
  const activity: ActivityId = still ? state.activity : 'edit'
  const mod = moduleActivities.find((a) => a.id === activity)
  state = {
    ...state,
    activity,
    moduleActivities: moduleActivities.slice(),
    moduleRoute: mod?.route ?? null,
    moduleUnsignedBanner: banner,
  }
  emit()
}

/** Clear module activities (kill module / tests). Core rail unchanged. */
export function clearModuleActivities(): void {
  setModuleActivities([])
}

export function isCoreActivity(id: string): id is CoreActivityId {
  return CORE_SET.has(id)
}

export function isModuleActivity(id: string): boolean {
  return moduleActivities.some((a) => a.id === id)
}

export function setActivity(id: ActivityId): void {
  if (CORE_SET.has(id)) {
    state = { ...state, activity: id, moduleRoute: null }
    writeChromeHash()
    emit()
    return
  }
  const mod = moduleActivities.find((a) => a.id === id)
  if (!mod) return
  state = { ...state, activity: id, moduleRoute: mod.route }
  writeChromeHash()
  emit()
}

export function openTab(tab: DocTab): void {
  const tabs = state.tabs.some((t) => t.id === tab.id)
    ? state.tabs.map((t) => (t.id === tab.id ? { ...t, ...tab } : t))
    : [...state.tabs, tab]
  state = { ...state, tabs, activeTabId: tab.id }
  writeChromeHash()
  emit()
}

export function closeTab(id: string): void {
  if (state.tabs.length <= 1) return
  const tabs = state.tabs.filter((t) => t.id !== id)
  let activeTabId = state.activeTabId
  if (activeTabId === id) activeTabId = tabs[tabs.length - 1]?.id ?? null
  state = { ...state, tabs, activeTabId }
  writeChromeHash()
  emit()
}

export function setActiveTab(id: string): void {
  if (!state.tabs.some((t) => t.id === id)) return
  state = { ...state, activeTabId: id }
  writeChromeHash()
  emit()
}

export function setTabDirty(id: string, dirty: boolean): void {
  state = {
    ...state,
    tabs: state.tabs.map((t) => (t.id === id ? { ...t, dirty } : t)),
  }
  emit()
}

export function setViewport(v: ViewportId): void {
  state = { ...state, viewport: v }
  emit()
}

export function setThemeMode(themeId: string, modeId: string): void {
  state = { ...state, themeId, modeId }
  emit()
}

export function setModelRev(rev: number): void {
  state = { ...state, modelRev: rev }
  emit()
}

export function setHealth(
  ops: ShellState['opsHealth'],
  watch: ShellState['watchHealth'],
): void {
  state = { ...state, opsHealth: ops, watchHealth: watch }
  emit()
}

export function setFabVisible(v: boolean): void {
  state = { ...state, fabVisible: v }
  publishFabVisible(v)
  emit()
}

/** Sync from bridge settings message without re-broadcast loop. */
export function applyFabVisibleFromBridge(v: boolean): void {
  if (state.fabVisible === v) return
  state = { ...state, fabVisible: v }
  emit()
}

export function setAutoFix(v: boolean): void {
  state = { ...state, autoFix: v }
  emit()
}

export function setBadges(checks: number, tasks: number): void {
  if (state.checksBadge === checks && state.tasksBadge === tasks) return
  state = { ...state, checksBadge: checks, tasksBadge: tasks }
  emit()
}

export function setSailPath(path: string | null): void {
  state = { ...state, sailPath: path }
  emit()
}

export function setRegionNodes(nodes: { nodeId: string; label: string }[]): void {
  state = { ...state, regionNodes: nodes }
  emit()
}

/** Wire selection store → shell snapshot (call once at shell mount). */
export function bindSelectionToShell(): () => void {
  return subscribeSelection((selection) => {
    state = { ...state, selection }
    emit()
  })
}

/** Wire batch tray count into shell snapshot. */
export function bindBatchToShell(): () => void {
  return subscribeBatch((items: BatchItem[]) => {
    state = { ...state, batchCount: items.length }
    emit()
  })
}

/** Handle bridge annotate/screenshot: resolve region → selection + tray. */
export async function handleBridgeRegion(
  kind: BatchKind,
  rect: BridgeRect,
): Promise<void> {
  const item = await addRegionToBatch(kind, rect, {
    sailPath: state.sailPath,
    viewport: { width: state.viewport, height: Math.round((state.viewport * 9) / 16) },
    theme: state.themeId,
    mode: state.modeId,
  })
  state = {
    ...state,
    regionNodes: item.nodes.map((n) => ({ nodeId: n.nodeId, label: n.label })),
    batchCount: getBatchItems().length,
  }
  emit()
  // Primary selection = first resolved node (shared everywhere).
  if (item.nodes[0]) {
    selectFromBridge(item.nodes[0].nodeId, item.nodes[0].selector)
  }
}

/** Bootstrap FAB from localStorage at mount. */
export function hydrateFabFromStorage(): void {
  const v = readFabVisible()
  state = { ...state, fabVisible: v }
  emit()
}

export { selectNode, selectFromBridge, clearSelection, getSelection }
export { getBatchItems, subscribeBatch, clearBatch, removeBatchItem } from './batch'
export type { BatchItem, BatchKind } from './batch'

export function __resetShellForTests(): void {
  moduleActivities = []
  state = {
    activity: 'edit',
    tabs: [{ id: 'shell', label: 'shell', dirty: false }],
    activeTabId: 'shell',
    viewport: 1440,
    themeId: 'ship',
    modeId: 'dark',
    modelRev: 0,
    opsHealth: 'ok',
    watchHealth: 'ok',
    devUrl: 'sail dev',
    fabVisible: true,
    autoFix: false,
    selection: { nodeId: null, selector: null, source: 'clear' },
    checksBadge: 0,
    tasksBadge: 0,
    regionNodes: [],
    batchCount: 0,
    sailPath: null,
    moduleActivities: [],
    moduleRoute: null,
    moduleUnsignedBanner: false,
  }
  listeners.clear()
}
