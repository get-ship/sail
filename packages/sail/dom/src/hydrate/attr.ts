// Reactive attribute hydrator for SSR plain-field bindings lowered to
// `bindAttr` / attribute channel. Nullish or `false` removes the attribute.

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
