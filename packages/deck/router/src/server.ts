// @ship/deck/router — the SERVER (V8 / no-browser) resolution.
//
// The synthetic SSR build aliases `@ship/deck/router` to THIS module. On the
// server there is no `window`, so URL state can't come from `window.location`
// (the browser module's source). It comes from the HOST: the SSR entry fetches
// `route.current` over ship-js's host-dispatch wire and parks the result on
// `globalThis.__shipServerRoute`; this shim exposes it in the `RouteSignals`
// shape the synthesized controller constructs (`const route = route()`).
//
// Reads are LAZY by design. The synthesized controller runs `route()` at
// module-init — BEFORE the entry's async `route.current` has resolved — so
// `route()` must not read host state eagerly. Each `params.<name>()` read
// happens later, inside `mount()`, by which point the host value is parked.
//
// The synthesized render reads signals by CALLING them
// (`scope.route.params.id()`), matching production. So each `params.<name>`
// is a callable that returns the current host value — a read-only signal-
// shaped view (no `.set`; you don't navigate during SSR). It is cast to
// `Signal` because that is its type at the call site; the host owns mutation.

import type { Signal } from '@ship/sui-runtime'
import type { RouteSignals } from './types'

export type { RouteSignals, RouteShape } from './types'

interface HostRoute {
  pathname?: string
  params?: Record<string, string>
  query?: Record<string, string>
  hash?: string
}

function hostRoute(): HostRoute {
  const g = globalThis as { __shipServerRoute?: HostRoute }
  return g.__shipServerRoute ?? { params: {}, query: {} }
}

/** A read-only, lazily-resolved signal-shaped view of one host value. */
function lazy<T>(read: () => T): Signal<T> {
  return (() => read()) as unknown as Signal<T>
}

function lazyRecord(
  pick: (h: HostRoute) => Record<string, string> | undefined,
): Record<string, Signal<string | undefined>> {
  return new Proxy({} as Record<string, Signal<string | undefined>>, {
    get(_t, key) {
      if (typeof key !== 'string') return undefined
      return lazy(() => pick(hostRoute())?.[key])
    },
  })
}

/** The synthesized controller's `route()`. Returns the host-backed,
 *  lazily-read URL handle in `RouteSignals` shape. */
export function route(): RouteSignals {
  return {
    params: lazyRecord((h) => h.params),
    query: lazyRecord((h) => h.query),
    pathname: lazy(() => hostRoute().pathname ?? '/'),
    hash: lazy(() => hostRoute().hash ?? ''),
  }
}

// Browser-only navigation/registry surface the broader controller graph may
// name. Server-side they are inert — the host owns matching + mounting.
export function defineRoutes(): void {}
export function navigate(): void {}
export function setRouterPort(): void {}
export function setEditMode(): void {}
export function isEditMode(): boolean {
  return false
}
export function matchRoute(): undefined {
  return undefined
}
export function mergeRoutes(): { merged: never[]; collisions: never[] } {
  return { merged: [], collisions: [] }
}
