import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for alert — axes from prop-registry only.
 * Variants: default | banner (SPEC-L1-PASS-1).
 */
export const contract = {
  slug: 'alert',
  tag: 'alert',
  kind: 'host',
  axes: {
    tone: {
      values: ['muted', 'info', 'success', 'warning', 'danger'] as const,
      default: 'info',
      control: 'swatches',
    },
    variant: {
      values: ['default', 'banner'] as const,
      default: 'default',
      control: 'select',
      docsSkip: true,
    },
    dismissible: { default: false, control: 'switch' },
    onDismiss: { control: 'none', event: true },
  },
  faces: [],
  visual: {
    defaultRow: true,
    cross: ['tone', 'variant', 'dismissible'],
  },
  examples: true,
} as const satisfies SurfaceContract
