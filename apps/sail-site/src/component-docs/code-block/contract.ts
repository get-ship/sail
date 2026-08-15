import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for code-block — axes from prop-registry only.
 * code is content; lang is authoring label (default sail).
 * terminal darkens chrome. line-numbers is a language gap (no highlighter
 * line wrap / platform gutter).
 */
export const contract = {
  slug: 'code-block',
  tag: 'code-block',
  kind: 'host',
  axes: {
    code: { default: '', control: 'text' },
    lang: { default: 'sail', control: 'select' },
    terminal: { default: false, control: 'switch', docsSkip: true },
  },
  faces: [
    { path: 'card', label: 'Background' },
    { path: 'card-foreground', label: 'Foreground' },
    { path: 'border', label: 'Border' },
  ],
  visual: {
    defaultRow: true,
    cross: ['code', 'lang', 'terminal'],
  },
  examples: true,
} as const satisfies SurfaceContract
