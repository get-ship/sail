import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for table — axes from prop-registry only.
 * sticky-header (SPEC-L1-PASS-1): thead/th sticky via descendant map.
 */
export const contract = {
  slug: 'table',
  tag: 'table',
  kind: 'host',
  axes: {
    dense: { default: false, control: 'switch' },
    striped: { default: false, control: 'switch' },
    hoverable: { default: false, control: 'switch' },
    bordered: { default: false, control: 'switch' },
    'sticky-header': { default: false, control: 'switch', docsSkip: true },
  },
  faces: [],
  visual: {
    defaultRow: true,
    cross: ['dense', 'striped', 'hoverable', 'bordered', 'sticky-header'],
  },
  examples: true,
} as const satisfies SurfaceContract
