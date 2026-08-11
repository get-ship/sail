// route.ts — URL state as reactive signals.
//
// `route.params.<name>` and `route.query.<name>` are signals: reading
// them inside an `effect`, `derived`, or a `live()` source thunk
// registers the current observer as a dependency. Navigation
// (popstate or `navigate(path)`) re-evaluates the URL and updates the
// signals; subscribers re-fire just like any other signal write.
//
// Param names are not declared at construct time. The `params` and
// `query` records are JS Proxies that lazily create a signal the first
// time a key is read. The initial value of each signal is computed
// from the current URL: `params` matches the pathname against the
// shapes registered via `defineRoutes(...)`; `query` reads
// `URL.searchParams`.
//
// Single source of truth for navigation: `navigate(path)` calls
// `history.pushState` (or `replaceState`) and triggers a global
// `__shipSetRouteUpdate` callback. Every `route()` instance registers
// itself with that updater on construction so all attached routes see
// the same URL state.
//
// Hosts (deck shell, spaces scaffold, view-host) own the *policy* —
// history, path→route matching, mounting. The runtime owns the
// *mechanism* (reactive signals + popstate listener). The
// `setRouterPort` plug-in keeps the runtime host-agnostic.

import { signal, type Signal } from '@ship/sail-runtime'
import {
  installScrollRestoration,
  onNavigate,
  onPopState,
  applyScroll,
} from './scroll-restoration'

// Layout-chain resolution + the router live in their own modules (this file
// is at the size cap). Re-exported so `@ship/deck-router` is one import site.
export { resolveLayoutChain, type LayoutShape } from './layouts'
export { mountRouter, type MountRouterOptions, type RouterHandle } from './mount-router'
export {
  installRouteBoundary,
  isDevMode,
  type RouteBoundaryHandle,
  type RouteBoundaryOptions,
} from './route-boundary'

// `RouteSignals` and `RouteShape` live in `./types` (importable without the
// browser router) and are re-exported here so `@ship/deck-router`'s public
// surface is unchanged. Param names are discovered lazily: reading
// `route.params.id` for the first time creates the signal and seeds it from
// the current pathname.
export type { RouteSignals, RouteShape } from './types'
import type { RouteSignals, RouteShape } from './types'

interface CompiledShape extends RouteShape {
  regex: RegExp
  keys: string[]
}

interface RouteInstance {
  update: () => void
}

// Module-level registry of route shapes. Shared across every
// `route()` instance so a `defineRoutes` call from one route's setup
// affects param matching for any subsequently constructed route.
const SHAPES: CompiledShape[] = []

// Live `route()` instances. The global `__shipSetRouteUpdate`
// callback walks this set on every navigation.
const INSTANCES: Set<RouteInstance> = new Set()

let popstateBound = false
let updaterInstalled = false

// Edit-mode toggle. Hosts (deck shell, spaces scaffold) call
// `setEditMode(true|false)` at boot. When `EDIT_MODE` is `false`
// (default), `defineRoutes` enforces a static baseline: the first
// call's sorted pattern list is taken as the baseline; subsequent
// calls' sorted-join must START WITH the baseline string (so
// additions are allowed; removals and reorderings are not). When
// `EDIT_MODE` is `true`, anything goes — runtime route additions
// from downloaded routes or hot-reload are accepted without
// enforcement.
//
// `mergeRoutes` is the supported way to bring a runtime half into a
// boot — the static half goes through the gate (or is set up in
// edit mode) and the runtime half is appended via the unchecked
// path. The gate exists to forbid *ad-hoc* runtime additions; the
// merge is explicit.
let EDIT_MODE = false
let STATIC_HASH: string | null = null

/** Enable or disable edit mode. In edit mode, `defineRoutes` accepts
 *  any pattern set; out of edit mode, the first call's pattern set
 *  becomes a static baseline that later calls must extend (never
 *  shrink or reorder). Hosts call this at boot with the value of
 *  the daemon's per-surface edit-mode config. */
export function setEditMode(on: boolean): void {
  EDIT_MODE = on
}

/** Returns the current edit-mode flag. Tests use this to assert the
 *  active state. */
export function isEditMode(): boolean {
  return EDIT_MODE
}

/**
 * Register one or more route shapes whose `:param` segments should
 * populate `route.params`. Safe to call any number of times; later
 * calls append to the registry. Triggers an immediate refresh so
 * already-constructed `route()` instances re-seed their params from
 * the new shapes.
 *
 * Edit-mode gate: out of edit mode, the first call's pattern set is
 * taken as the static baseline. Subsequent calls must be a superset
 * (sorted-join starts with the baseline) — runtime additions are
 * rejected so a shipped app cannot grow its own URL surface beyond
 * what was registered. In edit mode (`setEditMode(true)`) the gate
 * is skipped and any pattern set is accepted.
 *
 * Per-route convention: call `defineRoutes` from the route's `view.ts`
 * (or controller setup) before reading any params. The compiler may
 * lift this to a `<route pattern="..." />` directive in a follow-up;
 * for now it's an imperative call.
 *
 * ```ts
 * import { defineRoutes } from '@ship/deck-router'
 * defineRoutes([{ pattern: '/agents/:id' }])
 * ```
 */
export function defineRoutes(shapes: RouteShape[]): void {
  if (!EDIT_MODE && STATIC_HASH !== null) {
    const incoming = shapes.map((s) => s.pattern).sort().join('|')
    if (!incoming.startsWith(STATIC_HASH)) {
      throw new Error(
        '[sail] defineRoutes: runtime route additions are forbidden outside edit mode. ' +
          'Call setEditMode(true) at boot to allow dynamic patterns.',
      )
    }
  }
  defineRoutesUnchecked(shapes)
  if (STATIC_HASH === null) {
    // Lock the baseline. Once set, this is the pattern set subsequent
    // calls must extend (or equal) when out of edit mode.
    STATIC_HASH = shapes.map((s) => s.pattern).sort().join('|')
  }
}

/** Append shapes to the registry without going through the edit-mode
 *  gate. Internal — {@link mergeRoutes} uses this path because the
 *  static half has already passed the gate (or the host has set
 *  edit mode), and the runtime half is the explicitly-opted-in
 *  addition. User code always goes through {@link defineRoutes}. */
function defineRoutesUnchecked(shapes: RouteShape[]): void {
  for (const s of shapes) SHAPES.push(compilePattern(s))
  // Re-fire the global updater so existing route() instances pick up
  // the new shapes for any params already lazily read.
  globalThis.__shipSetRouteUpdate?.()
}

/**
 * Construct a `RouteSignals` from the current URL.
 *
 * @param opts.url Explicit URL (used by tests). Defaults to
 *   `window.location.href` in the browser; falls back to
 *   `http://localhost/` outside the browser.
 * @param opts.routes Convenience — calls {@link defineRoutes} before
 *   constructing, so the returned `params` are seeded against these
 *   shapes immediately.
 */
export function route(opts?: { url?: URL; routes?: RouteShape[] }): RouteSignals {
  if (opts?.routes) defineRoutes(opts.routes)

  let currentUrl: URL = opts?.url ?? readWindowUrl()

  const pathname = signal(currentUrl.pathname)
  const hash = signal(currentUrl.hash)
  const paramSigs = new Map<string, Signal<string | undefined>>()
  const querySigs = new Map<string, Signal<string | undefined>>()

  const seedParam = (key: string): string | undefined =>
    matchPathname(currentUrl.pathname)[key]
  const seedQuery = (key: string): string | undefined =>
    currentUrl.searchParams.get(key) ?? undefined

  const params = makeProxy(paramSigs, seedParam, 'params')
  const query = makeProxy(querySigs, seedQuery, 'query')

  const update = (): void => {
    currentUrl = readWindowUrl()
    pathname.set(currentUrl.pathname)
    hash.set(currentUrl.hash)
    const matched = matchPathname(currentUrl.pathname)
    for (const [k, s] of paramSigs) s.set(matched[k])
    for (const [k, s] of querySigs) {
      s.set(currentUrl.searchParams.get(k) ?? undefined)
    }
  }

  const instance: RouteInstance = { update }
  INSTANCES.add(instance)
  ensureGlobalUpdater()
  ensurePopstateListener()

  return { params, query, pathname, hash }
}

/**
 * A host-supplied router. The runtime owns the *mechanism* (the reactive
 * `route.*` signals); the host (Deck shell, Tauri shell, view-host) owns
 * the *policy* — history, path→route matching, and mounting. `navigate()`
 * delegates here so a single call from any bundle actually navigates.
 *
 * Registered via {@link setRouterPort}. Mirrors `SailLoader`:
 * the runtime names no host concept, the host plugs one in.
 */
export interface RouterPort {
  /** Navigate to `to` (a path). The host matches, updates history, and
   *  mounts or updates the route as appropriate. */
  navigate(to: string, opts?: { replace?: boolean }): void
  /** Go back one history entry. */
  back(): void
}

/**
 * Register the host router. Call once at host startup. The port is held
 * on `globalThis` so it is visible to every bundle that shares the single
 * `@ship/deck-router` instance (matching `__shipSetRouteUpdate`).
 */
export function setRouterPort(port: RouterPort): void {
  globalThis.__shipRouterPort = port
}

/**
 * SPA-aware navigation. Delegates to the host {@link RouterPort} when one
 * is registered — the host owns matching + mounting, so this actually
 * switches routes.
 *
 * When no port is registered (a single-route host like the view-host, or
 * tests), it falls back to updating the URL + reactive `route.*` signals
 * in place. Cross-route mounting requires a host router.
 *
 * ```ts
 * navigate('/agents/42')
 * navigate('/login', { replace: true })
 * ```
 */
/** Host navigate (scroll restoration). Author ambient inject uses
 * `packages/sail/runtime` `navigate` — both call `__shipRouterPort` when set
 * (SSOT). No-port fallback: runtime updates history+signals; deck also scrolls. */
export function navigate(path: string, opts: { replace?: boolean } = {}): void {
  const port = globalThis.__shipRouterPort
  if (port) {
    port.navigate(path, opts)
    return
  }
  if (typeof window === 'undefined' || !window.history) return
  // Save the outgoing scroll offset and get the history state (carrying this
  // entry's scroll key) before we change the URL.
  const state = onNavigate(opts.replace ?? false)
  if (opts.replace) {
    window.history.replaceState(state, '', path)
  } else {
    window.history.pushState(state, '', path)
  }
  globalThis.__shipSetRouteUpdate?.()
  // A fresh push scrolls to the top once the new route has rendered.
  applyScroll()
}

/**
 * Options for {@link mergeRoutes}. Hosts that boot with a static route
 * table (codegen'd from the file tree) AND a runtime route table
 * (fetched from the daemon) call `mergeRoutes` to combine them.
 *
 * The static half always wins on collision — a downloaded route that
 * declares `/agents/:id` cannot shadow the first-party shell's
 * `/agents/:id` route.
 */
export interface MergeRoutesOptions {
  /** What to do on collision between static and runtime patterns.
   *  Default: `'first-wins'` (the static pattern is kept, the runtime
   *  duplicate is silently dropped). `'log'` keeps the static and
   *  also calls `console.warn` for the host's diagnostics. `'error'`
   *  throws so a hard misconfiguration is loud at boot. */
  onCollision?: 'log' | 'first-wins' | 'error'
  /** The registry to merge into. Defaults to the global SHAPES
   *  registry (the same one `defineRoutes` appends to) so a
   *  subsequent `defineRoutes` call would see the merged set. When
   *  passed, the function appends the merged set to `target` and
   *  returns the merged list WITHOUT mutating the global registry —
   *  the caller decides what to do with it. */
  target?: RouteShape[]
}

/** The return shape of {@link mergeRoutes}. `merged` is the union
 *  with the static half first; `collisions` is the list of runtime
 *  patterns that were dropped (empty unless `onCollision: 'log'` or
 *  `'error'` and at least one collision occurred). */
export interface MergeRoutesResult {
  merged: RouteShape[]
  collisions: string[]
}

/**
 * Merge a static route table with a runtime route table, returning
 * the deduplicated union. The static patterns always win on
 * collision; a runtime pattern that matches a static pattern is
 * dropped. The behavior on collision is selected by
 * `opts.onCollision`:
 *
 * - `'first-wins'` (default): silent drop. The merged list excludes
 *   the runtime duplicate. `collisions` is empty.
 * - `'log'`: silent drop + `console.warn` per collision, and
 *   `collisions` is populated with the dropped patterns.
 * - `'error'`: throws an `Error` on the first collision so a hard
 *   misconfiguration is loud at boot.
 *
 * When `opts.target` is supplied, the merged list is appended to
 * `target` and returned WITHOUT mutating the global registry. When
 * omitted, the runtime half is appended to the global SHAPES
 * registry via the unchecked path (the gate only guards the
 * public `defineRoutes` surface; `mergeRoutes` is the supported
 * way to combine two halves).
 */
export function mergeRoutes(
  staticRoutes: RouteShape[],
  runtimeRoutes: RouteShape[],
  opts: MergeRoutesOptions = {},
): MergeRoutesResult {
  const onCollision = opts.onCollision ?? 'first-wins'
  const staticPatterns = new Set(staticRoutes.map((r) => r.pattern))
  const collisions: string[] = []
  const merged: RouteShape[] = [...staticRoutes]
  for (const r of runtimeRoutes) {
    if (staticPatterns.has(r.pattern)) {
      if (onCollision === 'error') {
        throw new Error(
          `[sail] mergeRoutes: runtime pattern '${r.pattern}' collides with a static pattern`,
        )
      }
      if (onCollision === 'log') {
        collisions.push(r.pattern)
        console.warn(
          `[sail] mergeRoutes: dropped runtime pattern '${r.pattern}' (collides with a static pattern)`,
        )
      }
      continue
    }
    merged.push(r)
  }
  if (opts.target) {
    // Caller supplied a target list — splice into it and return
    // without touching the global registry.
    for (const r of merged) opts.target.push(r)
  } else {
    // Default: append the runtime half to the global registry
    // directly. The static half is presumed already registered
    // (or about to be) by the host — re-adding it would be a
    // no-op pattern but waste a sorted-join check.
    defineRoutesUnchecked(merged.slice(staticRoutes.length))
  }
  return { merged, collisions }
}

/**
 * Find the registered route shape whose pattern matches `pathname` (or
 * `undefined`). The router uses this to pick the route module for a
 * navigation — modules are keyed by pattern. Reuses the same compiled
 * regexes as `route.params` matching, so there is one matcher, not two.
 */
export function matchRoute(pathname: string): RouteShape | undefined {
  for (const shape of SHAPES) {
    if (shape.regex.test(pathname)) {
      return shape.name === undefined
        ? { pattern: shape.pattern }
        : { pattern: shape.pattern, name: shape.name }
    }
  }
  return undefined
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

function readWindowUrl(): URL {
  if (typeof window !== 'undefined' && window.location?.href) {
    return new URL(window.location.href)
  }
  return new URL('http://localhost/')
}

function compilePattern(shape: RouteShape): CompiledShape {
  const keys: string[] = []
  const segments = shape.pattern.split('/').map((seg) => {
    if (seg.startsWith(':')) {
      keys.push(seg.slice(1))
      return '([^/]+)'
    }
    return escapeRegExp(seg)
  })
  return {
    ...shape,
    regex: new RegExp('^' + segments.join('/') + '/?$'),
    keys,
  }
}

const escapeRegExp = (s: string): string =>
  s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')

function matchPathname(pathname: string): Record<string, string | undefined> {
  for (const shape of SHAPES) {
    const m = shape.regex.exec(pathname)
    if (!m) continue
    const out: Record<string, string | undefined> = {}
    shape.keys.forEach((k, i) => {
      out[k] = m[i + 1]
    })
    return out
  }
  return {}
}

/**
 * Build the lazy Proxy backing `params` or `query`. `proxy[key]`
 * returns a `Signal<string|undefined>`, created on first access from
 * `seed(key)`. `proxy.toJSON()` serializes currently-known keys via
 * `.peek()` (untracked) so `JSON.stringify` works. `set`/
 * `deleteProperty` throw — change the URL via `navigate(path)`.
 * `Object.keys` reflects only keys that have been read at least once.
 */
function makeProxy(
  store: Map<string, Signal<string | undefined>>,
  seed: (key: string) => string | undefined,
  label: string,
): Record<string, Signal<string | undefined>> {
  const ensure = (key: string): Signal<string | undefined> => {
    let s = store.get(key)
    if (!s) {
      s = signal<string | undefined>(seed(key))
      store.set(key, s)
    }
    return s
  }
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
      return ensure(key)
    },
    set(_t, key) {
      throw new Error(
        `[sail] route.${label}.${String(key)} is read-only — call navigate(path) to change the URL`,
      )
    },
    deleteProperty(_t, key) {
      throw new Error(
        `[sail] cannot delete route.${label}.${String(key)} — call navigate(path) to change the URL`,
      )
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

function ensureGlobalUpdater(): void {
  if (updaterInstalled) return
  updaterInstalled = true
  globalThis.__shipSetRouteUpdate = (): void => {
    for (const inst of INSTANCES) inst.update()
  }
}

function ensurePopstateListener(): void {
  if (popstateBound) return
  if (typeof window === 'undefined' || !window.addEventListener) return
  popstateBound = true
  installScrollRestoration()
  window.addEventListener('popstate', () => {
    // Record the destination entry's restore intent, apply the route update,
    // then restore that entry's saved scroll offset once it has rendered.
    onPopState()
    globalThis.__shipSetRouteUpdate?.()
    applyScroll()
  })
}

declare global {
  // eslint-disable-next-line no-var
  var __shipSetRouteUpdate: (() => void) | undefined
  // eslint-disable-next-line no-var
  var __shipRouterPort: RouterPort | undefined
}
