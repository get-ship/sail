import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for dialog — axes from prop-registry only.
 * Overlay behavior depth stays operator later; scaffold props + static anatomy.
 */
export const contract = {
  slug: 'dialog',
  tag: 'dialog',
  kind: 'host',
  axes: {
    size: {
      values: ['sm', 'md', 'lg', 'full'] as const,
      default: 'md',
      control: 'select',
    },
    backdrop: {
      values: ['default', 'blur', 'solid'] as const,
      default: 'default',
      control: 'select',
    },
    side: {
      values: ['center', 'start', 'end', 'top', 'bottom'] as const,
      default: 'center',
      control: 'select',
    },
    dismissible: { default: true, control: 'switch' },
    onClose: { control: 'none', event: true },
    onDismiss: { control: 'none', event: true },
  },
  faces: [
    { path: 'popover', label: 'Background' },
    { path: 'foreground', label: 'Foreground' },
    { path: 'border', label: 'Border' },
  ],
  visual: {
    defaultRow: true,
    cross: ['size', 'backdrop', 'side', 'dismissible'],
  },
  examples: true,
} as const satisfies SurfaceContract
