/**
 * Config surface domain — fixture/read-only scaffold for project settings.
 * No write path in v1 scaffold; fields are honest about source.
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
  source: string
}

const CATS: { id: ConfigCatId; label: string }[] = [
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

export function createConfigSurface() {
  const selected = signal<ConfigCatId>('project')
  const categories = signal<ConfigCatRow[]>([])
  const sectionTitle = signal(TITLES.project)
  const sectionLead = signal(LEADS.project)
  const fields = signal<ConfigFieldRow[]>([])
  const showShipCloud = signal(true)
  const showDanger = signal(false)
  const showPlugins = signal(false)
  const projectRoot = signal(resolveProjectRoot())
  const devUrl = signal(getShellState().devUrl || 'sail dev')

  const fieldsFor = (id: ConfigCatId): ConfigFieldRow[] => {
    const root = projectRoot()
    switch (id) {
      case 'project':
        return [
          { key: 'root', label: 'Root', value: root, source: 'project' },
          { key: 'pkg', label: 'Package', value: '@ship/studio', source: 'package.json' },
          { key: 'render', label: 'Render', value: 'csr', source: 'sail.config' },
        ]
      case 'compiler':
        return [
          { key: 'layout', label: 'Layout severity', value: 'warn', source: 'sail.config' },
          {
            key: 'raw',
            label: 'Raw containers',
            value: 'false (ban div/span in src)',
            source: 'sail.config',
          },
          {
            key: 'theme',
            label: 'Default theme',
            value: 'studio · dark',
            source: 'sail.config',
          },
        ]
      case 'deck':
        return [
          { key: 'dev', label: 'Dev URL', value: devUrl(), source: 'live' },
          {
            key: 'route',
            label: 'App route',
            value: '/ (standalone) or /_studio',
            source: 'meta',
          },
        ]
      case 'studio':
        return [
          { key: 'editor', label: 'External editor', value: 'vscode', source: 'setting' },
          {
            key: 'checks-hotkey',
            label: 'Checks hotkey',
            value: 'Ctrl/⌘+Shift+C',
            source: 'target',
          },
        ]
      case 'plugins':
        return [
          {
            key: 'modules',
            label: 'Loaded modules',
            value: `${getShellState().moduleActivities.length} module activities on rail`,
            source: 'fixture',
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

  const publish = (id: ConfigCatId) => {
    selected.set(id)
    categories.set(
      CATS.map((c) => ({
        id: c.id,
        label: c.label,
        key: c.id,
        selected: c.id === id,
      })),
    )
    sectionTitle.set(TITLES[id])
    sectionLead.set(LEADS[id])
    fields.set(fieldsFor(id))
    showShipCloud.set(id === 'plugins')
    showPlugins.set(id === 'plugins')
    showDanger.set(id === 'danger')
  }

  publish('project')

  mountAmbient(() => {
    const unsub = subscribeShell(() => {
      devUrl.set(getShellState().devUrl || 'sail dev')
      publish(selected())
    })
    return () => unsub()
  })

  const onCatClick = (e: Event) => {
    const el = (e.target as Element | null)?.closest?.('[data-config-cat]') as HTMLElement | null
    const id = el?.getAttribute('data-config-cat') as ConfigCatId | null
    if (!id || !CATS.some((c) => c.id === id)) return
    publish(id)
  }

  return {
    categories,
    sectionTitle,
    sectionLead,
    fields,
    showShipCloud,
    showPlugins,
    showDanger,
    onCatClick,
  }
}
