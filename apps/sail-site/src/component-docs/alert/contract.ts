import type { SurfaceContract } from '../contract-types'

/** Surface contract for alert — axes from prop-registry only. */
export const contract = {
  slug: 'alert',
  tag: 'alert',
  kind: 'host',
  axes: {
    tone: {
      values: ['muted', 'info', 'success', 'warning', 'destructive'] as const,
      optional: true,
      control: 'swatches',
    },
    dismissible: { default: false, control: 'switch' },
    onDismiss: { control: 'none', event: true },
  },
  faces: [
    { path: 'component.alert.bg', label: 'Fill' },
    { path: 'component.alert.fg', label: 'Ink' },
    { path: 'component.alert.border', label: 'Border' },
    { path: 'component.alert.stripe', label: 'Stripe' },
  ],
  visual: {
    defaultRow: true,
    cross: ['tone', 'dismissible'],
  },
  examples: true,
} as const satisfies SurfaceContract
