import type { SurfaceContract } from '../contract-types'

/** Surface contract for empty-state — axes from prop-registry only. */
export const contract = {
  slug: 'empty-state',
  tag: 'empty-state',
  kind: 'host',
  axes: {
    variant: {
      values: ['default', 'inline', 'placeholder', 'connecting'] as const,
      default: 'default',
      control: 'select',
    },
    size: {
      values: ['md', 'sm', 'lg'] as const,
      default: 'md',
      control: 'slider',
    },
    tone: {
      values: ['primary', 'info', 'success', 'warning', 'danger', 'accent', 'muted'] as const,
      default: 'muted',
      control: 'swatches',
    },
    compact: { default: false, control: 'switch', docsSkip: true },
  },
  faces: [
    { path: 'card', label: 'Background' },
    { path: 'card-foreground', label: 'Foreground' },
    { path: 'border', label: 'Border' },
  ],
  visual: {
    defaultRow: true,
    cross: ['variant', 'size', 'tone', 'compact'],
  },
  examples: true,
} as const satisfies SurfaceContract
