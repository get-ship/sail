/**
 * Leave Studio / devtools.
 * Prefer host bridge postMessage; ambient location/history removed (DOMDEC).
 * Full-page leave without iframe needs navigate inject (gap) — bridge hosts close via message.
 */
import { closeStudioOverlay, hostDeckAppRoute } from '../../vendor/studio-shell'

function deckAppRouteBase(): string {
  return hostDeckAppRoute()
}

export function isStudioOverlayPath(pathname = ''): boolean {
  const p = pathname
  if (deckAppRouteBase() === '/_studio') return true
  return p === '/_studio' || p.startsWith('/_studio/')
}

export function canExitStudio(): boolean {
  return true
}

export function exitStudio(): void {
  closeStudioOverlay()
}
