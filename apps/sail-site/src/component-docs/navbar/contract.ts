import type { SurfaceContract } from '../contract-types'

/** Surface contract for navbar — axes from prop-registry only. */
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
    offset: { default: 0, control: 'number' },
  },
  faces: [
    { path: 'component.navbar.bg', label: 'Fill' },
    { path: 'component.navbar.fg', label: 'Ink' },
    { path: 'component.navbar.border', label: 'Border' },
  ],
  visual: {
    defaultRow: true,
    cross: ['placement', 'sticky', 'bordered', 'autohide', 'offset'],
  },
  examples: true,
} as const satisfies SurfaceContract
