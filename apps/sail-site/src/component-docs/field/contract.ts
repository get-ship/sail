import type { SurfaceContract } from '../contract-types'

/** Surface contract for field — axes from prop-registry only. */
export const contract = {
  slug: 'field',
  tag: 'field',
  kind: 'host',
  axes: {
    invalid: { default: false, control: 'switch' },
  },
  faces: [
    { path: 'input', label: 'Background' },
    { path: 'border', label: 'Border' },
    { path: 'foreground', label: 'Foreground' },
  ],
  visual: {
    defaultRow: true,
    cross: ['invalid'],
  },
  examples: true,
} as const satisfies SurfaceContract
