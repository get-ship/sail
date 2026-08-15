import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for checkbox — axes from stdlib props only.
 * bind:checked is a directive, not a host prop. No component.checkbox.* pack
 * (face tokens align with radio / field hairline).
 */
export const contract = {
  slug: 'checkbox',
  tag: 'checkbox',
  kind: 'host',
  axes: {
    size: {
      values: ['xs', 'sm', 'md', 'lg'] as const,
      default: 'md',
      control: 'slider',
    },
    invalid: { default: false, control: 'switch' },
    disabled: { default: false, control: 'switch' },
    name: { default: '', control: 'text', docsSkip: true },
    required: { default: false, control: 'switch', docsSkip: true },
    value: { default: '', control: 'text', docsSkip: true },
  },
  faces: [],
  visual: {
    defaultRow: true,
    cross: ['size', 'invalid', 'disabled'],
  },
  examples: true,
} as const satisfies SurfaceContract
