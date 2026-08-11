import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for tabs — axes from prop-registry only.
 * Faces via FACE_MERGE: component.tab* + component.tab-list* (no component.tabs.*).
 * Strip look lives on <tab-list variant>; size Style on each <tab>.
 */
export const contract = {
  slug: 'tabs',
  tag: 'tabs',
  kind: 'host',
  axes: {
    active: { default: '', control: 'text' },
    orientation: {
      values: ['horizontal', 'vertical'] as const,
      default: 'horizontal',
      control: 'select',
    },
    onTabChange: { control: 'none', event: true },
  },
  faces: [
    { path: 'component.tab.fg', label: 'Tab ink' },
    { path: 'component.tab.selected-fg', label: 'Selected ink' },
    { path: 'component.tab.selected-bg', label: 'Selected fill' },
    { path: 'component.tab.border', label: 'Tab border' },
    { path: 'component.tab-list.bg', label: 'List fill' },
    { path: 'component.tab-list.border', label: 'List border' },
    { path: 'component.tab-list.rail', label: 'Pill rail' },
  ],
  visual: {
    defaultRow: true,
    cross: ['orientation', 'active'],
  },
  examples: true,
} as const satisfies SurfaceContract
