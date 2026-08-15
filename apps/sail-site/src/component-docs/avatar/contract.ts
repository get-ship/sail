import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for avatar — axes from prop-registry only.
 * SPEC-L1-PASS-1: size · shape · tone · ring · presence · stack.
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
    ring: { default: false, control: 'switch' },
    presence: {
      values: ['off', 'online', 'busy', 'offline'] as const,
      default: 'off',
      control: 'select',
    },
    stack: { default: false, control: 'switch' },
  },
  faces: [
    { path: 'component.avatar.bg', label: 'Fill' },
    { path: 'component.avatar.fg', label: 'Ink' },
  ],
  visual: {
    defaultRow: true,
    cross: ['tone', 'size', 'shape', 'ring', 'presence', 'stack'],
  },
  examples: true,
} as const satisfies SurfaceContract
