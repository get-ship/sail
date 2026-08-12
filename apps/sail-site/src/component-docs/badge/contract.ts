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
      values: ['primary', 'info', 'success', 'warning', 'destructive', 'accent', 'muted'] as const,
      optional: true,
      control: 'swatches',
    },
    size: {
      values: ['sm', 'md', 'lg'] as const,
      default: 'md',
      control: 'slider',
    },
    pill: { default: false, control: 'switch' },
  },
  faces: [
    { path: 'component.badge.bg', label: 'Fill' },
    { path: 'component.badge.fg', label: 'Ink' },
    { path: 'component.badge.border', label: 'Border' },
  ],
  visual: {
    defaultRow: true,
    cross: ['variant', 'tone', 'size', 'pill'],
  },
  examples: true,
} as const satisfies SurfaceContract
