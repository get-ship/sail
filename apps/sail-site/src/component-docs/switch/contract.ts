import type { SurfaceContract } from '../contract-types'

/** Surface contract for switch — axes from prop-registry only. */
export const contract = {
  slug: 'switch',
  tag: 'switch',
  kind: 'host',
  axes: {
    checked: { default: false, control: 'switch' },
    disabled: { default: false, control: 'switch' },
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
  },
  faces: [
    { path: 'component.switch.track', label: 'Track' },
    { path: 'component.switch.border', label: 'Border' },
    { path: 'component.switch.thumb', label: 'Thumb' },
  ],
  visual: {
    defaultRow: true,
    cross: ['checked', 'disabled', 'tone', 'size'],
  },
  examples: true,
} as const satisfies SurfaceContract
