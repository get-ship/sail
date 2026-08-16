import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for accordion — axes from prop-registry only.
 * Faces: card / card-foreground / border.
 */
export const contract = {
  slug: 'accordion',
  tag: 'accordion',
  kind: 'host',
  axes: {
    open: { default: false, control: 'switch' },
    disabled: { default: false, control: 'switch' },
    name: { default: '', control: 'text' },
    multiple: { default: false, control: 'switch' },
    onToggle: { control: 'none', event: true },
  },
  faces: [
    { path: 'card', label: 'Background' },
    { path: 'card-foreground', label: 'Foreground' },
    { path: 'border', label: 'Border' },
  ],
  visual: {
    defaultRow: true,
    cross: ['open', 'disabled', 'name', 'multiple'],
  },
  examples: true,
} as const satisfies SurfaceContract
