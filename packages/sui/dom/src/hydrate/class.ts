// `class:NAME={cond}` hydrator — toggles a class on truthy / off on falsy.

import { effect } from '@ship/sui-runtime'

export function hydrateClass(
  el: Element,
  name: string,
  getter: () => unknown,
): () => void {
  return effect(() => {
    el.classList.toggle(name, !!getter())
  })
}
