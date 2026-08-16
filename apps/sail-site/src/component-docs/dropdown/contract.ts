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
    { path: 'popover', label: 'Background' },
    { path: 'foreground', label: 'Foreground' },
    { path: 'border', label: 'Border' },
  ],
  visual: {
    defaultRow: true,
    cross: ['placement'],
  },
  examples: true,
} as const satisfies SurfaceContract
