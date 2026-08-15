import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for slider — axes from prop-registry only.
 * Single thumb only (no dual-thumb range invent).
 */
export const contract = {
  slug: 'slider',
  tag: 'slider',
  kind: 'host',
  axes: {
    value: { default: 0, control: 'number' },
    min: { default: 0, control: 'number', docsSkip: true },
    max: { default: 100, control: 'number', docsSkip: true },
    step: { default: 1, control: 'number', docsSkip: true },
    disabled: { default: false, control: 'switch' },
    size: {
      values: ['sm', 'md', 'lg'] as const,
      default: 'md',
      control: 'slider',
    },
    name: { default: '', control: 'text', docsSkip: true },
  },
  faces: [],
  visual: {
    defaultRow: true,
    cross: ['value', 'size', 'disabled'],
  },
  examples: true,
} as const satisfies SurfaceContract
