import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for input — text field product only.
 * Shared paint: component.field.* (not component.input.*).
 */
export const contract = {
  slug: 'input',
  tag: 'input',
  kind: 'host',
  axes: {
    type: {
      values: [
        'text',
        'email',
        'password',
        'number',
        'search',
        'tel',
        'url',
        'hidden',
      ] as const,
      default: 'text',
      control: 'select',
    },
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
    // Password-reveal is type=password only; docs knob deferred.
    reveal: { default: true, control: 'switch', docsSkip: true },
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
    cross: ['type', 'size', 'surface', 'invalid', 'disabled'],
  },
  examples: true,
} as const satisfies SurfaceContract
