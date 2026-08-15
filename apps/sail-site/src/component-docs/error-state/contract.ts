import type { SurfaceContract } from '../contract-types'

/** Surface contract for error-state — axes from prop-registry only. */
export const contract = {
  slug: 'error-state',
  tag: 'error-state',
  kind: 'host',
  axes: {
    variant: {
      values: ['default', 'inline'] as const,
      default: 'default',
      control: 'select',
    },
    size: {
      values: ['md', 'sm', 'lg'] as const,
      default: 'md',
      control: 'slider',
    },
    code: { default: '', control: 'text' },
  },
  faces: [
    { path: 'card', label: 'Background' },
    { path: 'card-foreground', label: 'Foreground' },
    { path: 'border', label: 'Border' },
  ],
  visual: {
    defaultRow: true,
    cross: ['variant', 'size', 'code'],
  },
  examples: true,
} as const satisfies SurfaceContract
