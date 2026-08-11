import type { SurfaceContract } from '../contract-types'

/** Surface contract for field — axes from prop-registry only. */
export const contract = {
  slug: 'field',
  tag: 'field',
  kind: 'host',
  axes: {
    invalid: { default: false, control: 'switch' },
  },
  faces: [
    { path: 'component.field.bg', label: 'Fill' },
    { path: 'component.field.border', label: 'Border' },
    { path: 'component.field.border-hover', label: 'Border hover' },
    { path: 'component.field.error', label: 'Error' },
    { path: 'component.field.fg', label: 'Ink' },
    { path: 'component.field.hint', label: 'Hint' },
  ],
  visual: {
    defaultRow: true,
    cross: ['invalid'],
  },
  examples: true,
} as const satisfies SurfaceContract
