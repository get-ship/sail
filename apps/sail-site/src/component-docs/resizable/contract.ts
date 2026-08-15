import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for resizable — axes from stdlib props only.
 * orientation horizontal (default) or vertical. No resizable-specific roles pack.
 */
export const contract = {
  slug: 'resizable',
  tag: 'resizable',
  kind: 'host',
  axes: {
    orientation: {
      values: ['horizontal', 'vertical'] as const,
      default: 'horizontal',
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
