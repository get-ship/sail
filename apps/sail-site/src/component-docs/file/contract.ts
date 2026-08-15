import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for file — axes from stdlib props only.
 * variant picker (default) or drop. No component.file.* pack.
 */
export const contract = {
  slug: 'file',
  tag: 'file',
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
    accept: { default: '', control: 'text' },
    multiple: { default: false, control: 'switch' },
    variant: {
      values: ['picker', 'drop'] as const,
      default: 'picker',
      control: 'select',
    },
  },
  faces: [],
  visual: {
    defaultRow: true,
    cross: ['size', 'invalid', 'disabled', 'accept', 'multiple', 'variant'],
  },
  examples: true,
} as const satisfies SurfaceContract
