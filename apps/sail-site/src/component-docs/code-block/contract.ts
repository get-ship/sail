import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for code-block — axes from prop-registry only.
 * code is content; lang is authoring label (default sail).
 */
export const contract = {
  slug: 'code-block',
  tag: 'code-block',
  kind: 'host',
  axes: {
    code: { default: '', control: 'text' },
    lang: { default: 'sail', control: 'select' },
  },
  faces: [
    { path: 'component.code-block.bg', label: 'Fill' },
    { path: 'component.code-block.fg', label: 'Ink' },
  ],
  visual: {
    defaultRow: true,
    cross: ['code', 'lang'],
  },
  examples: true,
} as const satisfies SurfaceContract
