import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for ticker — axes from prop-registry only.
 * No ticker-specific pack (author Style on host).
 */
export const contract = {
  slug: 'ticker',
  tag: 'ticker',
  kind: 'host',
  axes: {
    duration: {
      values: ['fast', 'normal', 'slow'] as const,
      default: 'normal',
      control: 'select',
    },
    reverse: { default: false, control: 'switch' },
  },
  faces: [],
  visual: {
    defaultRow: true,
    cross: ['duration', 'reverse'],
  },
  examples: true,
} as const satisfies SurfaceContract
