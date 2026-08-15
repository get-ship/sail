import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for navbar — axes from prop-registry only.
 * floating inset chrome (SPEC-L1-PASS-1). solid-on-scroll is a declared
 * axis; Style cannot key on scroll (language gap).
 */
export const contract = {
  slug: 'navbar',
  tag: 'navbar',
  kind: 'host',
  axes: {
    placement: {
      values: ['top', 'bottom', 'rail'] as const,
      default: 'top',
      control: 'select',
    },
    sticky: { default: false, control: 'switch' },
    bordered: { default: false, control: 'switch' },
    autohide: { default: false, control: 'switch' },
    floating: { default: false, control: 'switch', docsSkip: true },
    // Style cannot key on scroll — language gap (SPEC-L1-BASE-PASS out of scope).
    'solid-on-scroll': { default: false, control: 'switch', docsSkip: true },
    offset: { default: 0, control: 'number' },
  },
  faces: [],
  visual: {
    defaultRow: true,
    cross: ['placement', 'sticky', 'bordered', 'autohide', 'floating', 'offset'],
  },
  examples: true,
} as const satisfies SurfaceContract
