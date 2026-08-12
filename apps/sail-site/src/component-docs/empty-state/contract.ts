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
      values: ['primary', 'info', 'success', 'warning', 'destructive', 'accent', 'muted'] as const,
      optional: true,
      control: 'swatches',
    },
  },
  faces: [
    { path: 'component.empty-state.bg', label: 'Fill' },
    { path: 'component.empty-state.border', label: 'Border' },
    { path: 'component.empty-state.fg', label: 'Ink' },
    { path: 'component.empty-state.title', label: 'Title' },
  ],
  visual: {
    defaultRow: true,
    cross: ['variant', 'size', 'tone'],
  },
  examples: true,
} as const satisfies SurfaceContract
