import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for theme-picker — axes from prop-registry only.
 * Product catalog depth stays operator; scaffold knobs for registry props.
 * Faces: popover role leaves only.
 */
export const contract = {
  slug: 'theme-picker',
  tag: 'theme-picker',
  kind: 'host',
  axes: {
    value: { default: '', control: 'text' },
    inline: { default: false, control: 'switch' },
    modeOnly: { default: false, control: 'switch' },
    variant: {
      values: ['icon', 'menu-row', 'segmented'] as const,
      default: 'icon',
      control: 'select',
      docsSkip: true,
    },
    onThemeChange: { control: 'none', event: true },
  },
  faces: [
    { path: 'popover', label: 'Background' },
    { path: 'foreground', label: 'Foreground' },
    { path: 'border', label: 'Border' },
  ],
  visual: {
    defaultRow: true,
    cross: ['value', 'inline', 'modeOnly', 'variant'],
  },
  examples: true,
} as const satisfies SurfaceContract
