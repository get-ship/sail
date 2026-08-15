import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for toggle-group — segmented control host.
 * Keep as host; no top-level toggle/ package. Faces: host rail only
 * (muted / border). Toggle cell leaves stay on <toggle>.
 */
export const contract = {
  slug: 'toggle-group',
  tag: 'toggle-group',
  kind: 'host',
  axes: {
    type: {
      values: ['single', 'multiple'] as const,
      default: 'single',
      control: 'select',
    },
    value: { default: '', control: 'text' },
    variant: {
      values: ['outline', 'ghost', 'segmented'] as const,
      default: 'outline',
      control: 'select',
    },
    orientation: {
      values: ['horizontal', 'vertical'] as const,
      default: 'horizontal',
      control: 'select',
    },
    onToggleChange: { control: 'none', event: true },
  },
  faces: [
    { path: 'muted', label: 'Background' },
    { path: 'border', label: 'Border' },
  ],
  visual: {
    defaultRow: true,
    cross: ['type', 'value', 'variant', 'orientation'],
  },
  examples: true,
} as const satisfies SurfaceContract
