import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for tabs — axes from prop-registry only.
 * Faces: tab + tab-list roles (no tabs-specific pack).
 * Strip look lives on <tab-list variant>; size Style on each <tab>.
 */
export const contract = {
  slug: 'tabs',
  tag: 'tabs',
  kind: 'host',
  axes: {
    active: { default: '', control: 'text', docsSkip: true },
    orientation: {
      values: ['horizontal', 'vertical'] as const,
      default: 'horizontal',
      control: 'select',
    },
    onTabChange: { control: 'none', event: true },
  },
  faces: [],
  visual: {
    defaultRow: true,
    cross: ['orientation'],
  },
  examples: true,
} as const satisfies SurfaceContract
