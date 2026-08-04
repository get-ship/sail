/**
 * Config surface domain — fixture/read-only scaffold for project settings.
 * No write path in v1 scaffold; field sources are honest (live vs fixture).
 */
import { mountAmbient, signal } from '@ship/sail-runtime'
import { getShellState, subscribeShell } from '@ship/sail-studio-shell'
import { resolveProjectRoot } from '../shell/project'

export type ConfigCatId =
  | 'project'
  | 'compiler'
  | 'deck'
  | 'studio'
  | 'plugins'
  | 'danger'

export type ConfigCatRow = {
  id: ConfigCatId
  label: string
  key: string
  selected: boolean
}

export type ConfigFieldRow = {
  key: string
  label: string
  value: string
  /** Honesty: `live` / `project` when read at runtime; `fixture` when hardcoded scaffold. */
  source: string
}

export const CONFIG_CATS: { id: ConfigCatId; label: string }[] = [
  { id: 'project', label: 'Project' },
  { id: 'compiler', label: 'Compiler' },
  { id: 'deck', label: 'Deck' },
  { id: 'studio', label: 'Studio' },
  { id: 'plugins', label: 'Plugins' },
  { id: 'danger', label: 'Danger zone' },
]

const TITLES: Record<ConfigCatId, string> = {
  project: 'Project',
  compiler: 'Compiler',
  deck: 'Deck',
  studio: 'Studio preferences',
  plugins: 'Plugins',
  danger: 'Danger zone',
}

const LEADS: Record<ConfigCatId, string> = {
  project: 'Where Studio opens sources and runs ops.',
  compiler: 'Layout gate and ban policy for this project (read-only scaffold).',
  deck: 'Dev server and app mount.',
  studio: 'Editor integration and chrome prefs (fixture).',
  plugins: 'Official + community slots. Ship Cloud is not a second app.',
  danger: 'Destructive project ops — disabled until productized.',
}

/** Checks panel chord — avoids Chromium Inspect Element (Ctrl/⌘+Shift+C). */
export const CHECKS_HOTKEY_LABEL = 'Ctrl/⌘+Shift+K'

export type ConfigFieldsCtx = {
  root: string
  devUrl: string
  moduleCount: number
}

/** Live shell fields Config mirrors (selective refresh — not selection/badges). */
export type ConfigLiveShell = {
  devUrl: string
  moduleCount: number
}

/** True when a live Config field may have changed (devUrl or module activity count). */
export function liveShellChanged(prev: ConfigLiveShell, next: ConfigLiveShell): boolean {
  return prev.devUrl !== next.devUrl || prev.moduleCount !== next.moduleCount
}

export function readLiveShell(s = getShellState()): ConfigLiveShell {
  return {
    devUrl: s.devUrl || 'sail dev',
    moduleCount: s.moduleActivities.length,
  }
}

/**
 * Pure field builder — unit-tested for source honesty.
 * Live: project root, shell devUrl, module activity count.
 * Everything else is explicit `fixture` until real reads land.
 */
export function buildConfigFields(id: ConfigCatId, ctx: ConfigFieldsCtx): ConfigFieldRow[] {
  switch (id) {
    case 'project':
      return [
        { key: 'root', label: 'Root', value: ctx.root, source: 'project' },
        { key: 'pkg', label: 'Package', value: '@ship/studio', source: 'fixture' },
        { key: 'render', label: 'Render', value: 'csr', source: 'fixture' },
      ]
    case 'compiler':
      return [
        { key: 'layout', label: 'Layout severity', value: 'warn', source: 'fixture' },
        {
          key: 'raw',
          label: 'Raw containers',
          value: 'false (ban div/span in src)',
          source: 'fixture',
        },
        {
          key: 'theme',
          label: 'Default theme',
          value: 'studio · dark',
          source: 'fixture',
        },
      ]
    case 'deck':
      return [
        { key: 'dev', label: 'Dev URL', value: ctx.devUrl, source: 'live' },
        {
          key: 'route',
          label: 'App route',
          value: '/ (standalone) or /_studio',
          source: 'fixture',
        },
      ]
    case 'studio':
      return [
        { key: 'editor', label: 'External editor', value: 'vscode', source: 'fixture' },
        {
          key: 'checks-hotkey',
          label: 'Checks hotkey',
          value: CHECKS_HOTKEY_LABEL,
          source: 'fixture',
        },
      ]
    case 'plugins':
      return [
        {
          key: 'modules',
          label: 'Loaded modules',
          value: `${ctx.moduleCount} module activities on rail`,
          source: 'live',
        },
      ]
    case 'danger':
      return [
        {
          key: 'reset',
          label: 'Reset Studio state',
          value: 'Clear local shell + editor tabs',
          source: 'disabled',
        },
        {
          key: 'purge',
          label: 'Purge theme drafts',
          value: 'Drop unsaved theme session drafts',
          source: 'disabled',
        },
      ]
  }
}

/** Pure surface state — no mountAmbient (unit-testable outside controller setup). */
export function createConfigSurfaceCore() {
  const selected = signal<ConfigCatId>('project')
  const categories = signal<ConfigCatRow[]>([])
  const sectionTitle = signal(TITLES.project)
  const sectionLead = signal(LEADS.project)
  const fields = signal<ConfigFieldRow[]>([])
  const showShipCloud = signal(false)
  const showDanger = signal(false)
  const projectRoot = signal(resolveProjectRoot())
  const live = readLiveShell()
  const devUrl = signal(live.devUrl)
  const moduleCount = signal(live.moduleCount)

  const ctx = (): ConfigFieldsCtx => ({
    root: projectRoot(),
    devUrl: devUrl(),
    moduleCount: moduleCount(),
  })

  const publish = (id: ConfigCatId) => {
    selected.set(id)
    categories.set(
      CONFIG_CATS.map((c) => ({
        id: c.id,
        label: c.label,
        key: c.id,
        selected: c.id === id,
      })),
    )
    sectionTitle.set(TITLES[id])
    sectionLead.set(LEADS[id])
    fields.set(buildConfigFields(id, ctx()))
    showShipCloud.set(id === 'plugins')
    showDanger.set(id === 'danger')
  }

  publish('project')

  const onCatClick = (e: Event) => {
    const el = (e.target as Element | null)?.closest?.('[data-config-cat]') as HTMLElement | null
    const id = el?.getAttribute('data-config-cat') as ConfigCatId | null
    if (!id || !CONFIG_CATS.some((c) => c.id === id)) return
    publish(id)
  }

  /** Test / programmatic select (same as cat click). */
  const selectCategory = (id: ConfigCatId) => {
    if (!CONFIG_CATS.some((c) => c.id === id)) return
    publish(id)
  }

  return {
    categories,
    sectionTitle,
    sectionLead,
    fields,
    showShipCloud,
    showDanger,
    onCatClick,
    selectCategory,
    selected,
    devUrl,
    moduleCount,
    /** Apply live shell snapshot; republish when devUrl or module count changes. */
    applyLiveShell: (next: ConfigLiveShell) => {
      const prev: ConfigLiveShell = {
        devUrl: devUrl(),
        moduleCount: moduleCount(),
      }
      if (!liveShellChanged(prev, next)) return false
      devUrl.set(next.devUrl)
      moduleCount.set(next.moduleCount)
      publish(selected())
      return true
    },
    publish,
  }
}

/**
 * Controller entry — refresh when live shell fields change
 * (devUrl, moduleActivities.length). Ignore selection/badge thrash.
 */
export function createConfigSurface() {
  const core = createConfigSurfaceCore()
  mountAmbient(() => {
    const unsub = subscribeShell((s) => {
      core.applyLiveShell(readLiveShell(s))
    })
    return () => unsub()
  })
  return core
}
