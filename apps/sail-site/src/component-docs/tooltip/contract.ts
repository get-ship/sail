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
    { path: 'component.tooltip.bg', label: 'Bubble fill' },
    { path: 'component.tooltip.fg', label: 'Bubble ink' },
  ],
  visual: {
    defaultRow: true,
    cross: ['placement', 'content'],
  },
  examples: true,
} as const satisfies SurfaceContract
