import type { SurfaceContract } from '../contract-types'

/** Surface contract for spinner — axes from prop-registry only. */
export const contract = {
  slug: 'spinner',
  tag: 'spinner',
  kind: 'host',
  axes: {
    size: {
      values: ['xs', 'sm', 'md', 'lg'] as const,
      default: 'md',
      control: 'slider',
    },
    tone: {
      values: ['primary', 'info', 'success', 'warning', 'destructive', 'accent'] as const,
      default: 'info',
      control: 'swatches',
    },
    centered: { default: false, control: 'switch' },
  },
  faces: [],
  visual: {
    defaultRow: true,
    cross: ['size', 'tone', 'centered'],
  },
  examples: true,
} as const satisfies SurfaceContract
