import type { SurfaceContract } from '../contract-types'

/** Surface contract for label — axes from prop-registry only. */
export const contract = {
  slug: 'label',
  tag: 'label',
  kind: 'host',
  axes: {
    size: {
      values: ['sm', 'md', 'lg'] as const,
      default: 'md',
      control: 'slider',
    },
    required: { default: false, control: 'switch' },
  },
  faces: [
    { path: 'component.label.fg', label: 'Ink' },
    { path: 'component.label.required', label: 'Required marker' },
  ],
  visual: {
    defaultRow: true,
    cross: ['size', 'required'],
  },
  examples: true,
} as const satisfies SurfaceContract
