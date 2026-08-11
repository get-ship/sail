import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for pagination — axes from prop-registry only.
 * No component.pagination.* leaves; page chrome uses free tokens.
 */
export const contract = {
  slug: 'pagination',
  tag: 'pagination',
  kind: 'host',
  axes: {
    value: { default: '', control: 'text' },
    onPageChange: { control: 'none', event: true },
  },
  faces: [],
  visual: {
    defaultRow: true,
    cross: ['value'],
  },
  examples: true,
} as const satisfies SurfaceContract
