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
  // Where the content lives depends on the DOM kind: shadow assigns the
  // host's light children to the slot (assignedNodes); light projection
  // moves them INTO the slot wrapper (childNodes). This is decided LAZILY
  // per read — the slot may still be detached (built in a fragment) at bind
  // time, so a once-at-bind decision mis-routes (the sail-empty-state
  // reflect-empty regression). Duck-type the shadow root by `.host`
  // (`instanceof ShadowRoot` is unreliable under jsdom).
  const nodesOf = (): Node[] => {
    const root = slot.getRootNode() as { host?: unknown } | null
    const inShadow = !!(root && root !== slot.ownerDocument && root.host)
    return inShadow ? slot.assignedNodes({ flatten: true }) : Array.from(slot.childNodes)
  }
  const sync = (): void => {
    const hasText = nodesOf().some(hasVisibleContent)
    if (hasText) host.removeAttribute(attrName)
    else host.setAttribute(attrName, '')
  }
  // Universal change signal: observe the HOST subtree — it catches shadow
  // light-child assignment AND light projection (children moved into the
  // wrapper), regardless of when the slot attaches. slotchange rides along
  // for shadow browsers where it fires precisely (harmless where it doesn't).
  slot.addEventListener('slotchange', sync)
  let observer: MutationObserver | null = null
  if (typeof MutationObserver !== 'undefined') {
    observer = new MutationObserver(sync)
    observer.observe(host, { childList: true, subtree: true, characterData: true })
  }
  sync()
  return () => {
    slot.removeEventListener('slotchange', sync)
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
