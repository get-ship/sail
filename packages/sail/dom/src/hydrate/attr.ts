// `attr:NAME={expr}` hydrator — reactive attribute value.
//
// Nullish or `false` removes the attribute (so authors can flip an
// attribute on/off via the signal value).

import { effect } from '@ship/sail-runtime'

export function hydrateAttr(
  el: Element,
  name: string,
  getter: () => unknown,
): () => void {
  return effect(() => {
    const v = getter()
    if (v == null || v === false) {
      el.removeAttribute(name)
      return
    }
    el.setAttribute(name, String(v))
  })
}
