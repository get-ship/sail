import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for ticker — zero-props, canonical loop from prop-registry.
 * No ticker-specific pack (author Style on host).
 */
export const contract = {
  slug: 'ticker',
  tag: 'ticker',
  kind: 'host',
  axes: {},
  faces: [],
  visual: {
    defaultRow: true,
    cross: [],
  },
  examples: true,
} as const satisfies SurfaceContract