import type { SurfaceContract } from '../contract-types'

/** Surface contract for separator — axes from prop-registry only. */
export const contract = {
  slug: 'separator',
  tag: 'separator',
  kind: 'host',
  axes: {
    orientation: {
      values: ['horizontal', 'vertical'] as const,
      default: 'horizontal',
      control: 'select',
    },
    variant: {
      values: ['solid', 'dashed'] as const,
      default: 'solid',
      control: 'select',
    },
    tone: {
      values: ['muted', 'primary', 'destructive'] as const,
      optional: true,
      control: 'swatches',
    },
    // Accessible name only — visible labeled chrome is a language gap on void <hr>.
    label: { default: '', control: 'text' },
  },
  faces: [{ path: 'component.separator.line', label: 'Line' }],
  visual: {
    defaultRow: true,
    cross: ['orientation', 'variant', 'tone'],
  },
  examples: true,
} as const satisfies SurfaceContract
