import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for combobox — axes from prop-registry only.
 * Listbox paint: component.combobox.bg / fg (field chrome uses free globals).
 */
export const contract = {
  slug: 'combobox',
  tag: 'combobox',
  kind: 'host',
  axes: {
    placeholder: { default: '', control: 'text' },
    value: { default: '', control: 'text' },
    onSelect: { control: 'none', event: true },
  },
  faces: [
    { path: 'component.combobox.bg', label: 'List fill' },
    { path: 'component.combobox.fg', label: 'List ink' },
  ],
  visual: {
    defaultRow: true,
    cross: ['placeholder', 'value'],
  },
  examples: true,
} as const satisfies SurfaceContract
