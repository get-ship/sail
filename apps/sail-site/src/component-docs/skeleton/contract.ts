import type { SurfaceContract } from '../contract-types'

/** Surface contract for skeleton — axes from prop-registry only. */
export const contract = {
  slug: 'skeleton',
  tag: 'skeleton',
  kind: 'host',
  axes: {
    shape: {
      values: ['text', 'line', 'block', 'circle'] as const,
      default: 'text',
      control: 'select',
    },
    size: {
      values: ['sm', 'md', 'lg'] as const,
      default: 'md',
      control: 'slider',
    },
  },
  faces: [
    { path: 'component.skeleton.bg', label: 'Band' },
    { path: 'component.skeleton.highlight', label: 'Highlight' },
  ],
  visual: {
    defaultRow: true,
    cross: ['shape', 'size'],
  },
  examples: true,
} as const satisfies SurfaceContract
