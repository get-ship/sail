import type { SurfaceContract } from '../contract-types'

/** Surface contract for radio — axes from prop-registry only. */
export const contract = {
  slug: 'radio',
  tag: 'radio',
  kind: 'host',
  axes: {
    value: { default: '', control: 'select' },
    disabled: { default: false, control: 'switch' },
  },
  faces: [{ path: 'component.radio.fg', label: 'Label ink' }],
  visual: {
    defaultRow: true,
    cross: ['disabled'],
  },
  examples: true,
} as const satisfies SurfaceContract
