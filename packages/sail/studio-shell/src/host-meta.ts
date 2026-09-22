/**
 * Host-stamped meta tags (deck / framework inject).
 * Apps read project + app-route through this module — never raw document.querySelector.
 */

function readMeta(name: string): string {
  if (typeof document === 'undefined') return ''
  try {
    return (
      document.querySelector(`meta[name="${name}"]`)?.getAttribute('content')?.trim() || ''
    )
  } catch {
    return ''
  }
}

/** Workspace-relative project root from host stamp (`sail-project`). */
export function hostSailProject(): string {
  return readMeta('sail-project').replace(/\/$/, '')
}

/** Multi-app mount base (e.g. `/_studio`) from host stamp (`deck-app-route`). */
export function hostDeckAppRoute(): string {
  const raw = readMeta('deck-app-route')
  if (!raw || raw === '/') return ''
  return raw.replace(/\/$/, '')
}
