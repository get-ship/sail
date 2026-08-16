import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for kbd — stdlib props:{} (no host axes).
 * Faces only; children are the key label.
 */
export const contract = {
  slug: 'kbd',
  tag: 'kbd',
  kind: 'host',
  axes: {},
  /** Docs-only content demo switches (not host props). */
  demoOnly: ['chord'] as const,
  faces: [
    { path: 'card', label: 'Background' },
    { path: 'card-foreground', label: 'Foreground' },
    { path: 'border', label: 'Border' },
  ],
  visual: {
    defaultRow: true,
    cross: [],
  },
  examples: true,
} as const satisfies SurfaceContract
