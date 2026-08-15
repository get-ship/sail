import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for card — axes from prop-registry only.
 * Variants: default | outline | soft | elevated | ghost (SPEC-L1-PASS-1).
 * fill is an alias of soft. spotlight is an alias of accent-bar=static.
 * image is a named slot (no URL prop).
 */
export const contract = {
  slug: 'card',
  tag: 'card',
  kind: 'host',
  axes: {
    tone: {
      values: [
        'primary',
        'info',
        'success',
        'warning',
        'destructive',
        'accent',
        'muted',
      ] as const,
      default: 'muted',
      control: 'swatches',
    },
    variant: {
      values: ['default', 'outline', 'soft', 'elevated', 'ghost'] as const,
      default: 'default',
      control: 'select',
    },
    padding: {
      values: ['md', 'none', 'sm', 'lg'] as const,
      default: 'md',
      control: 'select',
    },
    interactive: { default: false, control: 'switch' },
    'accent-bar': {
      values: ['off', 'hover', 'static'] as const,
      default: 'off',
      control: 'select',
    },
    'status-rail': { default: false, control: 'switch' },
    fill: { default: false, control: 'switch' },
    spotlight: { default: false, control: 'switch', docsSkip: true },
  },
  faces: [
    { path: 'component.card.bg', label: 'Fill' },
    { path: 'component.card.fg', label: 'Ink' },
    { path: 'component.card.texture', label: 'Texture' },
    { path: 'component.card.texture-blend', label: 'Texture blend' },
    { path: 'component.card.texture-opacity', label: 'Texture opacity' },
    { path: 'component.card.backdrop-filter', label: 'Backdrop filter' },
  ],
  visual: {
    defaultRow: true,
    cross: [
      'tone',
      'variant',
      'padding',
      'interactive',
      'accent-bar',
      'status-rail',
      'fill',
      'spotlight',
    ],
  },
  examples: true,
} as const satisfies SurfaceContract
