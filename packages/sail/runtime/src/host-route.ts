// host-route.ts — deck injects the live route handle for layout custom elements.
//
// Layout CEs (`component(tag, …)`) upgrade on connect and do not receive the
// `mount({ route })` ctx that tagless pages get. Deck registers the same
// `route()` instance here; the CE factory reads it into `ctx.route`.
// Hosts that grant no route leave this unset — CEs that never read route
// are unaffected.

import type { RouteSignals } from './route-signals'

export interface HostRoute {
  route: RouteSignals
  navigate: (path: string) => void
}

/** Register (or clear) the host route handle. Call from the deck router. */
export function setHostRoute(handle: HostRoute | null): void {
  globalThis.__shipHostRoute = handle ?? undefined
}

/** Current host route handle, if the host registered one. */
export function hostRoute(): HostRoute | undefined {
  return globalThis.__shipHostRoute
}

declare global {
  // eslint-disable-next-line no-var
  var __shipHostRoute: HostRoute | undefined
}
