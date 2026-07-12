// delegate.ts — event delegation.
//
// `addEventListener` is not free. A 1,000-row table with a click handler on the label
// and another on the remove button costs TWO THOUSAND addEventListener calls, each
// allocating a listener record in the browser, and two thousand cleanup closures to
// tear them down again. That is most of what is left of our creation gap: Solid and
// Svelte both delegate; we did not.
//
// Delegation installs ONE listener per event TYPE on the document, and stores the
// per-element handler as a property on the node itself (`el.$$click`). On dispatch we
// walk up from `event.target` and call whatever we find. Creating a row now costs one
// property assignment per handler instead of an addEventListener — and removing a row
// costs NOTHING, because the property dies with the node. No cleanup closure, no
// removeEventListener.
//
// WHAT CANNOT BE DELEGATED, and is therefore left alone (the compiler decides, and
// falls back to a real listener):
//
//   * NON-BUBBLING events — focus, blur, mouseenter, mouseleave, load, scroll. There
//     is nothing to catch at the root. (`focusin`/`focusout` DO bubble and are on the
//     list; `focus`/`blur` are not.)
//   * MODIFIERS (`prevent` / `stop` / `stopimmediate` / `self` / `trusted` / `key`).
//     Their semantics live in the `on()` runtime, and several of them —
//     `stopimmediate` above all — mean something specific about listener ORDER on a
//     single element, which a dispatch with exactly one listener, at the root, cannot
//     faithfully reproduce. Rather than delegate the easy ones and quietly change the
//     meaning of the hard ones, any modified handler keeps its real listener.
//     Delegation is an optimization; it may not alter semantics.
//
// So this is an optimization with a fallback, not a second way to write an event
// handler: `on:click` is `on:click` either way, and the author never sees the seam.

const DELEGATED = new Set<string>()

/**
 * Ensure a root-level listener exists for each named event. Idempotent — the compiler
 * emits one call per module and repeated names are ignored.
 */
export function delegateEvents(names: readonly string[]): void {
  if (typeof document === 'undefined') return
  for (const name of names) {
    if (DELEGATED.has(name)) continue
    DELEGATED.add(name)
    document.addEventListener(name, dispatchDelegated)
  }
}

type Delegatable = Node & Record<string, ((e: Event) => void) | undefined>

function dispatchDelegated(e: Event): void {
  const key = `$$${e.type}`
  let node = e.target as Delegatable | null

  // Walk toward the root, calling handlers as we find them — this reproduces
  // bubbling, including the part authors rely on: `stopPropagation()` in a handler
  // stops the ones above it.
  while (node) {
    const handler = node[key]
    if (typeof handler === 'function') {
      handler.call(node, e)
      if (e.cancelBubble) return
    }
    // `parentNode` rather than `parentElement`: a handler can sit on the element
    // above a text node's parent, and a ShadowRoot's `host` continues the walk out
    // of a shadow tree (light DOM is the default, but components may opt in).
    const parent = node.parentNode as Delegatable | null
    node = parent ?? ((node as unknown as { host?: Delegatable }).host ?? null)
  }
}
