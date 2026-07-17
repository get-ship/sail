// Event-handler hydrator (`on:<event>`).
//
// Thin wrapper over the runtime `on()` so direct-emit hydrate paths
// share the modifier pipeline with `__mount`. The compiler resolves
// the handler closure at hydrate time (via the same IIFE pattern it
// uses in `__mount`) and passes it in alongside the modifier list.

import { on, type Modifier } from '@ship/sail-runtime'

export function hydrateOn(
  el: Element,
  event: string,
  handler: (e: Event) => unknown,
  modifiers: readonly Modifier[] = [],
): () => void {
  return on(el, event, handler, { modifiers: modifiers as Modifier[] })
}
