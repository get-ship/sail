// dismiss-self — shared dismiss path for autoDismiss + dismissControl.
// Not a `use:` action; keeps dual stamps from double-firing / double-removing.
// Callers pick the event name (toast → sail-toast-dismiss, alert → sail-dismiss).

const dismissed = new WeakSet<Element>()

/** True after a successful dismissSelf (or if the node is already detached). */
export function isDismissed(el: Element): boolean {
  return dismissed.has(el) || !el.isConnected
}

/**
 * Emit a composed dismiss event then remove the element. Idempotent: a second
 * call on the same node is a no-op.
 */
export function dismissSelf(
  el: HTMLElement,
  eventName = 'sail-toast-dismiss',
): boolean {
  if (dismissed.has(el)) return false
  dismissed.add(el)
  el.dispatchEvent(new CustomEvent(eventName, { bubbles: true, composed: true }))
  el.remove()
  return true
}
