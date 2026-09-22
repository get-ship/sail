/**
 * Bridge: shell activity rail → Themes dirty leave guard.
 * Session registers while a facet is mounted; shell asks before leaving an open theme editor.
 */
import { getDesignState } from '../../design/index.ts'
import { pathForActivity } from '../shell/shell-page-vm'

/**
 * @returns true = navigate immediately; false = leave dialog owns the flow (do not navigate).
 */
export type ThemeLeaveNavigate = (href: string) => boolean

let navigateHandler: ThemeLeaveNavigate | null = null

/** Theme session registers while a facet is live; clears on unmount. */
export function setThemeLeaveNavigate(fn: ThemeLeaveNavigate | null): void {
  navigateHandler = fn
}

function relPath(href: string): string {
  try {
    if (href.startsWith('http')) {
      return pathForActivity(new URL(href).pathname)
    }
  } catch {
    /* */
  }
  // App-relative: /themes or /_studio/themes → normalize via pathForActivity when absolute path
  if (href.startsWith('/')) return pathForActivity(href)
  return href.startsWith('/') ? href : `/${href}`
}

/**
 * Call from shell rail before navigate. When editing a theme and dirty, leaving
 * that theme (home or other activity) opens Save/Discard/Cancel.
 */
export function requestThemeLeaveNavigate(href: string): boolean {
  if (!href) return true
  try {
    const ds = getDesignState()
    if (!ds.dirty) return true
    // Without ambient location, treat any leave of /themes/* as leave-theme.
    const dest = relPath(href)
    const stayOnThemes = dest === '/themes' || dest.startsWith('/themes/')
    if (stayOnThemes) {
      // Same theme facet switches stay; leaving theme id entirely still fires.
      // Without from-id, only destinations outside /themes trigger the dialog.
      return true
    }
    if (!navigateHandler) return true
    return navigateHandler(href)
  } catch {
    return true
  }
}
