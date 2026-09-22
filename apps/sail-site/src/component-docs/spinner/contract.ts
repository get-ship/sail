import type { SurfaceContract } from '../contract-types'

/** Surface contract for spinner — axes from prop-registry only. */
export const contract = {
  slug: 'spinner',
  tag: 'spinner',
  kind: 'host',
  axes: {
    size: {
      values: ['xs', 'sm', 'md', 'lg'] as const,
      default: 'md',
      control: 'select',
    },
    tone: {
      values: ['primary', 'info', 'success', 'warning', 'danger', 'accent'] as const,
      default: 'info',
      control: 'swatches',
    },
    // Centering is the parent's job. Keep the host prop but do not knob it.
    centered: { default: false, control: 'switch', docsSkip: true },
  },
  faces: [],
  visual: {
    defaultRow: true,
    cross: ['size', 'tone'],
  },
  examples: true,
} as const satisfies SurfaceContract
