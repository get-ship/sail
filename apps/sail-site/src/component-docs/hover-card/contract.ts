import type { SurfaceContract } from '../contract-types'

/** Surface contract for hover-card — placement string only. */
export const contract = {
  slug: 'hover-card',
  tag: 'hover-card',
  kind: 'host',
  axes: {
    placement: { default: '', control: 'text' },
  },
  faces: [
    { path: 'component.hover-card.bg', label: 'Fill' },
    { path: 'component.hover-card.fg', label: 'Ink' },
    { path: 'component.hover-card.border', label: 'Border' },
  ],
  visual: {
    defaultRow: true,
    cross: ['placement'],
  },
  examples: true,
} as const satisfies SurfaceContract
