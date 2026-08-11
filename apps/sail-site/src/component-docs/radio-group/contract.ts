import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for radio-group — axes from prop-registry only.
 * Host is layout + typeface; option paint is child <radio> (no component.radio-group.* leaves).
 */
export const contract = {
  slug: 'radio-group',
  tag: 'radio-group',
  kind: 'host',
  axes: {
    value: { default: '', control: 'select' },
    disabled: { default: false, control: 'switch' },
    name: { default: '', control: 'text', docsSkip: true },
    label: { default: '', control: 'text' },
    required: { default: false, control: 'switch' },
    onChange: { control: 'none', event: true },
  },
  faces: [],
  visual: {
    defaultRow: true,
    cross: ['disabled', 'required', 'value'],
  },
  examples: true,
} as const satisfies SurfaceContract
