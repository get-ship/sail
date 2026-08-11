import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for avatar — axes from prop-registry only.
 * NO invent product variants beyond registry (size · shape · tone).
 */
export const contract = {
  slug: 'avatar',
  tag: 'avatar',
  kind: 'host',
  axes: {
    src: { default: '', control: 'text', docsSkip: true },
    'icon-slug': { default: '', control: 'text', docsSkip: true },
    name: { default: '', control: 'text' },
    initials: { default: '', control: 'text' },
    tone: {
      values: [
        'primary',
        'accent',
        'info',
        'success',
        'warning',
        'destructive',
        'muted',
      ] as const,
      optional: true,
      control: 'swatches',
    },
    size: {
      values: ['md', 'xs', 'sm', 'lg', 'xl'] as const,
      default: 'md',
      control: 'slider',
    },
    shape: {
      values: ['round', 'square'] as const,
      default: 'round',
      control: 'select',
    },
  },
  faces: [
    { path: 'component.avatar.bg', label: 'Fill' },
    { path: 'component.avatar.fg', label: 'Ink' },
  ],
  visual: {
    defaultRow: true,
    cross: ['tone', 'size', 'shape'],
  },
  examples: true,
} as const satisfies SurfaceContract
