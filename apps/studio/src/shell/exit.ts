/**
 * Leave Studio / devtools — always available, never a dead-end.
 *
 * - Full-page overlay (`/_studio…`) → primary app `/`
 * - Host bridge iframe → postMessage close to parent
 * - Else history.back() or `/`
 */
import { hostDeckAppRoute, STUDIO_CHANNEL } from '@ship/sail-studio-shell'

function deckAppRouteBase(): string {
  return hostDeckAppRoute()
}

export function isStudioOverlayPath(pathname?: string): boolean {
  const p =
    pathname ??
    (typeof location !== 'undefined' ? location.pathname : '')
  if (deckAppRouteBase() === '/_studio') return true
  return p === '/_studio' || p.startsWith('/_studio/')
}

/** True when the Exit control should show (always on overlay; always when we can leave). */
export function canExitStudio(): boolean {
  if (typeof location === 'undefined') return true
  if (isStudioOverlayPath()) return true
  // Standalone dogfood still needs an escape hatch.
  return true
}

export function exitStudio(): void {
  // Bridge embeds Studio in a shadow frame / iframe — tell the host to close.
  try {
    if (typeof window !== 'undefined' && window.parent && window.parent !== window) {
      window.parent.postMessage({ channel: STUDIO_CHANNEL, type: 'close' }, '*')
    }
  } catch {
    /* cross-origin — fall through to navigation */
  }

  if (typeof location === 'undefined') return

  if (isStudioOverlayPath()) {
    location.assign('/')
    return
  }

  if (typeof history !== 'undefined' && history.length > 1) {
    history.back()
    return
  }
  location.assign('/')
}
