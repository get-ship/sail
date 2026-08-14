import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for pagination — axes from prop-registry only.
 * No component.pagination.* leaves; page chrome uses free tokens.
 * variant default | simple (SPEC-L1-PASS-1): simple is denser Style.
 */
export const contract = {
  slug: 'pagination',
  tag: 'pagination',
  kind: 'host',
  axes: {
    value: { default: '', control: 'text' },
    variant: {
      values: ['default', 'simple'] as const,
      default: 'default',
      control: 'select',
    },
    onPageChange: { control: 'none', event: true },
  },
  faces: [],
  visual: {
    defaultRow: true,
    cross: ['value', 'variant'],
  },
  examples: true,
} as const satisfies SurfaceContract
