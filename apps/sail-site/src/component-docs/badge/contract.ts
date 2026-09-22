import type { SurfaceContract } from '../contract-types'

/** Surface contract for badge — axes from prop-registry only. */
export const contract = {
  slug: 'badge',
  tag: 'badge',
  kind: 'host',
  axes: {
    variant: {
      values: ['soft', 'solid', 'outline'] as const,
      default: 'soft',
      control: 'select',
    },
    tone: {
      values: ['primary', 'info', 'success', 'warning', 'danger', 'accent', 'muted'] as const,
      default: 'muted',
      control: 'swatches',
    },
    size: {
      values: ['sm', 'md', 'lg'] as const,
      default: 'md',
      control: 'slider',
    },
    pill: { default: false, control: 'switch' },
    dot: { default: false, control: 'switch', docsSkip: true },
  },
  faces: [],
  visual: {
    defaultRow: true,
    cross: ['variant', 'tone', 'size', 'pill', 'dot'],
  },
  examples: true,
} as const satisfies SurfaceContract
