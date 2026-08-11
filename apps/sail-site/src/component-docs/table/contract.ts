import type { SurfaceContract } from '../contract-types'

/** Surface contract for table — axes from prop-registry only. */
export const contract = {
  slug: 'table',
  tag: 'table',
  kind: 'host',
  axes: {
    dense: { default: false, control: 'switch' },
    striped: { default: false, control: 'switch' },
    hoverable: { default: false, control: 'switch' },
    bordered: { default: false, control: 'switch' },
  },
  faces: [
    { path: 'component.table.fg', label: 'Ink' },
    { path: 'component.table.muted-fg', label: 'Muted ink' },
    { path: 'component.table.sticky-bg', label: 'Sticky fill' },
    { path: 'component.table.stripe', label: 'Stripe' },
  ],
  visual: {
    defaultRow: true,
    cross: ['dense', 'striped', 'hoverable', 'bordered'],
  },
  examples: true,
} as const satisfies SurfaceContract
