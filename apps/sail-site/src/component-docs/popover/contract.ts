import type { SurfaceContract } from '../contract-types'

/** Surface contract for popover — placement string only (behavior = operator later). */
export const contract = {
  slug: 'popover',
  tag: 'popover',
  kind: 'host',
  axes: {
    placement: { default: '', control: 'text' },
  },
  faces: [
    { path: 'component.popover.bg', label: 'Fill' },
    { path: 'component.popover.fg', label: 'Ink' },
    { path: 'component.popover.border', label: 'Border' },
  ],
  visual: {
    defaultRow: true,
    cross: ['placement'],
  },
  examples: true,
} as const satisfies SurfaceContract
