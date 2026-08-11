import type { SurfaceContract } from '../contract-types'

/** Surface contract for dropdown — placement string + onSelect event. */
export const contract = {
  slug: 'dropdown',
  tag: 'dropdown',
  kind: 'host',
  axes: {
    placement: { default: '', control: 'text' },
    onSelect: { control: 'none', event: true },
  },
  faces: [
    { path: 'component.dropdown.bg', label: 'Fill' },
    { path: 'component.dropdown.fg', label: 'Ink' },
    { path: 'component.dropdown.border', label: 'Border' },
  ],
  visual: {
    defaultRow: true,
    cross: ['placement'],
  },
  examples: true,
} as const satisfies SurfaceContract
