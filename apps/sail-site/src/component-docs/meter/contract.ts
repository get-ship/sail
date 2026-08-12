import type { SurfaceContract } from '../contract-types'

/** Surface contract for meter — axes from prop-registry only. */
export const contract = {
  slug: 'meter',
  tag: 'meter',
  kind: 'host',
  axes: {
    value: { default: 0, control: 'number' },
    min: { default: 0, control: 'number', docsSkip: true },
    max: { default: 1, control: 'number', docsSkip: true },
    low: { default: 0, control: 'number', docsSkip: true },
    high: { default: 1, control: 'number', docsSkip: true },
    optimum: { default: 1, control: 'number', docsSkip: true },
    size: {
      values: ['sm', 'md', 'lg'] as const,
      default: 'md',
      control: 'slider',
    },
  },
  faces: [
    { path: 'component.meter.track', label: 'Track' },
    { path: 'component.meter.optimum', label: 'Optimum' },
    { path: 'component.meter.suboptimum', label: 'Suboptimum' },
    { path: 'component.meter.sub-suboptimum', label: 'Even less good' },
  ],
  visual: {
    defaultRow: true,
    cross: ['value', 'size'],
  },
  examples: true,
} as const satisfies SurfaceContract
