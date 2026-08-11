import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for ticker — axes from prop-registry only.
 * No component.ticker.* leaves (author paint on host).
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
