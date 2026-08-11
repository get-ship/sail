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
  faces: [
    { path: 'component.scroll-area.thumb', label: 'Thumb' },
    { path: 'component.scroll-area.thumb-hover', label: 'Thumb hover' },
  ],
  visual: {
    defaultRow: true,
    cross: ['orientation'],
  },
  examples: true,
} as const satisfies SurfaceContract
