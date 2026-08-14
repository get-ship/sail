import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for textarea — axes from prop-registry only.
 * Shared paint: component.field.* with input/select (no component.textarea.*).
 */
export const contract = {
  slug: 'textarea',
  tag: 'textarea',
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
    },
    invalid: { default: false, control: 'switch' },
    disabled: { default: false, control: 'switch' },
    placeholder: { default: '', control: 'text' },
    rows: { default: 3, control: 'number' },
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
    cross: ['size', 'surface', 'invalid', 'disabled'],
  },
  examples: true,
} as const satisfies SurfaceContract
