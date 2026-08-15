import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for color — axes from stdlib props only.
 * System swatch (native type=color). No component.color.* pack.
 */
export const contract = {
  slug: 'color',
  tag: 'color',
  kind: 'host',
  axes: {
    value: { default: '#000000', control: 'text' },
    disabled: { default: false, control: 'switch' },
    invalid: { default: false, control: 'switch' },
    name: { default: '', control: 'text', docsSkip: true },
  },
  faces: [],
  visual: {
    defaultRow: true,
    cross: ['value', 'disabled', 'invalid'],
  },
  examples: true,
} as const satisfies SurfaceContract
