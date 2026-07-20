// L1 element controllers — optional JS enhancement registered by tag.
//
// Discovery: compile_element stamps `data-sail-el="<tag>"` on structure roots
// that declare a `<controller>`. After a route/component mounts its DOM, the
// host calls `attachElementControllers(root)` which walks
// `[data-sail-el]` and runs the matching controller once per node.
//
// PE honesty:
// - No-JS / SSR-only: structure + skin ship; controllers never run.
// - With-JS: one attach per node; a WeakSet guard prevents double-binding if
//   mount + a later enhance both see the same node.
// - Controllers are NOT `use:` mix-ins — they are the element's own brain.

import { chip, type ElementController, type ElementControllerResult } from './chip'

export type { ElementController, ElementControllerResult }
export { chip }

/** Registry: L1 tag → controller. Only tags with a controller entry enhance. */
export const ELEMENT_CONTROLLERS: Record<string, ElementController> = {
  chip,
}

const ATTACHED = new WeakSet<Element>()
const RESULTS = new WeakMap<Element, ElementControllerResult | void>()

/**
 * Attach every registered L1 element controller under `root` (inclusive).
 * Idempotent per node: a second call is a no-op for already-enhanced nodes
 * (hydration / remount safety — no double listeners).
 *
 * @returns disposer that tears down every controller this call attached.
 */
export function attachElementControllers(root: ParentNode): () => void {
  const attached: Element[] = []
  const nodes: Element[] = []
  if (root instanceof Element && root.hasAttribute('data-sail-el')) {
    nodes.push(root)
  }
  if (typeof (root as ParentNode).querySelectorAll === 'function') {
    root.querySelectorAll('[data-sail-el]').forEach((n) => nodes.push(n))
  }
  for (const el of nodes) {
    if (ATTACHED.has(el)) continue
    const tag = el.getAttribute('data-sail-el')
    if (!tag) continue
    const ctrl = ELEMENT_CONTROLLERS[tag]
    if (!ctrl) continue
    ATTACHED.add(el)
    const result = ctrl(el)
    RESULTS.set(el, result)
    attached.push(el)
  }
  return () => {
    for (const el of attached) {
      const result = RESULTS.get(el)
      try {
        result?.destroy?.()
      } catch {
        /* teardown must not throw through dispose */
      }
      RESULTS.delete(el)
      ATTACHED.delete(el)
    }
  }
}

/** Test helper: clear attach bookkeeping for a node (jsdom isolation). */
export function __resetElementControllerAttach(el?: Element): void {
  if (el) {
    RESULTS.delete(el)
    ATTACHED.delete(el)
    return
  }
}
