import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for breadcrumb — trail host only, no PE props in stdlib.
 * Faces: component.breadcrumb.current, fg. Compose with nested a / small.
 */
export const contract = {
  slug: 'breadcrumb',
  tag: 'breadcrumb',
  kind: 'host',
  axes: {},
  faces: [
    { path: 'component.breadcrumb.fg', label: 'Trail ink' },
    { path: 'component.breadcrumb.current', label: 'Current' },
  ],
  visual: {
    defaultRow: true,
    cross: [],
  },
  examples: true,
} as const satisfies SurfaceContract
