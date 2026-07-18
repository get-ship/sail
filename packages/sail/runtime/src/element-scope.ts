// element-scope — the DOM-mirroring scope tree for custom elements.
//
// `useStore(def)` resolves "nearest instance up the scope tree, creating
// one at the root". For that contract to mean anything across ELEMENTS,
// element scopes must form a tree that mirrors the DOM — a bare
// `createScope()` per element made every element its own root, so three
// components sharing one `defineStore` got three instances (the
// store-counter / store-total docs-demo bug).
//
// On connect, the wrapper class (component.ts) calls
// `connectElementScope(host)`:
//   1. Walk UP from the host (parentNode, crossing shadow-root barriers
//      via `.host` — same traversal as error-boundary.ts) to the nearest
//      ancestor element carrying a LIVE (non-disposed) element scope.
//   2. Found one → the new scope is `createChildScope(ancestorScope)`.
//      This is what makes `provideStore` on an ancestor element visible
//      to descendant elements while sibling subtrees keep the root.
//   3. No sail ancestor → parent into the shared per-document CLIENT
//      ROOT scope (lazy module singleton below). All top-level elements
//      on a page chain to the same root, so root-resolved stores are
//      shared page-wide. This module is only reachable from the
//      browser-only component path — SSR renders through server.ts's
//      per-request roots and never imports it — so a module singleton
//      cannot leak state across server requests.
//
// Reconnect (disconnect → connectedCallback again, e.g. a DOM move)
// re-resolves the parent at connect time — the scope tree follows the
// DOM. Ordering hazard: connectedCallback fires in tree order (parents
// before children) for parsing and subtree insertion, so children find
// their parent's stamp. The exception is upgrade order — if a CHILD's
// custom element class is defined before its PARENT's, the child
// upgrades (and connects) first and parents into the client root. It
// still shares root-homed stores; it misses ancestor `provideStore`
// overrides until something reconnects it. Load parent-component
// modules before their children's if override visibility matters.

import { createChildScope, createScope, type EffectScope } from './effect-scope'

/** Symbol property a connected sail host carries pointing at its live
 *  element scope. `Symbol.for` so independent copies of the runtime in
 *  one page (e.g. a baked bundle plus a dev bundle) still see each
 *  other's stamps. */
const ELEMENT_SCOPE_KEY = Symbol.for('sail:element-scope')

interface HostWithScope {
  [ELEMENT_SCOPE_KEY]?: EffectScope
}

/** The per-document root scope that top-level element scopes parent
 *  into. Lazy: created on the first no-ancestor connect. Recreated if
 *  something disposed it (tests). */
let CLIENT_ROOT: EffectScope | null = null

/** The shared per-document client root. Exposed so tests (and future
 *  bootstrap code that wants module-level `useStore` calls to share the
 *  element tree's root) can reach it. */
export function clientRootScope(): EffectScope {
  if (!CLIENT_ROOT || CLIENT_ROOT.disposed) {
    CLIENT_ROOT = createScope()
  }
  return CLIENT_ROOT
}

/** TEST-ONLY: dispose and drop the client root so a test file gets a
 *  fresh page-level scope (module state survives across vitest tests
 *  in one process). */
export function _resetClientRootScope(): void {
  CLIENT_ROOT?.dispose()
  CLIENT_ROOT = null
}

/** The live element scope stamped on `el`, or `null` when `el` is not
 *  a connected sail host (or its scope already disposed). */
export function elementScopeOf(el: Element): EffectScope | null {
  const scope = (el as unknown as HostWithScope)[ELEMENT_SCOPE_KEY]
  return scope && !scope.disposed ? scope : null
}

/** Walk up from `el` (EXCLUSIVE — `el`'s own stale stamp from a prior
 *  connect must not become its own parent) to the nearest ancestor
 *  element with a live element scope. Crosses shadow-root barriers via
 *  `.host` so a host outside the shadow tree still parents inner
 *  elements. Returns `null` when no sail ancestor exists. */
function findAncestorElementScope(el: Element): EffectScope | null {
  let cur: Node | null = el.parentNode
  while (cur) {
    if (cur instanceof Element) {
      const scope = elementScopeOf(cur)
      if (scope) return scope
    }
    const parent: Node | null = cur.parentNode
    if (parent) {
      cur = parent
      continue
    }
    // Shadow-root barrier: parentNode is null at the root, but `host`
    // exposes the element that owns this shadow tree.
    const root = cur as unknown as { host?: Node }
    cur = root.host ?? null
  }
  return null
}

/**
 * Create the scope for a connecting sail host: a CHILD of the nearest
 * ancestor element's live scope, or of the per-document client root
 * when no sail ancestor exists. Stamps the scope on the element so
 * descendants connecting later find it. Called from the wrapper
 * class's `runSetup` (component.ts) — once per connect.
 */
export function connectElementScope(host: HTMLElement): EffectScope {
  const parent = findAncestorElementScope(host) ?? clientRootScope()
  const scope = createChildScope(parent)
  ;(host as unknown as HostWithScope)[ELEMENT_SCOPE_KEY] = scope
  return scope
}

/**
 * Drop the stamp `connectElementScope` left, but only when it still
 * points at `scope` — a synchronous disconnect + reconnect during setup
 * (see the re-entrancy comment in component.ts `runSetup`) can have
 * already stamped the NEW connect's scope by the time the old
 * teardown runs; that stamp must survive.
 */
export function disconnectElementScope(host: HTMLElement, scope: EffectScope): void {
  const holder = host as unknown as HostWithScope
  if (holder[ELEMENT_SCOPE_KEY] === scope) {
    delete holder[ELEMENT_SCOPE_KEY]
  }
}
