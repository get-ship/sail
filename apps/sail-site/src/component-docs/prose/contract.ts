import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for prose — scope host only (no host props).
 * Host foreground from theme roles; nested type map is framework skins.
 */
export const contract = {
  slug: 'prose',
  tag: 'prose',
  kind: 'host',
  axes: {},
  /** Docs-only content demo switches (not host props). */
  demoOnly: ['sample'] as const,
  faces: [{ path: 'foreground', label: 'Foreground' }],
  visual: {
    defaultRow: true,
    cross: [],
  },
  examples: true,
} as const satisfies SurfaceContract
