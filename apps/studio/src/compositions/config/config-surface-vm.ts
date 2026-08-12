import { closestAttr, type UiEvent } from '../ui-event'
/**
 * Config surface domain — settings from shell + project only (no invented knobs).
 * Config fields are read-only; danger actions stay disabled.
 */
import { mountAmbient } from '@ship/sail-runtime'
import { getShellState, subscribeShell } from '@ship/sail-studio-shell'
import { resolveProjectRoot } from '../shell/project'

import { signal } from '@ship/sail-runtime'
export type ConfigCatId =
  | 'project'
  | 'runtime'
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
  /** `live` shell · `project` path · `disabled` for unwired ops */
  source: string
}

export type ConfigSectionVm = {
  id: ConfigCatId
  key: string
  title: string
  lead: string
  fields: ConfigFieldRow[]
  selected: boolean
  showShipCloud: boolean
  showDanger: boolean
  showFields: boolean
}

export const CONFIG_CATS: { id: ConfigCatId; label: string }[] = [
  { id: 'project', label: 'Project' },
  { id: 'runtime', label: 'Runtime' },
  { id: 'plugins', label: 'Plugins' },
  { id: 'danger', label: 'Danger zone' },
]

const TITLES: Record<ConfigCatId, string> = {
  project: 'Project',
  runtime: 'Runtime',
  plugins: 'Plugins',
  danger: 'Danger zone',
}

const LEADS: Record<ConfigCatId, string> = {
  project: 'Open root and theme/mode from the running shell.',
  runtime: 'Dev server and loaded module activities.',
  plugins: 'Module activities on the rail. Ship Cloud install is unavailable here.',
  danger: 'Destructive actions are not available yet.',
}

/** Checks panel chord — avoids Chromium Inspect Element (Ctrl/⌘+Shift+C). */
export const CHECKS_HOTKEY_LABEL = 'Ctrl/⌘+Shift+K'

export type ConfigFieldsCtx = {
  root: string
  devUrl: string
  moduleCount: number
  themeId: string
  modeId: string
}

export type ConfigLiveShell = {
  devUrl: string
  moduleCount: number
  themeId: string
  modeId: string
}

export function liveShellChanged(prev: ConfigLiveShell, next: ConfigLiveShell): boolean {
  return (
    prev.devUrl !== next.devUrl ||
    prev.moduleCount !== next.moduleCount ||
    prev.themeId !== next.themeId ||
    prev.modeId !== next.modeId
  )
}

export function readLiveShell(s = getShellState()): ConfigLiveShell {
  return {
    devUrl: s.devUrl || '—',
    moduleCount: s.moduleActivities.length,
    themeId: s.themeId || '—',
    modeId: s.modeId || '—',
  }
}

/**
 * Only values we can read from the running Studio shell / project root.
 * No placeholder compiler knobs.
 */
export function buildConfigFields(id: ConfigCatId, ctx: ConfigFieldsCtx): ConfigFieldRow[] {
  switch (id) {
    case 'project':
      return [
        { key: 'root', label: 'Root', value: ctx.root, source: 'project' },
        { key: 'theme', label: 'Theme', value: ctx.themeId, source: 'live' },
        { key: 'mode', label: 'Mode', value: ctx.modeId, source: 'live' },
      ]
    case 'runtime':
      return [
        { key: 'dev', label: 'Dev URL', value: ctx.devUrl, source: 'live' },
        {
          key: 'modules',
          label: 'Module activities',
          value: String(ctx.moduleCount),
          source: 'live',
        },
        {
          key: 'findings-hotkey',
          label: 'Findings hotkey',
          value: CHECKS_HOTKEY_LABEL,
          source: 'live',
        },
      ]
    case 'plugins':
      return [
        {
          key: 'modules',
          label: 'Loaded modules',
          value: `${ctx.moduleCount} on rail`,
          source: 'live',
        },
      ]
    case 'danger':
      return [
        {
          key: 'reset',
          label: 'Reset Studio state',
          value: 'Not available',
          source: 'disabled',
        },
        {
          key: 'purge',
          label: 'Purge theme drafts',
          value: 'Not available',
          source: 'disabled',
        },
      ]
  }
}

function buildSections(selected: ConfigCatId, ctx: ConfigFieldsCtx): ConfigSectionVm[] {
  return CONFIG_CATS.map((c) => {
    const id = c.id
    return {
      id,
      key: id,
      title: TITLES[id],
      lead: LEADS[id],
      fields: buildConfigFields(id, ctx),
      selected: id === selected,
      showShipCloud: id === 'plugins',
      showDanger: id === 'danger',
      showFields: true,
    }
  })
}

export function createConfigSurfaceCore() {
  const selected = signal<ConfigCatId>('project')
  const categories = signal<ConfigCatRow[]>([])
  const sectionTitle = signal(TITLES.project)
  const sectionLead = signal(LEADS.project)
  const fields = signal<ConfigFieldRow[]>([])
  const sections = signal<ConfigSectionVm[]>([])
  const showShipCloud = signal(false)
  const showDanger = signal(false)
  const projectRoot = signal(resolveProjectRoot())
  const live = readLiveShell()
  const devUrl = signal(live.devUrl)
  const moduleCount = signal(live.moduleCount)
  const themeId = signal(live.themeId)
  const modeId = signal(live.modeId)

  const ctx = (): ConfigFieldsCtx => ({
    root: projectRoot(),
    devUrl: devUrl(),
    moduleCount: moduleCount(),
    themeId: themeId(),
    modeId: modeId(),
  })

  const publish = (id: ConfigCatId) => {
    selected.set(id)
    const c = ctx()
    categories.set(
      CONFIG_CATS.map((cat) => ({
        id: cat.id,
        label: cat.label,
        key: cat.id,
        selected: cat.id === id,
      })),
    )
    sectionTitle.set(TITLES[id])
    sectionLead.set(LEADS[id])
    fields.set(buildConfigFields(id, c))
    sections.set(buildSections(id, c))
    showShipCloud.set(id === 'plugins')
    showDanger.set(id === 'danger')
  }

  publish('project')

  const onCatClick = (e: UiEvent) => {
    const id = closestAttr(e, '[config-cat]', 'config-cat') as ConfigCatId | ''
    if (!id || !CONFIG_CATS.some((c) => c.id === id)) return
    publish(id)
  }

  const selectCategory = (id: ConfigCatId) => {
    if (!CONFIG_CATS.some((c) => c.id === id)) return
    publish(id)
  }

  return {
    categories,
    sectionTitle,
    sectionLead,
    fields,
    sections,
    showShipCloud,
    showDanger,
    onCatClick,
    selectCategory,
    selected,
    devUrl,
    moduleCount,
    applyLiveShell: (next: ConfigLiveShell) => {
      const prev: ConfigLiveShell = {
        devUrl: devUrl(),
        moduleCount: moduleCount(),
        themeId: themeId(),
        modeId: modeId(),
      }
      if (!liveShellChanged(prev, next)) return false
      devUrl.set(next.devUrl)
      moduleCount.set(next.moduleCount)
      themeId.set(next.themeId)
      modeId.set(next.modeId)
      publish(selected())
      return true
    },
    publish,
  }
}

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
