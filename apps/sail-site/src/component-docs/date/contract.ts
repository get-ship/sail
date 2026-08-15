import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for date — axes from stdlib props only.
 * Shared paint: component.field.* (same surface as input).
 */
export const contract = {
  slug: 'date',
  tag: 'date',
  kind: 'host',
  axes: {
    type: {
      values: ['date', 'time', 'datetime-local', 'month', 'week'] as const,
      default: 'date',
      control: 'select',
    },
    size: {
      values: ['xs', 'sm', 'md', 'lg'] as const,
      default: 'md',
      control: 'slider',
    },
    invalid: { default: false, control: 'switch' },
    disabled: { default: false, control: 'switch' },
    placeholder: { default: '', control: 'text', docsSkip: true },
    name: { default: '', control: 'text', docsSkip: true },
    required: { default: false, control: 'switch', docsSkip: true },
    readonly: { default: false, control: 'switch', docsSkip: true },
    value: { default: '', control: 'text', docsSkip: true },
    min: { default: '', control: 'text', docsSkip: true },
    max: { default: '', control: 'text', docsSkip: true },
    step: { default: '', control: 'text', docsSkip: true },
  },
  faces: [
    { path: 'component.field.bg', label: 'Fill' },
    { path: 'component.field.border', label: 'Border' },
    { path: 'component.field.border-hover', label: 'Border hover' },
    { path: 'component.field.error', label: 'Error' },
    { path: 'component.field.fg', label: 'Ink' },
  ],
  visual: {
    defaultRow: true,
    cross: ['type', 'size', 'invalid', 'disabled'],
  },
  examples: true,
} as const satisfies SurfaceContract
