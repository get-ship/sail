import type { SurfaceContract } from '../contract-types'

/** Surface contract for tooltip — axes from prop-registry only. */
export const contract = {
  slug: 'tooltip',
  tag: 'tooltip',
  kind: 'host',
  axes: {
    placement: {
      values: ['top', 'bottom', 'left', 'right'] as const,
      default: 'top',
      control: 'select',
    },
    content: { default: '', control: 'text' },
  },
  faces: [
    { path: 'popover', label: 'Background' },
    { path: 'foreground', label: 'Foreground' },
    { path: 'border', label: 'Border' },
  ],
  visual: {
    defaultRow: true,
    cross: ['placement', 'content'],
  },
  examples: true,
} as const satisfies SurfaceContract
