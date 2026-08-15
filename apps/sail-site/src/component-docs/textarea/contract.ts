import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for textarea — axes from prop-registry only.
 * Shared paint: input role with input/select (no textarea-specific roles).
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
      docsSkip: true,
    },
    invalid: { default: false, control: 'switch' },
    disabled: { default: false, control: 'switch' },
    placeholder: { default: '', control: 'text' },
    rows: { default: 3, control: 'number' },
  },
  faces: [
    { path: 'input', label: 'Background' },
    { path: 'border', label: 'Border' },
    { path: 'foreground', label: 'Foreground' },
  ],
  visual: {
    defaultRow: true,
    cross: ['size', 'surface', 'invalid', 'disabled'],
  },
  examples: true,
} as const satisfies SurfaceContract
