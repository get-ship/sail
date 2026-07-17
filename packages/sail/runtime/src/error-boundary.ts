// Error-boundary runtime helpers.
//
// A `<sail-error-boundary>` host attaches a `BoundaryHandle` to itself
// via `attachBoundary(host, handle)`. Throw-prone call sites in the
// runtime — `bindFor`'s render(), `bindIf`'s branch render, the
// component factory's setup (which catches synchronous controller
// `mount()` throws), and `effect()`'s tracked re-run — wrap their
// execution with `runProtected(node, source, fn)`. On throw,
// `findNearestBoundary` walks up from `node` (crossing shadow-root
// boundaries) to the innermost boundary; if one is found, its
// `.catch(err, source)` is invoked and the throw is swallowed.
//
// When no boundary is in scope, errors fall through to the existing
// console.error behaviour so previously-uncaught throws stay logged
// rather than silently disappearing.
//
// The boundary lookup is DOM-driven on purpose: it works uniformly
// across the runtime/component/effect layers without any of them
// needing to carry a boundary reference. The trade-off is that the
// throwing code must hand `runProtected` a node anchored in the DOM
// subtree the boundary wraps. Every existing integration point has
// such a node (anchors, hosts).

/** Origin of a caught error — surfaced via the boundary's
 *  `sail-error-boundary:caught` event so listeners can branch on
 *  whether the throw came from a re-run effect, a controller mount
 *  callback, or a render-time expression. `'unknown'` is the safety
 *  fallback when the call site couldn't classify the source. */
export type BoundarySource = 'effect' | 'mount' | 'render' | 'unknown'

/** The contract a `<sail-error-boundary>` element installs on its host
 *  via `attachBoundary`. The runtime consumes it via `.catch` when an
 *  inner throw is caught; authors may call `.reset()` programmatically
 *  to re-render the protected subtree. */
export interface BoundaryHandle {
  /** Surface a caught error to the boundary. Called by `runProtected`
   *  after locating this boundary via `findNearestBoundary`. */
  catch(err: unknown, source: BoundarySource): void
  /** Re-render the boundary's protected subtree by clearing the
   *  caught-error signal. Idempotent when the boundary is already in
   *  the healthy state. */
  reset(): void
}

/** Symbol property the boundary host carries. Using a Symbol keeps
 *  the key invisible to ordinary property enumeration and avoids name
 *  collisions with user attributes. */
const BOUNDARY_KEY = Symbol.for('sail:error-boundary')

interface HostWithBoundary {
  [BOUNDARY_KEY]?: BoundaryHandle
}

/** Attach a boundary handle to its host element. Called from the
 *  boundary controller's `mount`. */
export function attachBoundary(host: HTMLElement, handle: BoundaryHandle): void {
  ;(host as unknown as HostWithBoundary)[BOUNDARY_KEY] = handle
}

/** Remove the boundary handle from its host. Called on `unmount`. */
export function detachBoundary(host: HTMLElement): void {
  delete (host as unknown as HostWithBoundary)[BOUNDARY_KEY]
}

/**
 * Walk up the DOM from `node` until the nearest element carrying a
 * `BoundaryHandle` is found, or the root is reached. Traverses shadow
 * roots: when the walk reaches a `ShadowRoot`, it continues from the
 * host element so a `<sail-error-boundary>` outside the shadow tree
 * still catches inner throws.
 *
 * @param node Any node (element, text, comment) inside the subtree
 *   being protected. Typically a binding anchor or a custom-element
 *   host that just threw.
 * @returns The handle of the innermost boundary ancestor, or `null`
 *   if there is none.
 */
export function findNearestBoundary(node: Node | null): BoundaryHandle | null {
  let cur: Node | null = node
  while (cur) {
    if (cur instanceof Element) {
      const handle = (cur as unknown as HostWithBoundary)[BOUNDARY_KEY]
      if (handle) return handle
    }
    const parent: Node | null = cur.parentNode
    if (parent) {
      cur = parent
      continue
    }
    // Cross a shadow-root barrier: parentNode is null at the root,
    // but `host` exposes the element that owns this shadow tree.
    const root = cur as unknown as { host?: Node }
    if (root.host) {
      cur = root.host
      continue
    }
    cur = null
  }
  return null
}

/**
 * Run `fn` inside try/catch. On throw, find the nearest boundary above
 * `node` and route the error there; if none exists, log via
 * `console.error` so the previously-uncaught behaviour is preserved.
 *
 * @param node DOM anchor used to locate the boundary on throw. Pass
 *   the binding's anchor, the component's host, or any other node in
 *   the subtree being protected.
 * @param source Classifier surfaced to the boundary's listener.
 * @param fn The function to invoke.
 * @returns Whatever `fn` returns, or `undefined` if it threw and a
 *   boundary caught the error.
 */
export function runProtected<T>(
  node: Node | null,
  source: BoundarySource,
  fn: () => T,
): T | undefined {
  try {
    return fn()
  } catch (err) {
    const handle = findNearestBoundary(node)
    if (handle) {
      handle.catch(err, source)
      return undefined
    }
    console.error('[sui] uncaught throw (no error boundary):', err)
    return undefined
  }
}
