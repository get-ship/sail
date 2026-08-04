/**
 * Components route / mode helpers — href builders + path parse.
 */
import { appRouteBase, hrefForActivity } from '../shell/shell-page-vm'
import type { ComponentMode } from '../components/types'

export function componentsHomeHref(mode?: ComponentMode): string {
  const base = `${appRouteBase()}/components`
  if (mode === 'inspect') return `${base}/inspect`
  return base
}

export function componentHref(id: string, mode?: ComponentMode): string {
  // Inspect is a dedicated segment route (not /:id) so id deep-links stay library.
  if (mode === 'inspect') return componentsHomeHref('inspect')
  if (!id) return componentsHomeHref('library')
  return `${appRouteBase()}/components/${encodeURIComponent(id)}`
}

/** Component id from /components or /components/:id (app-base aware). */
export function componentIdFromLocation(pathname?: string): string {
  if (typeof location === 'undefined' && !pathname) return ''
  const path = pathname ?? (typeof location !== 'undefined' ? location.pathname : '')
  const base = appRouteBase()
  const rest = base && path.startsWith(base) ? path.slice(base.length) : path
  // /components/inspect is a mode segment, not an id
  if (rest === '/components/inspect' || rest.startsWith('/components/inspect/')) return ''
  const m = rest.match(/^\/components\/([^/]+)\/?$/)
  return m ? decodeURIComponent(m[1]!) : ''
}

export function modeFromLocation(search?: string, pathname?: string): ComponentMode {
  const path =
    pathname ??
    (typeof location !== 'undefined' ? location.pathname : '')
  const base = appRouteBase()
  const rest = base && path.startsWith(base) ? path.slice(base.length) : path
  if (rest === '/components/inspect' || rest.startsWith('/components/inspect/')) {
    return 'inspect'
  }
  const raw =
    search ??
    (typeof location !== 'undefined' ? location.search : '')
  try {
    const q = new URLSearchParams(raw.startsWith('?') ? raw : `?${raw}`)
    if (q.get('mode') === 'inspect') return 'inspect'
  } catch {
    /* */
  }
  return 'library'
}

export function themesComponentsHref(themeId = 'studio'): string {
  const base = appRouteBase()
  return `${base}/themes/${encodeURIComponent(themeId)}/components`
}

export function editActivityHref(): string {
  return hrefForActivity('edit')
}

export function tasksActivityHref(): string {
  return hrefForActivity('tasks')
}
