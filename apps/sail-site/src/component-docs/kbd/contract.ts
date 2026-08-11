import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for kbd — stdlib props:{} (no PE axes).
 * Faces only; children are the key label.
 */
export const contract = {
  slug: 'kbd',
  tag: 'kbd',
  kind: 'host',
  axes: {},
  /** Docs-only content demo switches (not PE props). */
  demoOnly: ['chord'] as const,
  faces: [
    { path: 'component.kbd.bg', label: 'Fill' },
    { path: 'component.kbd.border', label: 'Border' },
    { path: 'component.kbd.fg', label: 'Ink' },
  ],
  visual: {
    defaultRow: true,
    cross: [],
  },
  examples: true,
} as const satisfies SurfaceContract
