import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for breadcrumb — size axis (SPEC-L1-PASS-1).
 * Faces: component.breadcrumb.current, fg. Compose with nested a / small.
 * LANGUAGE GAP: separator types (content() is empty-only) and collapse
 * (no descendant nth-child / dynamic crumb structure).
 */
export const contract = {
  slug: 'breadcrumb',
  tag: 'breadcrumb',
  kind: 'host',
  axes: {
    size: {
      values: ['sm', 'md', 'lg'] as const,
      default: 'md',
      control: 'slider',
    },
  },
  faces: [
    { path: 'component.breadcrumb.fg', label: 'Trail ink' },
    { path: 'component.breadcrumb.current', label: 'Current' },
  ],
  visual: {
    defaultRow: true,
    cross: ['size'],
  },
  examples: true,
} as const satisfies SurfaceContract
