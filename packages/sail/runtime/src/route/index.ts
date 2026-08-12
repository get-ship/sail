// SAIL runtime — the reactive URL factory.
//
// The SAIL runtime names no host concept, so this is the *mechanism* half of
// routing: it builds a `RouteSignals` (pathname/hash + lazy params/query
// proxies) from a URL. The *policy* half (history, route table, popstate
// listener, `navigate()`) lives in `@ship/deck-router`. SAIL packs and the
// `sail-dom` test harness import this module so a compiled route that reads
// `route.params.x()` evaluates without pulling deck into the dependency
// graph — sail ⊥ deck at the module boundary.
//
// A `route()` instance is a passive reader by design. The deck router
// registers a host-owned `__shipSetRouteUpdate` callback that walks a set
// of `{ update }` records and calls `update()` on every popstate. Without
// that registration, the signals stay at their initial seed values — the
// right behaviour for a test harness that wants a frozen URL.

import { signal, type Signal } from '../signal'
import type { RouteSignals } from '../route-signals'
export type { RouteSignals } from '../route-signals'

/** Internal: a live `RouteSignals` paired with the hooks a host uses to
 *  re-seed the instance on every navigation. */
export interface RouteHandle extends RouteSignals {
  /** Re-seed all signals from the URL this handle was constructed with.
   *  No-op until a host wires the global updater (see module doc). */
  update(): void
  /** The URL snapshot this instance was constructed from. Hosts use it
   *  to compute the next path or expose a "current URL" API. */
  readonly url: URL
}

/** Construct a `RouteSignals` from a URL. Lazy `params` / `query` proxies
 *  materialise a signal on first access; `pathname` and `hash` are seeded
 *  eagerly. `params.<name>` returns `undefined` here — the runtime has no
 *  route table; the deck router's `route()` factory wraps this and seeds
 *  params from the registered shapes. */
export function route(opts: { url?: URL } = {}): RouteSignals {
  const { params, query, pathname, hash } = buildRoute(opts.url)
  return { params, query, pathname, hash }
}

/** Construct a `RouteSignals` plus the host-facing `update` / `url`
 *  hooks. Hosts that own navigation register the returned handle with
 *  their global updater and call `update()` on every popstate. The
 *  deck router consumes this shape; the test harness uses {@link route}
 *  directly because it wants a frozen URL. */
export function routeWithUpdate(opts: { url?: URL } = {}): RouteHandle {
  return buildRoute(opts.url)
}

function buildRoute(url: URL | undefined): RouteHandle {
  // Explicit `opts.url` freezes the handle (test harness). Default seeds from
  // the live location and re-reads on `update()` so hosts that call
  // `__shipSetRouteUpdate` after navigate/popstate see pathname same-tick.
  const pinned = url !== undefined
  let current = url ?? defaultUrl()
  const pathname = signal<string>(current.pathname)
  const hash = signal<string>(current.hash)
  const paramCache = new Map<string, Signal<string | undefined>>()
  const queryCache = new Map<string, Signal<string | undefined>>()

  const params = lazyProxy(paramCache, () => undefined)
  const query = lazyProxy(queryCache, (key) => current.searchParams.get(key) ?? undefined)

  const handle: RouteHandle = {
    params,
    query,
    pathname,
    hash,
    get url() {
      return current
    },
    update(): void {
      if (!pinned) current = defaultUrl()
      pathname.set(current.pathname)
      hash.set(current.hash)
      for (const [k, s] of queryCache) s.set(current.searchParams.get(k) ?? undefined)
      // params stay `undefined` — no route table at the runtime boundary.
    },
  }
  return handle
}

function defaultUrl(): URL {
  if (typeof globalThis.location !== 'undefined' && globalThis.location?.href) {
    return new URL(globalThis.location.href)
  }
  return new URL('http://localhost/')
}

/** Build a lazy proxy that materialises a Signal on first read. Mirrors
 *  the deck router's `makeProxy` shape so a host-supplied route instance
 *  is structurally compatible. Throws on write/delete — change the URL
 *  via `navigate(path)` instead. */
function lazyProxy(
  store: Map<string, Signal<string | undefined>>,
  seed: (key: string) => string | undefined,
): Record<string, Signal<string | undefined>> {
  const target: Record<string, Signal<string | undefined>> = Object.create(null)
  return new Proxy(target, {
    get(_t, key) {
      if (typeof key !== 'string') return undefined
      if (key === 'toJSON') {
        return () => {
          const out: Record<string, string | undefined> = {}
          for (const [k, s] of store) out[k] = s.__peek()
          return out
        }
      }
      let s = store.get(key)
      if (!s) {
        s = signal<string | undefined>(seed(key))
        store.set(key, s)
      }
      return s
    },
    set() {
      throw new Error('[sail] route record is read-only — call navigate(path) to change the URL')
    },
    deleteProperty() {
      throw new Error('[sail] route record is read-only — call navigate(path) to change the URL')
    },
    has(_t, key) {
      return typeof key === 'string' && store.has(key)
    },
    ownKeys() {
      return [...store.keys()]
    },
    getOwnPropertyDescriptor(_t, key) {
      if (typeof key !== 'string') return undefined
      const s = store.get(key)
      if (!s) return undefined
      return { configurable: true, enumerable: true, writable: false, value: s }
    },
  })
}

export type { RouteSignals } from '../route-signals'
