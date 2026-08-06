// dismiss-self — shared dismiss path for autoDismiss + dismissControl.
// Not a `use:` action; keeps dual stamps from double-firing / double-removing.
// Invokes the host's `onDismiss` callback prop, then removes the node.

import { invokeCallback } from '../callback'

const dismissed = new WeakSet<Element>()

/** True after a successful dismissSelf (or if the node is already detached). */
export function isDismissed(el: Element): boolean {
  return dismissed.has(el) || !el.isConnected
}

/**
 * Invoke `onDismiss` then remove the element. Idempotent: a second call on
 * the same node is a no-op.
 */
export function dismissSelf(el: HTMLElement): boolean {
  if (dismissed.has(el)) return false
  dismissed.add(el)
  invokeCallback(el, 'onDismiss')
  el.remove()
  return true
}
