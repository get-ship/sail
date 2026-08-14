import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for theme-picker — axes from prop-registry only.
 * Product catalog depth stays operator; scaffold knobs for registry props.
 * Faces: component.theme-picker.* leaves only.
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
    },
    onThemeChange: { control: 'none', event: true },
  },
  faces: [
    { path: 'component.theme-picker.bg', label: 'Panel fill' },
    { path: 'component.theme-picker.fg', label: 'Panel ink' },
    { path: 'component.theme-picker.trigger-fg', label: 'Trigger ink' },
    { path: 'component.theme-picker.option-fg', label: 'Option ink' },
    { path: 'component.theme-picker.option-hover-bg', label: 'Option hover' },
  ],
  visual: {
    defaultRow: true,
    cross: ['value', 'inline', 'modeOnly', 'variant'],
  },
  examples: true,
} as const satisfies SurfaceContract
