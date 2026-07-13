// Light-DOM slot projection — the compiler-managed replacement for native
// <slot> assignment (which only exists inside shadow roots). A light
// component's original children are captured at connect, the template
// renders, then each captured node moves INTO its <slot> element, which
// stays in the tree as a display:contents wrapper (the light contracts
// sheet carries `slot { display: contents }`).
//
// Semantics mirrored from shadow slotting:
//   - a node with slot="name" goes to <slot name="name">
//   - nodes without a slot attribute (elements and non-empty text) go to
//     the default <slot>
//   - nodes with no matching slot are removed (unslotted content does not
//     render in shadow DOM either)
//   - slot fallback content is dropped when anything is assigned, kept
//     when nothing is
//
// KNOWN LIMIT (documented, not hidden): projection runs once at connect.
// Children appended to the host LATER are not re-projected — the dynamic
// composition path in SUI templates goes through compiled call sites, not
// raw DOM appends, so this covers the authored surface. Streaming-parse
// upgrades (element connects before its children parse) are the other
// gap; bundles load deferred today so the DOM is complete at registration.

/** Slots owned by `host` itself — not ones inside a nested component. */
const ownSlots = (host: HTMLElement): HTMLSlotElement[] =>
  Array.from(host.querySelectorAll('slot')).filter(
    (s) => s.closest('[data-component]') === host,
  )

const isProjectable = (n: Node): boolean => {
  // <template> is an inert declaration (controllers read it, nothing renders
  // it) — never moved, never removed by projection.
  if (n.nodeType === Node.ELEMENT_NODE) return (n as Element).tagName !== 'TEMPLATE'
  if (n.nodeType === Node.TEXT_NODE) return (n.textContent ?? '').trim().length > 0
  return false
}

/**
 * Project `captured` (the host's original children, in order) into the
 * <slot> elements the template rendered. Call AFTER the template mount.
 */
export function projectSlots(host: HTMLElement, captured: Node[]): void {
  const slots = ownSlots(host)
  if (slots.length === 0) {
    // No slots in the template — original children have no home (matches
    // shadow behavior where unslotted light DOM never renders). Templates
    // stay: they are declarations, not content.
    for (const n of captured) {
      if (isProjectable(n) || n.nodeType === Node.TEXT_NODE) (n as ChildNode).remove?.()
    }
    return
  }
  const named = new Map<string, HTMLSlotElement>()
  let fallback: HTMLSlotElement | null = null
  for (const s of slots) {
    const name = s.getAttribute('name')
    if (name) named.set(name, s)
    else fallback ??= s
  }
  const assigned = new Map<HTMLSlotElement, Node[]>()
  for (const n of captured) {
    const slotName =
      n.nodeType === Node.ELEMENT_NODE ? (n as Element).getAttribute('slot') : null
    const target = slotName ? (named.get(slotName) ?? null) : fallback
    if (!isProjectable(n) && n.nodeType === Node.ELEMENT_NODE) continue
    if (target && (slotName || isProjectable(n))) {
      const list = assigned.get(target) ?? []
      list.push(n)
      assigned.set(target, list)
    } else {
      ;(n as ChildNode).remove?.()
    }
  }
  for (const [slot, nodes] of assigned) {
    // Anything assigned replaces the slot's fallback content.
    slot.replaceChildren(...nodes)
  }
}
