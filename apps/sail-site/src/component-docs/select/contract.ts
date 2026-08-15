import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for select — axes from prop-registry only.
 * Shared paint: component.field.* with input/textarea (no component.select.*).
 * Native closed picker only — not combobox.
 */
export const contract = {
  slug: 'select',
  tag: 'select',
  kind: 'host',
  axes: {
    size: {
      values: ['xs', 'sm', 'md', 'lg'] as const,
      default: 'md',
      control: 'slider',
    },
    surface: {
      values: ['default', 'filled', 'flushed'] as const,
      default: 'default',
      control: 'select',
      docsSkip: true,
    },
    invalid: { default: false, control: 'switch' },
    disabled: { default: false, control: 'switch' },
    multiple: { default: false, control: 'switch' },
  },
  faces: [
    { path: 'component.field.bg', label: 'Fill' },
    { path: 'component.field.border', label: 'Border' },
    { path: 'component.field.border-hover', label: 'Border hover' },
    { path: 'component.field.error', label: 'Error' },
    { path: 'component.field.fg', label: 'Ink' },
    { path: 'component.field.hint', label: 'Hint' },
  ],
  visual: {
    defaultRow: true,
    cross: ['size', 'surface', 'invalid', 'disabled', 'multiple'],
  },
  examples: true,
} as const satisfies SurfaceContract
