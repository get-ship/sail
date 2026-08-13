// Host port for the live RouteSignals grant. Chrome CEs reuse across
// same-prefix navigations and cannot import deck; they read this port
// the same way navigate reads `__shipRouterPort`.

import type { RouteSignals } from './types'

declare global {
  // eslint-disable-next-line no-var
  var __shipRouteSignals: RouteSignals | undefined
}

/** Park the host RouteSignals on globalThis for the runtime CE factory. */
export function setRouteSignals(route: RouteSignals | undefined): void {
  globalThis.__shipRouteSignals = route
}
