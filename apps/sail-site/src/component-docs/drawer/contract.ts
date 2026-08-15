import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for drawer — axes from stdlib props only.
 * No drawer-specific roles pack; host paints card / card-foreground.
 */
export const contract = {
  slug: 'drawer',
  tag: 'drawer',
  kind: 'host',
  axes: {
    size: {
      values: ['sm', 'md', 'lg'] as const,
      default: 'md',
      control: 'select',
    },
    side: {
      values: ['start', 'end', 'top', 'bottom'] as const,
      default: 'end',
      control: 'select',
    },
    dismissible: { default: true, control: 'switch' },
    onClose: { control: 'none', event: true },
    onDismiss: { control: 'none', event: true },
  },
  faces: [
    { path: 'card', label: 'Background' },
    { path: 'card-foreground', label: 'Foreground' },
    { path: 'border', label: 'Border' },
  ],
  visual: {
    defaultRow: true,
    cross: ['size', 'side', 'dismissible'],
  },
  examples: true,
} as const satisfies SurfaceContract
