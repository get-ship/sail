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
      optional: true,
      control: 'swatches',
    },
    centered: { default: false, control: 'switch' },
  },
  faces: [
    { path: 'component.spinner.fg', label: 'Arc' },
    { path: 'component.spinner.track', label: 'Track' },
  ],
  visual: {
    defaultRow: true,
    cross: ['size', 'tone', 'centered'],
  },
  examples: true,
} as const satisfies SurfaceContract
