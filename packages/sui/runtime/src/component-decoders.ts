// Per-`PropDef` decoder selection.
//
// Lifted out of `component-factory.ts` so `component-form.ts` can
// reuse it for `formStateRestoreCallback(state)` without dragging in
// the whole factory module (and creating a circular import).

import {
  decodeBoolean,
  decodeEnum,
  decodeNumber,
  decodeString,
  type Decoder,
} from './decode'
import type { PropDef } from './component-factory'

export const decoderFor = (def: PropDef<unknown>): Decoder<unknown> => {
  switch (def.type) {
    case 'string':
      return decodeString
    case 'bool':
      return decodeBoolean
    case 'number':
      return decodeNumber
    case 'union': {
      const values = def.values ?? []
      const fallback =
        (def.default as string | undefined) ??
        (values[0] as string | undefined) ??
        ''
      return decodeEnum(values, fallback) as Decoder<unknown>
    }
    case 'property':
      // Property props are never observed as attributes, so this decoder is
      // never invoked; identity keeps the switch exhaustive and safe.
      return (v) => v
  }
}
