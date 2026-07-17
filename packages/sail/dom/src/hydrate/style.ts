// `style:NAME={value}` hydrator — sets a single inline-style property.
//
// Falsy values clear the property (mirrors how the SSR walker omits
// the style fragment when its expression is falsy).

import { effect } from '@ship/sail-runtime'

export function hydrateStyle(
  el: HTMLElement,
  prop: string,
  getter: () => unknown,
): () => void {
  return effect(() => {
    const v = getter()
    if (v == null || v === false || v === '') {
      el.style.removeProperty(prop)
      return
    }
    el.style.setProperty(prop, String(v))
  })
}
