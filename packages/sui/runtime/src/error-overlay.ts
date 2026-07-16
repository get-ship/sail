// Fallback error-surface channel for the deck dev overlay.
//
// `runProtected`/`runEffect`/the component setup catch sites already route
// a throw to the nearest `<sui-error-boundary>` when one is in scope. When
// none is, the throw only ever reached `console.error` — invisible unless
// a developer already has devtools open (task #80: a page that rendered
// nothing, dozens of throws, zero visible failure surface). This module
// adds a second, structured channel for that no-boundary case: a
// `sui:error` CustomEvent a dev-mode overlay can listen for. It is a
// fallback surface, not a boundary replacement — call it only from the
// same branch that already falls back to `console.error`.

/** Origin of an error reaching the fallback channel. Mirrors the subset of
 *  `BoundarySource` that has no boundary to catch it structurally — render
 *  throws are always routed through a boundary or `runProtected`'s own
 *  console.error, so they're out of scope here. */
export type SuiErrorPhase = 'effect' | 'setup'

export interface SuiErrorDetail {
  error: unknown
  phase: SuiErrorPhase
  host?: Element
  tag?: string
}

/**
 * Dispatch the `sui:error` fallback event. Bubbles + composed from `host`
 * when it's an element (so an ancestor overlay listener catches it even
 * across shadow-root boundaries); dispatched on `document` otherwise.
 * `tag` defaults to `host.dataset.component`, set by the component wrapper
 * before setup/effects can run.
 */
export function dispatchSuiError(host: Node | null, error: unknown, phase: SuiErrorPhase): void {
  const detail: SuiErrorDetail = { error, phase }
  let target: EventTarget = document
  if (host instanceof Element) {
    detail.host = host
    target = host
    const tag = (host as HTMLElement).dataset?.component
    if (tag) detail.tag = tag
  }
  target.dispatchEvent(new CustomEvent<SuiErrorDetail>('sui:error', { detail, bubbles: true, composed: true }))
}
