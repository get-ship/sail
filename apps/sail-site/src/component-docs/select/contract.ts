import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for select — axes from prop-registry only.
 * Shared paint: input role with input/textarea (no select-specific roles).
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
    { path: 'input', label: 'Background' },
    { path: 'border', label: 'Border' },
    { path: 'foreground', label: 'Foreground' },
  ],
  visual: {
    defaultRow: true,
    cross: ['size', 'surface', 'invalid', 'disabled', 'multiple'],
  },
  examples: true,
} as const satisfies SurfaceContract
