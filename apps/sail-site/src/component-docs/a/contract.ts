import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for a — axes from prop-registry only.
 * Dual mode: bare (no variant attr) = text link; variant set = button-link.
 * Paint uses tone roles when button-link (no anchor-specific pack).
 */
export const contract = {
  slug: 'a',
  tag: 'a',
  kind: 'host',
  axes: {
    variant: {
      values: ['text', 'default', 'solid', 'soft', 'outline', 'ghost'] as const,
      default: 'text',
      control: 'select',
    },
    tone: {
      values: ['primary', 'info', 'success', 'warning', 'danger', 'accent', 'muted'] as const,
      optional: true,
      control: 'swatches',
    },
    disabled: { default: false, control: 'switch' },
  },
  // Freeform bare link + match arms paint button tokens — no invented pack.
  faces: [],
  visual: {
    defaultRow: true,
    cross: ['variant', 'tone', 'disabled'],
  },
  examples: true,
} as const satisfies SurfaceContract
