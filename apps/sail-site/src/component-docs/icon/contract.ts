import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for icon — size/spin/name/label from prop-registry.
 * No icon-specific pack (foreground is currentColor).
 */
export const contract = {
  slug: 'icon',
  tag: 'icon',
  kind: 'host',
  axes: {
    name: { default: '', control: 'select' },
    src: { default: '', control: 'none', docsSkip: true },
    // Registry type is string; docs UI toggles decorative vs label={name}.
    label: { default: '', control: 'text', docsControl: 'switch' },
    size: {
      values: ['xs', 'sm', 'md', 'lg', 'xl'] as const,
      default: 'md',
      control: 'slider',
    },
    spin: { default: false, control: 'switch' },
  },
  faces: [],
  visual: {
    defaultRow: true,
    cross: ['size', 'spin'],
  },
  examples: true,
} as const satisfies SurfaceContract
