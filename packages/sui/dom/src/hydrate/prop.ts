// `prop:NAME={expr}` hydrator — assigns the JS property on the element.

import { bindProp } from '@ship/sui-runtime'

export function hydrateProp(
  el: Element,
  name: string,
  getter: () => unknown,
): () => void {
  return bindProp(el, name, getter)
}
