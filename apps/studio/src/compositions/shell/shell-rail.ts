/** Pure rail + path helpers for Studio chrome (no mount / ops). */
import type { ActivityId } from '../../vendor/studio-shell'
import { hostDeckAppRoute, hotkeyModLabel } from '../../vendor/studio-shell'

/** Digit → activity; Ctrl/Cmd+digit navigates. Checks is a panel, not digit-5. */
export const ACTIVITY_BY_DIGIT: Record<string, ActivityId> = {
  '1': 'edit',
  '2': 'components',
  '3': 'themes',
  '4': 'tasks',
  '5': 'config',
}

const RAIL: Record<string, { label: string; icon: string; digit: string }> = {
  edit: { label: 'Edit', icon: 'files', digit: '1' },
  components: { label: 'Components', icon: 'box', digit: '2' },
  themes: { label: 'Themes', icon: 'palette', digit: '3' },
  tasks: { label: 'Tasks', icon: 'messages-square', digit: '4' },
  config: { label: 'Config', icon: 'settings', digit: '5' },
}

const RAIL_ORDER: ActivityId[] = ['edit', 'components', 'themes', 'tasks', 'config']

export type RailRow = {
  id: ActivityId
  label: string
  icon: string
  href: string
  active: boolean
  key: string
}

/** Platform mod glyph for tooltips — ⌘ on Apple, Ctrl+ elsewhere. */
export function hotkeyMod(): string {
  return hotkeyModLabel()
}

export function railLabel(id: string, mod = hotkeyMod()): string {
  const r = RAIL[id]
  if (r) return `${r.label} ${mod}${r.digit}`
  return id.charAt(0).toUpperCase() + id.slice(1)
}

/** Multi-app mount base (e.g. `/_studio`) so rail links stay in-app. */
export function appRouteBase(): string {
  return hostDeckAppRoute()
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

/**
 * Map path → activity id.
 * `/checks` deep-link maps to edit (panel is chrome-hosted).
 */
export function activityFromPath(pathname: string): ActivityId {
  const p = pathForActivity(pathname)
  if (p === '/components' || p.startsWith('/components/')) return 'components'
  if (p === '/config' || p.startsWith('/config/')) return 'config'
  if (p === '/checks' || p.startsWith('/checks/')) return 'edit'
  if (p === '/design' || p.startsWith('/design/')) return 'themes'
  if (p === '/themes' || p.startsWith('/themes/')) return 'themes'
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
  return RAIL_ORDER.map((id) => ({
    id,
    label: railLabel(id, mod),
    icon: RAIL[id]?.icon || 'box',
    href: hrefForActivity(id),
    active: active === id,
    key: id,
  }))
}
