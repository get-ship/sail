// mountRouter — the per-app router. Promotes the shell's hand-rolled
// `mountRoute` (apps/shell/src/main.ts) into the deck framework so every
// per-app deck host (hub, spaces, …) gets SPA navigation + layout chains from
// one place. Implements ADR `unified-deck-router` for the per-app model: when
// that ADR was written the host was the single shell; the per-app cutover
// means the framework must provide the router so no app re-hand-rolls it.
//
// Invariants this file holds to:
//   - deck ⊥ ship: there is no `ship` here — clients are a generic name→client
//     map; the client name belongs to the app, not the framework.
//   - no hardcoding: the root element, route table, and layouts are injected
//     by the bundler-emitted entry (from config); nothing is baked in.
//   - deck → sui only: this imports the runtime's `mount()`; the runtime never
//     imports this.

import { mount, type MountHandle, type ViewPayload } from '@ship/sail-runtime'
import {
  installScrollRestoration,
  onNavigate,
  onPopState,
  applyScroll,
} from './scroll-restoration'
import {
  matchRoute,
  navigate,
  route as createRoute,
  setRouterPort,
  type RouteShape,
} from './index'
import { resolveLayoutChain, type LayoutShape } from './layouts'
import { installRouteBoundary, type RouteBoundaryHandle } from './route-boundary'

export interface MountRouterOptions {
  /** Mount target — a selector or element. Injected by the emitted entry; the
   *  framework bakes in no `#root`. */
  root: string | HTMLElement
  /** The generated route table (`DYNAMIC_ROUTE_PATTERNS`). */
  routes: RouteShape[]
  /** The generated layout table (`LAYOUTS`); empty = no chrome wrappers. */
  layouts?: LayoutShape[]
  /** Pattern → statically-imported view module. The emitted entry imports each
   *  route's `page.sui` and builds this map. */
  modules: Record<string, unknown>
  /** Host-injected named clients, spread into view scope. Generic — the client
   *  name is the app's (e.g. `ship`), not a framework constant. */
  clients?: Record<string, unknown>
  /** Optional host loader (by-name/by-path payload fetch). Per-app bundles
   *  co-bundle their views, so this is usually unset. */
  loader?: unknown
  /** pathPrefix → statically-imported layout view module. When an entry exists
   *  for a prefix, the layout is rendered via mount() so its full authored
   *  markup (chrome element + any composed slotted children) appears in the
   *  DOM. When absent for a prefix, falls back to bare createElement(tag) —
   *  the same behavior as before this field existed — so tag-only layouts
   *  never regress. Emitted by the bundler alongside `modules`. */
  layoutModules?: Record<string, unknown>
  /** Install the DEFAULT route-level error boundary (the framework wraps every
   *  route outlet so a throwing view degrades to an error card, not a dead
   *  subtree). On by default. Set `false` to opt the whole app out — a layout
   *  can opt out a subtree on its own by supplying its own `<sail-error-boundary>`
   *  (which this detects and defers to). */
  errorBoundary?: boolean
}

/** A running router. `dispose()` detaches its global listeners (HMR/tests). */
export interface RouterHandle {
  dispose(): void
}

export function mountRouter(opts: MountRouterOptions): RouterHandle {
  const rootEl =
    typeof opts.root === 'string' ? document.querySelector(opts.root) : opts.root
  if (!rootEl) {
    throw new Error(`[deck/route] mountRouter: root '${String(opts.root)}' not found`)
  }

  // Registers the patterns (route() calls defineRoutes internally) and returns
  // the reactive URL handle injected into every mounted view.
  const routeSignals = createRoute({ routes: opts.routes })
  const layouts = opts.layouts ?? []
  const layoutModules = opts.layoutModules ?? {}
  const errorBoundaryEnabled = opts.errorBoundary !== false

  let token = 0
  let current: MountHandle | null = null
  // Layout MountHandles for the current navigation. Disposed when the next
  // navigation replaces the chrome chain or the router itself disposes.
  let prevLayoutHandles: MountHandle[] = []
  // The default route boundary installed on the current outlet. For chrome
  // navigations the outlet is a fresh slot discarded with the old chrome tree;
  // for the rootless case it persists and is reused (the double-wrap check in
  // installRouteBoundary returns null, leaving this untouched). Detached on
  // router dispose.
  let routeBoundary: RouteBoundaryHandle | null = null

  // `restore`: snapshot of the previous mount's signal VALUES (HMR / reload).
  // Passed to `mount()` as `restoreScope` so the fresh module's matching signals
  // seed from the old values — state survives a re-mount. Undefined for a normal
  // navigation (fresh scope).
  async function mountCurrent(restore?: Record<string, unknown>): Promise<void> {
    const pathname = location.pathname
    const myToken = ++token
    const matched = matchRoute(pathname)
    const raw = matched ? opts.modules[matched.pattern] : undefined
    if (!raw) {
      // No bundled view for this path — build the layout chain (so the nav +
      // studio rail stay visible) and mount a graceful fallback in the route slot.
      // A missing route must never produce a blank screen.
      const chain = resolveLayoutChain(pathname, layouts)
      const { target, layoutHandles } = await buildMountTarget(rootEl!, chain, layoutModules)
      if (myToken !== token) {
        for (const h of layoutHandles) h.dispose()
        return
      }
      target.replaceChildren(buildNotFound(pathname))
      current?.dispose()
      for (const h of prevLayoutHandles) h.dispose()
      current = null
      prevLayoutHandles = layoutHandles
      return
    }
    // Module resolution. Eager (co-bundled, static `import * as`) modules ARE
    // the view namespace already. Code-split modules are a `() => import(...)`
    // thunk that resolves to the namespace — await it here so a route's chunk
    // loads on demand (the module-granular bundling path). Backward-compatible:
    // a non-function `raw` (the eager norm) passes straight through.
    const mod =
      typeof raw === 'function'
        ? await (raw as () => Promise<unknown>)()
        : raw
    // A navigation during the dynamic import supersedes this one.
    if (myToken !== token) return
    const { target, layoutHandles } = await buildMountTarget(
      rootEl!,
      resolveLayoutChain(pathname, layouts),
      layoutModules,
    )
    // Race guard after buildMountTarget (it is async when layout modules are
    // present): a later navigation already superseded this one.
    if (myToken !== token) {
      for (const h of layoutHandles) h.dispose()
      return
    }
    // Install the DEFAULT route boundary on the outlet BEFORE mounting the view,
    // so a throw during the view's render / controller mount / effect is caught
    // and degrades to an error card. No-op when a layout already supplied a
    // boundary (installRouteBoundary returns null) or when opted out.
    if (errorBoundaryEnabled) {
      const installed = installRouteBoundary(target, {
        onReset: () => { void mountCurrent() },
      })
      if (installed) routeBoundary = installed
    }
    const previous = current
    const prevHandles = prevLayoutHandles
    // Co-bundled views are the norm: `opts.modules` is statically imported, so
    // there is no SSR payload to fetch and no loader. Pass an empty payload so
    // `mount()` skips by-name resolution (which requires a loader) and, finding
    // the target empty, runs the view's `__mount(target)` to build the DOM
    // (CSR) then `hydrate`. A host that DOES inject a loader keeps by-name SSR
    // fetching (pass the pathname so `mount()` resolves the payload).
    const handle = await mount(
      target,
      opts.loader ? pathname : coBundledPayload(pathname),
      {
        module: mod as never,
        // `clients` is an exactOptionalPropertyType — present-but-undefined is
        // rejected, so omit the key entirely when the host passed no clients.
        ...(opts.clients ? { clients: opts.clients } : {}),
        // Same exactOptional discipline for the HMR/reload scope snapshot.
        ...(restore ? { restoreScope: restore } : {}),
        loader: opts.loader as never,
        route: routeSignals,
        navigate,
        cssScope: 'adjacent',
      },
    )
    // Race guard: a later navigation already superseded this mount.
    if (myToken !== token) {
      handle.dispose()
      for (const h of layoutHandles) h.dispose()
      return
    }
    current = handle
    prevLayoutHandles = layoutHandles
    previous?.dispose()
    for (const h of prevHandles) h.dispose()
  }

  function go(to: string, navOpts?: { replace?: boolean }): void {
    // Scroll is part of navigating. This router had its own history calls and
    // never told scroll-restoration about them, so a push kept the previous
    // page's offset — you landed halfway down a doc you had never seen.
    // `onNavigate` saves the outgoing offset and marks the entry; `applyScroll`
    // must run AFTER the route renders, or it scrolls the OLD tree.
    const state = onNavigate(navOpts?.replace ?? false)
    if (navOpts?.replace) history.replaceState(state, '', to)
    else history.pushState(state, '', to)
    globalThis.__shipSetRouteUpdate?.()
    void mountCurrent().then(() => applyScroll())
  }

  function onClick(e: MouseEvent): void {
    if (
      e.defaultPrevented || e.button !== 0 ||
      e.metaKey || e.ctrlKey || e.shiftKey || e.altKey
    ) {
      return
    }
    let anchor: HTMLAnchorElement | undefined
    for (const el of e.composedPath?.() ?? []) {
      if (el instanceof HTMLAnchorElement) { anchor = el; break }
    }
    anchor ??= (e.target as Element | null)?.closest?.('a') ?? undefined
    if (!anchor) return
    const href = anchor.getAttribute('href')
    if (!href || anchor.target === '_blank' || anchor.hasAttribute('download')) return
    const url = new URL(anchor.href, location.href)
    if (url.origin !== location.origin) return
    e.preventDefault()
    go(url.pathname + url.search + url.hash)
  }

  const onPop = (): void => {
    onPopState()
    void mountCurrent().then(() => applyScroll())
  }

  installScrollRestoration()
  setRouterPort({ navigate: (to, navOpts) => go(to, navOpts), back: () => history.back() })
  document.addEventListener('click', onClick)
  window.addEventListener('popstate', onPop)

  void mountCurrent()

  // ── HMR seam ─────────────────────────────────────────────────────────────
  // Snapshot the current mount's WRITABLE signals (value, not the signal) so a
  // re-mount can re-seed them. Only signals (callable + `.set`) are captured;
  // every other scope member rebuilds fresh from the new module.
  function captureScope(handle: MountHandle | null): Record<string, unknown> | undefined {
    if (!handle?.scope) return undefined
    const out: Record<string, unknown> = {}
    for (const k of Object.keys(handle.scope)) {
      const v = (handle.scope as Record<string, unknown>)[k]
      if (typeof v === 'function' && typeof (v as { set?: unknown }).set === 'function') {
        try {
          out[k] = (v as () => unknown)()
        } catch {
          /* not a readable signal — skip */
        }
      }
    }
    return out
  }

  // Re-mount the current route preserving signal state. The module is re-read
  // from `opts.modules` (so a `swapModule` patch takes effect) and re-imported.
  async function reloadCurrent(): Promise<void> {
    await mountCurrent(captureScope(current))
  }

  // HMR: point a route at a fresh chunk URL, then re-mount it preserving state
  // IF it's the route on screen (else the next navigation loads it fresh). The
  // deck dev live-reload client calls this with the rebuilt chunk on edit.
  async function swapModule(pattern: string, freshUrl: string): Promise<void> {
    ;(opts.modules as Record<string, unknown>)[pattern] = () =>
      import(/* @vite-ignore */ freshUrl)
    if (matchRoute(location.pathname)?.pattern === pattern) await reloadCurrent()
  }

  // Replace the whole module map with a freshly-built one (the rebuilt entry's
  // `modules`, whose thunks point at the new chunk URLs), then re-mount the
  // current route preserving state. The HMR-aware entry bootstrap calls this on
  // re-import instead of re-running mountRouter — so a full app rebuild becomes
  // a state-preserving soft reload, no full document reload.
  async function swapModules(fresh: Record<string, unknown>): Promise<void> {
    const map = opts.modules as Record<string, unknown>
    for (const k of Object.keys(fresh)) map[k] = fresh[k]
    await reloadCurrent()
  }

  ;(globalThis as unknown as { __deckHmr?: unknown }).__deckHmr = {
    reloadCurrent,
    swapModule,
    swapModules,
  }

  return {
    dispose(): void {
      document.removeEventListener('click', onClick)
      window.removeEventListener('popstate', onPop)
      delete (globalThis as unknown as { __deckHmr?: unknown }).__deckHmr
      current?.dispose()
      current = null
      routeBoundary?.dispose()
      routeBoundary = null
      for (const h of prevLayoutHandles) h.dispose()
      prevLayoutHandles = []
    },
  }
}

/**
 * Empty CSR payload for the co-bundled case. The view module is statically
 * imported (`opts.modules`), so there is no daemon-produced SSR payload and no
 * loader to fetch one. Empty `html`/`css`/`scope` leave the mount target empty,
 * so `mount()` runs the view's `__mount(target)` (the per-app CSR builder) then
 * `hydrate`. `params` is empty — `route.params` is seeded from the URL by the
 * `route()` instance (`routeSignals`), not from a payload.
 */
function coBundledPayload(name: string): ViewPayload {
  return { name, html: '', css: '', scope: {}, moduleUrl: '', params: {}, clients: [] }
}

 /**
  * Build the 404 fallback element. The chrome chain is already mounted at
  * this point (see `mountCurrent`'s no-match branch), so this element lives
  * INSIDE the chrome's route slot — the rail and nav stay visible. The
  * surface is the shared `<ship-error-state>` primitive from
  * `@ship/primitives`; the chrome's compiled view emits a
  * `@sail-tag/ship-error-state` import for us, so by the time this runs
  * the custom element is registered.
  *
  * `textContent` (not innerHTML) on the body keeps user-controlled URL
  * paths safe from XSS. The element upgrades synchronously because the
  * chrome's mount precedes this call.
  */
 function buildNotFound(pathname: string): HTMLElement {
   // Guard against an unbuilt workspace — falling back to a bare text node
   // (the previous behaviour) is better than throwing. The framework
   // never imports `@ship/primitives` directly; registration is a bundler
   // concern, and the chrome view does the heavy lifting.
   if (!customElements.get('ship-error-state')) {
     const fallback = document.createElement('div')
     fallback.className = 'deck-not-found'
     const msg = document.createElement('p')
     msg.textContent = `Not found: ${pathname}`
     return fallback
   }
   const el = document.createElement('ship-error-state')
   // variant prop default is 'not-found' — set it explicitly so the host
   // attribute is observable (the component-factory skips the first
   // reflect pass for defaults).
   el.setAttribute('variant', 'not-found')
   el.setAttribute('body', `No route matches ${pathname}.`)
   el.setAttribute('action-label', 'Go back')
   return el
 }

/**
 * Build the chrome chain for `chain` (deepest-first) under `root`, returning
 * the innermost `<div slot="route">` to mount the route content into, plus any
 * layout MountHandles for later disposal. Empty chain → the root element itself.
 *
 * When a layout has an entry in `layoutModules`, its view module is mounted via
 * the runtime's `mount()` so its full authored markup (the chrome element plus
 * any composed slotted children, e.g. `<sail-command-bar slot="command-bar">`)
 * renders into the DOM. The rendered chrome element is located by `layout.tag`
 * after mounting; `parent` (the inner content accumulated so far) is appended
 * to it. When no module entry exists for a prefix, falls back to the original
 * `document.createElement(tag)` path — no regression for tag-only layouts.
 *
 * The chain is rebuilt per navigation; reusing chrome across same-prefix
 * navigations is a later optimization.
 */
async function buildMountTarget(
  root: Element,
  chain: LayoutShape[],
  layoutModules: Record<string, unknown>,
): Promise<{ target: HTMLElement; layoutHandles: MountHandle[] }> {
  if (chain.length === 0) return { target: root as HTMLElement, layoutHandles: [] }

  const routeSlot = document.createElement('div')
  routeSlot.setAttribute('slot', 'route')
  let parent: HTMLElement = routeSlot
  const layoutHandles: MountHandle[] = []

  // Chain is deepest-first (index 0 = innermost). Iterate deepest-to-outermost,
  // wrapping `parent` in each chrome layer; `root.replaceChildren` places the
  // assembled tree into the document once complete.
  for (const layout of chain) {
    const mod = layoutModules[layout.pathPrefix]
    if (mod) {
      // Layout has a view module — mount it to render authored markup.
      // Mount into a temporary container so the rendered chrome can be
      // extracted and moved into the chain (DOM move, no clone needed).
      const container = document.createElement('div')
      const handle = await mount(
        container,
        coBundledPayload(`layout:${layout.pathPrefix}`),
        { module: mod as never, cssScope: 'adjacent' },
      )
      layoutHandles.push(handle)
      // Find the rendered chrome by the known tag; fall back to firstElementChild
      // for resilience, and to container itself if the mount produced nothing
      // (degenerate layout module — never blank).
      const chrome =
        (container.querySelector(layout.tag) as HTMLElement | null) ??
        (container.firstElementChild as HTMLElement | null)
      if (chrome) {
        chrome.appendChild(parent)
        parent = chrome
      } else {
        container.appendChild(parent)
        parent = container
      }
    } else {
      // Fallback: bare custom-element tag. The element registers via the
      // side-effect import emitted in routes.gen.ts; authored layout children
      // are not rendered (pre-fix behavior preserved for tag-only layouts).
      const chrome = document.createElement(layout.tag)
      chrome.appendChild(parent)
      parent = chrome
    }
  }

  root.replaceChildren(parent)
  return { target: routeSlot, layoutHandles }
}
