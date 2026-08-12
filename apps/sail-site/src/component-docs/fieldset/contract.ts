import type { SurfaceContract } from '../contract-types'

/** Surface contract for fieldset — disabled only; legend documented nested. */
export const contract = {
  slug: 'fieldset',
  tag: 'fieldset',
  kind: 'host',
  axes: {
    disabled: { default: false, control: 'switch' },
  },
  faces: [{ path: 'component.fieldset.border', label: 'Border' }],
  visual: {
    defaultRow: true,
    cross: ['disabled'],
  },
  examples: true,
} as const satisfies SurfaceContract
