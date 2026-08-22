import type { SurfaceContract } from '../contract-types'

/** Surface contract for toast — axes from prop-registry only. */
export const contract = {
  slug: 'toast',
  tag: 'toast',
  kind: 'host',
  axes: {
    tone: {
      values: [
        'primary',
        'info',
        'success',
        'warning',
        'danger',
        'accent',
        'muted',
      ] as const,
      default: 'info',
      control: 'swatches',
    },
    duration: { default: '', control: 'text' },
    onDismiss: { control: 'none', event: true },
  },
  faces: [
    { path: 'popover', label: 'Background' },
    { path: 'foreground', label: 'Foreground' },
    { path: 'border', label: 'Border' },
  ],
  visual: {
    defaultRow: true,
    cross: ['tone', 'duration'],
  },
  examples: true,
} as const satisfies SurfaceContract
