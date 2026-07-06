// `<slot reflect-empty="<attr>">` runtime helper.
//
// Wires a `slotchange` listener that toggles the named boolean
// attribute on the host element based on whether the slot's assigned
// content carries any visible text. Mirrors the imperative pattern
// every Lit primitive (`ship-button`, `ship-list-tile`, `ship-chip`,
// `ship-badge`, `ship-empty-state`, …) used to hand-roll.
//
// Counts as "visible content":
//   - any text node whose trimmed text is non-empty
//   - any element node whose `textContent` trims non-empty OR which
//     has element children of its own
//
// Returns a disposer; cleans up the listener on dispose.

/**
 * Wire `<slot>` emptiness to a host attribute.
 *
 * The compiler emits one of these per `<slot reflect-empty>` directive
 * inside the component's setup body. Authors do not call this directly.
 *
 * @param slot     The rendered `<slot>` element.
 * @param host     The component's host element (gets `setAttribute` /
 *                 `removeAttribute` calls based on slot state).
 * @param attrName The boolean attribute to toggle on the host.
 * @returns Disposer that removes the slotchange listener.
 */
export function bindSlotEmpty(
  slot: HTMLSlotElement,
  host: HTMLElement,
  attrName: string,
): () => void {
  // Light DOM: the slot is a plain projection wrapper (no shadow tree) —
  // assignedNodes() is always [], the projected content lives INSIDE the
  // slot as real children. Detect by root node.
  const inShadow =
    typeof ShadowRoot !== 'undefined' && slot.getRootNode() instanceof ShadowRoot
  const nodesOf = (): Node[] =>
    inShadow ? slot.assignedNodes({ flatten: true }) : Array.from(slot.childNodes)
  const sync = (): void => {
    const hasText = nodesOf().some(hasVisibleContent)
    if (hasText) host.removeAttribute(attrName)
    else host.setAttribute(attrName, '')
  }
  // Shadow: slotchange is the change signal. Light: there is none — watch
  // the wrapper's children (projection lands them after mount).
  let observer: MutationObserver | null = null
  if (inShadow) {
    slot.addEventListener('slotchange', sync)
  } else if (typeof MutationObserver !== 'undefined') {
    observer = new MutationObserver(sync)
    observer.observe(slot, { childList: true, subtree: true, characterData: true })
  }
  // Content may already be in place at attach time without a change event
  // firing — sync immediately so the host attribute reflects the initial
  // state.
  sync()
  return () => {
    if (inShadow) slot.removeEventListener('slotchange', sync)
    observer?.disconnect()
  }
}

function hasVisibleContent(n: Node): boolean {
  if (n.nodeType === Node.TEXT_NODE) {
    return (n.textContent ?? '').trim().length > 0
  }
  if (n.nodeType === Node.ELEMENT_NODE) {
    const el = n as Element
    return (el.textContent ?? '').trim().length > 0 || el.childElementCount > 0
  }
  return false
}
