// Ambient navigate — host router port (same contract as deck-router navigate).
// Controllers get this via RUNTIME_AMBIENT inject; no @ship/deck/router import.

export type NavigateOpts = { replace?: boolean }

declare global {
  // eslint-disable-next-line no-var
  var __shipRouterPort:
    | { navigate(path: string, opts?: NavigateOpts): void; back(): void }
    | undefined
  // eslint-disable-next-line no-var
  var __shipSetRouteUpdate: (() => void) | undefined
}

/**
 * Navigate to `path` through the host router port when present; otherwise
 * best-effort history mutation + route signal refresh (browser only).
 */
export function navigate(path: string, opts: NavigateOpts = {}): void {
  const port = globalThis.__shipRouterPort
  if (port) {
    port.navigate(path, opts)
    return
  }
  if (typeof window === 'undefined' || !window.history) return
  if (opts.replace) window.history.replaceState(window.history.state, '', path)
  else window.history.pushState(window.history.state, '', path)
  globalThis.__shipSetRouteUpdate?.()
}
