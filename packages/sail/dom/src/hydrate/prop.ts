// Property hydrator for SSR plain-field bindings lowered to `bindProp`
// (JS property channel). Assigns the live property on the element.

import { bindProp } from '@ship/sail-runtime'

export function hydrateProp(
  el: Element,
  name: string,
  getter: () => unknown,
): () => void {
  return bindProp(el, name, getter)
}
