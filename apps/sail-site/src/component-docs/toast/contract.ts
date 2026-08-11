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
        'destructive',
        'accent',
        'muted',
      ] as const,
      optional: true,
      control: 'swatches',
    },
    duration: { default: '', control: 'text' },
    onDismiss: { control: 'none', event: true },
  },
  faces: [
    { path: 'component.toast.bg', label: 'Fill' },
    { path: 'component.toast.fg', label: 'Ink' },
    { path: 'component.toast.border', label: 'Border' },
    { path: 'component.toast.stripe', label: 'Stripe' },
  ],
  visual: {
    defaultRow: true,
    cross: ['tone', 'duration'],
  },
  examples: true,
} as const satisfies SurfaceContract
