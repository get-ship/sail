import type { SurfaceContract } from '../contract-types'

/** Surface contract for scroll-area — axes from prop-registry only. */
export const contract = {
  slug: 'scroll-area',
  tag: 'scroll-area',
  kind: 'host',
  axes: {
    orientation: {
      values: ['vertical', 'horizontal', 'both'] as const,
      default: 'vertical',
      control: 'select',
    },
  },
  faces: [],
  visual: {
    defaultRow: true,
    cross: ['orientation'],
  },
  examples: true,
} as const satisfies SurfaceContract
